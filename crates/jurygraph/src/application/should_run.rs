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
    if all.is_empty() {
        return Err(AppError::InvalidArguments(
            "give --diff <file> or --paths <a,b,...>".into(),
        ));
    }
    let all: Vec<String> = all.into_iter().collect();
    let c = trigger::classify(&cfg, &all);
    // no store is needed: the stored panel only sharpens the persona count
    let store = Store::open_existing(root).ok();
    let mut spend = Vec::new();
    if matches!(c.quorum, Quorum::Low | Quorum::High) {
        for hit in c.hits.iter().filter(|h| h.quorum == c.quorum) {
            for rule in cfg
                .rules
                .iter()
                .filter(|r| r.class == hit.class && r.quorum == c.quorum)
            {
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
                        e["tokens"] = json!(briefs as u64 * t);
                    }
                }
                spend.push(e);
            }
        }
    }
    Ok(json!({ "quorum": c.quorum, "hits": c.hits, "unmatched": c.unmatched, "spend": spend }))
}
