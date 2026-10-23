//! should-run: the trigger classifier of report §10 (plan Task 5)
use super::{AppError, Result};
use crate::domain::*;
use crate::store::Store;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::read_text as read;

pub fn classify(
    root: &Path,
    diff: Option<&Path>,
    paths: &[String],
    config: Option<&Path>,
    subject: Option<&str>,
) -> Result<Value> {
    let config_path: PathBuf = config
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join("jurygraph.toml"));
    let cfg = trigger::parse_config(&read(&config_path)?)?;
    let mut all: BTreeSet<String> = paths
        .iter()
        .filter(|p| !p.trim().is_empty())
        .cloned()
        .collect();
    if let Some(d) = diff {
        all.extend(trigger::paths_from_diff(&read(d)?)?);
    }
    // a page added by hand has no diff: its recorded source paths land on a rule instead
    if let Some(name) = subject {
        let store = Store::open_existing(root)?;
        let sub = store
            .subject(name)?
            .ok_or_else(|| AppError::NotFound(format!("no subject named {name}")))?;
        for a in &sub.attachments {
            all.insert(super::persona::display_path(root, Path::new(&a.source_path)));
        }
    }
    if all.is_empty() {
        return Err(AppError::InvalidArguments(
            "give --diff <file>, --paths <a,b,...> or --subject <name>".into(),
        ));
    }
    let all: Vec<String> = all.into_iter().collect();
    let c = trigger::classify(&cfg, &all);
    // no store is needed: the stored panel only sharpens the persona count
    let store = Store::open_existing(root).ok();
    let mut spend = Vec::new();
    // a rule that names a panel or lens this store lacks would refuse at `brief --rule`: say so now (it may run with no store)
    let mut warnings: Vec<String> = Vec::new();
    if matches!(c.quorum, Quorum::Low | Quorum::High) {
        for hit in c.hits.iter().filter(|h| h.quorum == c.quorum) {
            for rule in cfg
                .rules
                .iter()
                .filter(|r| r.class == hit.class && r.quorum == c.quorum)
            {
                if let Some(st) = store.as_ref() {
                    if let Some(p) = rule.panel.as_deref() {
                        if !matches!(st.panel(p), Ok(Some(_))) {
                            warnings.push(format!(
                                "rule {}: panel {p} is not in the store; add it with `jurygraph panel add`",
                                rule.class
                            ));
                        }
                    }
                    for l in &rule.lenses {
                        if !matches!(st.lens(l), Ok(Some(_))) {
                            warnings.push(format!(
                                "rule {}: lens {l} is not in the store; add it with `jurygraph lens add`",
                                rule.class
                            ));
                        }
                    }
                }
                let stored = rule.panel.as_deref().and_then(|name| {
                    store
                        .as_ref()?
                        .panel(name)
                        .ok()?
                        .filter(|p| !p.persona_ids.is_empty())
                });
                let personas: Option<usize> = if rule.spend > 0 {
                    Some(rule.spend)
                } else {
                    stored.map(|p| p.persona_ids.len())
                };
                let lenses = rule.lenses.len();
                let mut e = json!({
                    "class": rule.class,
                    "panel": rule.panel,
                    "personas": match personas {
                        Some(n) => json!(n),
                        None => json!(format!("full panel {}", rule.panel.as_deref().unwrap_or("(none)"))),
                    },
                    "lenses": lenses,
                    "detail": rule.detail,
                });
                if let Some(n) = personas {
                    let briefs = n * lenses;
                    e["briefs"] = json!(briefs);
                    if let Some(t) = cfg.tokens_per_brief {
                        e["tokens"] = json!((briefs as u64).saturating_mul(t));
                    }
                }
                spend.push(e);
            }
        }
    }
    let mut out = json!({ "quorum": c.quorum, "hits": c.hits, "unmatched": c.unmatched, "spend": spend });
    if !warnings.is_empty() {
        out["warnings"] = json!(warnings);
    }
    Ok(out)
}
