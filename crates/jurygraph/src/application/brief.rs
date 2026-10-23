//! brief: deterministic, hashed, one file per persona × lens; creates the run (plan Task 4)
use super::{AppError, Result};
use crate::domain::hash::{self, sha256_hex};
use crate::domain::*;
use crate::store::{now_iso, Store};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// Per-attachment cap of a full brief (D3).
const CAP: usize = 65_536;

/// What `jurygraph brief` was asked for. A caller that names a `rule` takes its panel, lenses, spend and detail as
/// defaults and may override any of them, with a `reason` (D10).
#[derive(Debug, Clone, Default)]
pub struct BriefArgs {
    pub subject: String,
    /// a saved panel; or `query`, an ad-hoc panel that is not saved; or neither, when a rule names one
    pub panel: Option<String>,
    pub query: Option<String>,
    /// the lenses, one run each; empty = the rule's
    pub lenses: Vec<String>,
    pub spend: Option<usize>,
    /// explicit stakes: they win over the panel's own term stakes
    pub stakes: Vec<(String, Stake)>,
    pub model: String,
    pub comparators: Vec<String>,
    pub detail: Option<Detail>,
    pub rule: Option<String>,
    /// hex seed of the draw, instead of the one derived from the subject's name
    pub draw: Option<String>,
    pub allow_large: bool,
    pub reason: Option<String>,
    pub by: Option<String>,
    /// seats per wave in the `waves` answer (D12); default 30
    pub wave: Option<usize>,
}

/// Seats per wave when the caller does not say (D12).
pub const DEFAULT_WAVE: usize = 30;

/// What a brief costs the agent when the project says nothing (the same default `should-run` documents).
pub(crate) const DEFAULT_TOKENS_PER_BRIEF: u64 = 15_000;

fn stake_meaning(s: Stake) -> &'static str {
    match s {
        Stake::Primary => "you would use it",
        Stake::Secondary => "you buy, approve or support it",
        Stake::Bystander => "you pass by",
    }
}
fn evidence_meaning(e: Evidence) -> &'static str {
    match e {
        Evidence::Seen => "you looked at an attachment of this run and cite it",
        Evidence::Told => "this brief told you",
        Evidence::Recalled => "from your own memory; say so",
    }
}

/// An aria-json walk, one line per node: two spaces per depth, then `<id> <role> "<name>"`.
fn flatten_walk(v: &Value) -> String {
    fn go(nodes: &[Value], depth: usize, out: &mut String) {
        for n in nodes {
            let f = |k: &str| n.get(k).and_then(Value::as_str).unwrap_or("").to_string();
            out.push_str(&format!(
                "{}{} {} {}\n",
                "  ".repeat(depth),
                f("id"),
                f("role"),
                serde_json::to_string(&f("name")).expect("string serialises")
            ));
            if let Some(c) = n.get("children").and_then(Value::as_array) {
                go(c, depth + 1, out);
            }
        }
    }
    let mut out = String::new();
    if let Some(nodes) = v.get("nodes").and_then(Value::as_array) {
        go(nodes, 0, &mut out);
    }
    out
}

fn inline(root: &Path, a: &Attachment) -> Result<Option<String>> {
    if a.media == "image" {
        return Ok(None);
    }
    let p = root.join(".jurygraph").join("blobs").join(&a.hash);
    let bytes = std::fs::read(&p).map_err(|e| AppError::Io(p.clone(), e))?;
    let mut text = if a.media == "aria-json" {
        match serde_json::from_slice::<Value>(&bytes) {
            Ok(v) => flatten_walk(&v),
            Err(_) => String::from_utf8_lossy(&bytes).into_owned(),
        }
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    };
    let mut cut = false;
    if text.len() > CAP {
        let mut n = CAP;
        while !text.is_char_boundary(n) {
            n -= 1;
        }
        text.truncate(n);
        cut = true;
    }
    if !text.ends_with('\n') {
        text.push('\n');
    }
    if cut {
        text.push_str(&format!("[truncated at {CAP} bytes]\n"));
    }
    let longest = text
        .split(|c| c != '`')
        .map(str::len)
        .max()
        .unwrap_or(0)
        .max(2);
    let fence = "`".repeat(longest + 1);
    Ok(Some(format!("{fence}\n{text}{fence}\n")))
}

fn attachment_block(
    root: &Path,
    prefix: &str,
    atts: &[Attachment],
    detail: Detail,
    out: &mut String,
) -> Result<()> {
    for a in atts {
        out.push_str(&format!(
            "- `{prefix}{}` — {}, file `.jurygraph/blobs/{}`, sha256 {}\n",
            a.label, a.media, a.hash, a.hash
        ));
        if detail == Detail::Full {
            if let Some(t) = inline(root, a)? {
                out.push('\n');
                out.push_str(&t);
                out.push('\n');
            }
        }
    }
    Ok(())
}

struct Cmp {
    subject: Subject,
}

#[allow(clippy::too_many_arguments)]
fn compose_text(
    root: &Path,
    persona: &Persona,
    stake: Stake,
    lens: &Lens,
    subject: &Subject,
    cmps: &[Cmp],
    options: &[String],
    detail: Detail,
) -> Result<String> {
    let mut t = String::new();
    t.push_str("jurygraph-brief 1\n\n");
    t.push_str(&format!(
        "# {} judges {} under the {} lens\n\n",
        persona.name, subject.name, lens.name
    ));
    t.push_str(&format!(
        "You are {} ({}). Judge the subject below as this person would, not as an expert. \
The engine that wrote this brief calls no model; you are the judge, and your answer is checked line by line against the rules at the end.\n\n",
        persona.name, persona.id
    ));
    t.push_str("## Persona\n\n");
    if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&persona.profile_json) {
        for (k, label) in [
            ("role", "Role"),
            ("context", "Context"),
            ("daily_tasks", "Daily tasks"),
            ("fears", "Fears"),
            ("never_reads", "Never reads"),
            ("decides_by", "Decides by"),
        ] {
            if let Some(v) = m.get(k).and_then(Value::as_str) {
                t.push_str(&format!("- {label}: {v}\n"));
            }
        }
    }
    t.push_str(&format!(
        "\n## Your stake in this run\n\n{}: {}.\n\nStake rules: {}\n\n",
        stake.name(),
        stake_meaning(stake),
        persona.stake_rules
    ));
    t.push_str(&format!(
        "## Lens: {} (version {})\n\nScores are integers from {} to {}.\n\n",
        lens.name, lens.version, lens.scale[0], lens.scale[1]
    ));
    for d in &lens.dimensions {
        t.push_str(&format!("### {} — {}\n\n", d.id, d.title));
        let mut anchors: Vec<(i64, &String)> = d
            .anchors
            .iter()
            .filter_map(|(k, v)| k.parse::<i64>().ok().map(|n| (n, v)))
            .collect();
        anchors.sort();
        for (n, w) in anchors {
            t.push_str(&format!("- {n}: {w}\n"));
        }
        t.push('\n');
    }
    t.push_str("## Evidence classes this lens allows\n\n");
    for e in &lens.evidence {
        t.push_str(&format!("- `{}`: {}\n", e.name(), evidence_meaning(*e)));
    }
    t.push_str(&format!(
        "\n## Subject: {} (kind {}, version {}, hash {})\n\n",
        subject.name,
        subject.kind.name(),
        subject.version,
        subject.hash
    ));
    t.push_str("Attachments. Open what you need; paths are relative to the workspace root.\n\n");
    attachment_block(root, "", &subject.attachments, detail, &mut t)?;
    if !cmps.is_empty() {
        t.push_str("\n## Comparators (snapshots this run holds)\n\n");
        for c in cmps {
            t.push_str(&format!(
                "### cmp:{} (kind {}, version {}, hash {})\n\n",
                c.subject.name,
                c.subject.kind.name(),
                c.subject.version,
                c.subject.hash
            ));
            attachment_block(
                root,
                &format!("cmp:{}/", c.subject.name),
                &c.subject.attachments,
                detail,
                &mut t,
            )?;
            t.push('\n');
        }
        t.push_str(
            "A comparator this run holds no snapshot of may only be mentioned as `recalled`.\n",
        );
    }
    if !lens.standard.trim().is_empty() {
        t.push_str(&format!(
            "\n## The standard (`told`)\n\n{}\n",
            lens.standard.trim()
        ));
    }
    let scores_options = lens.asks_pick || lens.compares;
    if scores_options && !options.is_empty() {
        t.push_str("\n## Options\n\n`about` names the option a line scores, and the score is that option's own score on the lens anchors. A line without `about` scores the subject as a whole.");
        if lens.compares {
            t.push_str(" Under this lens a line without `about` judges the subject against the comparators: such a `seen` line must cite a held comparator (`cmp:<name>/<label>`) and may also cite subject attachments. A line `about: \"cmp:<name>\"` scores that comparator itself. Never put a comparator in `about` to score the subject against it.");
        }
        t.push_str(
            " Score every option you were asked to weigh, one line per dimension and option.\n\n",
        );
        for o in options {
            t.push_str(&format!("- `{o}`\n"));
        }
    }
    if lens.asks_pick {
        t.push_str("\n## Pick\n\nThis lens also asks you to pick one of the options above and give a reason.\n");
    }
    t.push_str("\n## How to cite\n\n- `<label>`: an attachment of the subject\n- `<label>#<node id>`: a node of an aria-json walk\n- `<label>#L<n>`: line n of a diff\n");
    t.push_str("- `cmp:<name>/<label>` (and `#…` as above): an attachment of a comparator\n");
    t.push_str("\n## Your answer\n\nAnswer with one JSON object and nothing else, in this shape (a filled example for you and this lens):\n\n");
    let example = example_verdict(persona, lens, subject, cmps, options);
    t.push_str("```json\n");
    t.push_str(&serde_json::to_string_pretty(&example).expect("example serialises"));
    t.push_str("\n```\n\nRules, checked when the verdict is ingested:\n\n");
    if scores_options {
        t.push_str("- One line per dimension and option: at most one line for each (`dimension`, `about`) pair, and at least one line for every lens dimension.\n");
        t.push_str("- `about`, when given, is one of the options above.\n");
    } else {
        t.push_str("- One line per dimension: this lens scores the subject as a whole; leave `about` out.\n");
    }
    t.push_str(&format!(
        "- `score` is an integer from {} to {}.\n",
        lens.scale[0], lens.scale[1]
    ));
    let anchors: Vec<String> = lens.anchor_scores().iter().map(i64::to_string).collect();
    t.push_str(&format!(
        "- `anchor_met`: one of {}, at or below your `score`.\n",
        anchors.join(", ")
    ));
    t.push_str(
        "- A `seen` line needs at least one citation that names something this run holds.\n",
    );
    if scores_options {
        t.push_str("- A line `about` a comparator the run holds no snapshot of (`cmp:<name>`) is `recalled` only.\n");
    }
    t.push_str(&format!(
        "- A score at the bottom of the scale ({}) needs a blocker that says what failed.\n",
        lens.scale[0]
    ));
    if lens.compares {
        t.push_str("- Under this lens a `seen` line must be `about` a comparator this run holds (`cmp:<name>`) or cite one (`cmp:<name>/<label>`): a comparison with no comparator snapshot cannot be seen.\n");
    }
    t.push_str(
        "- `blockers` lists what stops you, one sentence each; leave it empty when nothing does.\n",
    );
    if lens.asks_pick {
        t.push_str(
            "- `pick` is required: `choice` is one of the options above, with a `reason`.\n",
        );
    } else {
        t.push_str("- Leave `pick` out.\n");
    }
    Ok(t)
}

fn example_line(
    dimension: &str,
    mid: i64,
    evidence: Evidence,
    cites: Vec<String>,
    about: Option<String>,
    note: &str,
) -> Value {
    let mut line = serde_json::Map::new();
    line.insert("dimension".into(), json!(dimension));
    if let Some(a) = about {
        line.insert("about".into(), json!(a));
    }
    line.insert("score".into(), json!(mid));
    line.insert("anchor_met".into(), json!(mid));
    line.insert("evidence".into(), json!(evidence.name()));
    line.insert("citations".into(), json!(cites));
    line.insert("blockers".into(), json!([]));
    line.insert("note".into(), json!(note));
    Value::Object(line)
}

fn example_verdict(
    persona: &Persona,
    lens: &Lens,
    subject: &Subject,
    cmps: &[Cmp],
    options: &[String],
) -> Value {
    // a comparing lens with no comparator in the run cannot show a seen line (D6 A2): show the next class it allows
    let evidence = lens
        .evidence
        .iter()
        .copied()
        .find(|e| !(lens.compares && cmps.is_empty() && *e == Evidence::Seen))
        .unwrap_or(Evidence::Recalled);
    let anchors = lens.anchor_scores();
    let mid = anchors
        .get(anchors.len() / 2)
        .copied()
        .unwrap_or(lens.scale[0]);
    let mut lines = Vec::new();
    let cmp_first = cmps
        .first()
        .and_then(|c| c.subject.attachments.first().map(|a| (c, a.label.clone())));
    for (i, d) in lens.dimensions.iter().enumerate() {
        let mut cites: Vec<String> = Vec::new();
        if evidence == Evidence::Seen {
            if let Some(a) = subject.attachments.first() {
                cites.push(a.label.clone());
            }
            if lens.compares {
                if let Some((c, label)) = &cmp_first {
                    cites.push(format!("cmp:{}/{}", c.subject.name, label));
                }
            }
        }
        lines.push(example_line(
            &d.id,
            mid,
            evidence,
            cites,
            None,
            "one sentence in your own voice",
        ));
        if i == 0 && lens.compares && evidence == Evidence::Seen {
            if let Some((c, label)) = &cmp_first {
                lines.push(example_line(
                    &d.id,
                    mid,
                    evidence,
                    vec![format!("cmp:{}/{}", c.subject.name, label)],
                    Some(format!("cmp:{}", c.subject.name)),
                    "this line scores the comparator itself",
                ));
            }
        }
    }
    let mut v = serde_json::Map::new();
    v.insert("persona".into(), json!(persona.id));
    v.insert("lines".into(), Value::Array(lines));
    if lens.asks_pick {
        v.insert(
            "pick".into(),
            json!({"choice": options.first().cloned().unwrap_or_default(), "reason": "why, in one sentence"}),
        );
    }
    Value::Object(v)
}


/// The panel a brief draws from.
enum PanelSrc {
    Saved(String),
    Adhoc(String),
}

/// Everything the runs of one brief invocation share.
struct Shared<'a> {
    subject: &'a Subject,
    cmps: &'a [Cmp],
    panel_name: &'a str,
    resolved: &'a super::panel::Resolved,
    stakes: &'a BTreeMap<String, Stake>,
    selected: &'a [String],
    model: &'a str,
    detail: Detail,
    seed_hex: &'a str,
    override_json: Option<&'a Value>,
    wave: usize,
}

/// Compose and write the briefs of one lens and create its run.
fn one_run(root: &Path, store: &Store, sh: &Shared, lens: &Lens) -> Result<Value> {
    // the same list `verdict` checks `about` and a pick against: subject labels, then `cmp:<name>` per comparator
    let mut options: Vec<String> = sh
        .subject
        .attachments
        .iter()
        .map(|x| x.label.clone())
        .collect();
    options.extend(sh.cmps.iter().map(|c| format!("cmp:{}", c.subject.name)));
    let run = RunRow {
        id: 0,
        subject_name: sh.subject.name.clone(),
        subject_version: sh.subject.version,
        subject_hash: sh.subject.hash.clone(),
        panel_name: sh.panel_name.to_string(),
        panel_hash: sh.resolved.hash.clone(),
        lens_name: lens.name.clone(),
        lens_version: lens.version,
        model: sh.model.to_string(),
        spend: sh.selected.len() as i64,
        stakes: sh.stakes.clone(),
        personas: sh.selected.to_vec(),
        comparators: sh
            .cmps
            .iter()
            .map(|c| (c.subject.name.clone(), c.subject.version))
            .collect(),
        briefs: vec![],
        created_at: now_iso(),
        draw_seed: Some(sh.seed_hex.to_string()),
        override_json: sh.override_json.cloned(),
    };
    // compose every brief first (the text names no run), so nothing can fail between the run row and its files
    let mut texts = Vec::new();
    for id in sh.selected {
        let persona = store
            .persona(id)?
            .ok_or_else(|| AppError::NotFound(format!("no persona {id}")))?;
        let stake = sh.stakes[id];
        let text = compose_text(
            root, &persona, stake, lens, sh.subject, sh.cmps, &options, sh.detail,
        )?;
        texts.push((id.clone(), persona.version, stake, text));
    }
    let run_id = store.insert_run(&run)?;
    let dir_rel = format!(".jurygraph/runs/{run_id}");
    let dir = root.join(&dir_rel);
    // a write that fails leaves no orphan run (final review: an orphan clears nothing and confuses check)
    let written = (|| -> Result<Vec<BriefRef>> {
        std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(dir.clone(), e))?;
        let mut refs = Vec::new();
        for (id, version, stake, text) in &texts {
            let rel = format!("{dir_rel}/{id}--{}.md", lens.name);
            let path = root.join(&rel);
            std::fs::write(&path, text.as_bytes()).map_err(|e| AppError::Io(path.clone(), e))?;
            refs.push(BriefRef {
                persona: id.clone(),
                persona_version: *version,
                stake: *stake,
                path: rel,
                hash: sha256_hex(text.as_bytes()),
                bytes: text.len(),
            });
        }
        store.set_run_briefs(run_id, &refs)?;
        Ok(refs)
    })();
    let refs = match written {
        Ok(r) => r,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            store.delete_run(run_id)?;
            return Err(e);
        }
    };
    let bytes_total: usize = refs.iter().map(|r| r.bytes).sum();
    let briefs: Vec<Value> = refs
        .iter()
        .map(|r| json!({"persona": r.persona, "stake": r.stake.name(), "path": r.path, "hash": r.hash, "bytes": r.bytes}))
        .collect();
    Ok(json!({
        "run": run_id,
        "subject": {"name": sh.subject.name, "version": sh.subject.version, "hash": sh.subject.hash},
        "lens": {"name": lens.name, "version": lens.version},
        "model": sh.model,
        "spend": sh.selected.len(),
        // what the panel query asked for and what it resolved to: under-resolution is visible, not silent
        "panel": {
            "name": sh.panel_name,
            "personas": sh.resolved.ids.len(),
            "terms": sh.resolved.terms.iter().map(|(t, got)| {
                let mut v = json!({"tag": t.tag, "wanted": t.count, "got": got});
                if let Some(s) = t.stake { v["stake"] = json!(s.name()); }
                v
            }).collect::<Vec<_>>(),
        },
        "detail": if sh.detail == Detail::Full { "full" } else { "compact" },
        // run one wave, ingest it, then the next (D12): persona ids in the order of `briefs`
        "waves": sh.selected.chunks(sh.wave).map(|c| c.to_vec()).collect::<Vec<_>>(),
        "briefs": briefs,
        "bytes_total": bytes_total,
    }))
}

fn detail_name(d: Detail) -> &'static str {
    if d == Detail::Full {
        "full"
    } else {
        "compact"
    }
}

pub fn compose(root: &Path, a: &BriefArgs) -> Result<Value> {
    if a.wave == Some(0) {
        return Err(AppError::InvalidArguments("--wave must be at least 1".into()));
    }
    if a.comparators.iter().any(|c| c.trim().is_empty()) {
        return Err(AppError::InvalidArguments(
            "--comparator has an empty element; name each comparator".into(),
        ));
    }
    if a.model.trim().is_empty() {
        return Err(AppError::InvalidArguments(
            "--model names the model the run is pinned to; it cannot be empty".into(),
        ));
    }
    let store = Store::open(root)?;
    let subject = store
        .subject(&a.subject)?
        .ok_or_else(|| AppError::NotFound(format!("no subject named {}", a.subject)))?;
    if subject.is_comparator {
        return Err(AppError::InvalidArguments(format!(
            "{} is a comparator; give it with --comparator and brief a subject",
            a.subject
        )));
    }

    // the rule, when one is named: its panel, lenses, spend and detail are the defaults (D9)
    let cfg_path = root.join("jurygraph.toml");
    let cfg: Option<TriggerConfig> = match &a.rule {
        Some(_) => Some(trigger::parse_config(&super::read_text(&cfg_path)?)?),
        // a toml that is there but malformed is refused, not ignored (M7)
        None => match std::fs::read_to_string(&cfg_path) {
            Ok(t) => Some(trigger::parse_config(&t)?),
            Err(_) => None,
        },
    };
    let tokens_per_brief = cfg
        .as_ref()
        .and_then(|c| c.tokens_per_brief)
        .unwrap_or(DEFAULT_TOKENS_PER_BRIEF);
    let rule: Option<&TriggerRule> = match (&a.rule, &cfg) {
        (Some(class), Some(cfg)) => {
            let r = cfg.rules.iter().find(|r| r.class == *class).ok_or_else(|| {
                AppError::NotFound(format!("no rule {class} in {}", cfg_path.display()))
            })?;
            // no override can run a jury on a human class as if it decided
            if HUMAN_CLASSES.contains(&trigger::normalise_class(&r.class).as_str())
                || r.quorum == Quorum::Human
            {
                return Err(Refusal::HumanClassUnderPersonaQuorum {
                    class: r.class.clone(),
                    quorum: "a persona jury".into(),
                }
                .into());
            }
            if r.quorum == Quorum::None {
                return Err(AppError::InvalidArguments(format!(
                    "rule {class} has quorum none: no jury runs on it"
                )));
            }
            // a page that falls under a human rule is never briefed under another rule's name, whatever the overrides
            let paths: Vec<String> = subject
                .attachments
                .iter()
                .map(|x| super::persona::display_path(root, Path::new(&x.source_path)))
                .collect();
            for hit in trigger::classify(cfg, &paths).hits {
                if hit.quorum == Quorum::Human
                    || HUMAN_CLASSES.contains(&trigger::normalise_class(&hit.class).as_str())
                {
                    return Err(Refusal::HumanPathUnderPersonaQuorum {
                        class: hit.class.clone(),
                        path: hit.paths.first().cloned().unwrap_or_default(),
                    }
                    .into());
                }
            }
            Some(r)
        }
        _ => None,
    };

    if a.panel.is_some() && a.query.is_some() {
        return Err(AppError::InvalidArguments(
            "give --panel (a saved panel) or --query (an ad-hoc one), not both".into(),
        ));
    }
    let panel_src = match (&a.query, &a.panel, rule) {
        (Some(q), _, _) => PanelSrc::Adhoc(q.clone()),
        (None, Some(p), _) => PanelSrc::Saved(p.clone()),
        (None, None, Some(r)) => PanelSrc::Saved(r.panel.clone().unwrap_or_default()),
        (None, None, None) => {
            return Err(AppError::InvalidArguments(
                "give --panel <saved panel>, --query \"<terms>\" or --rule <class>".into(),
            ))
        }
    };
    let mut lens_names: Vec<String> = Vec::new();
    for l in &a.lenses {
        if !lens_names.contains(l) {
            lens_names.push(l.clone());
        }
    }
    let lenses_from_rule = lens_names.is_empty();
    if lenses_from_rule {
        match rule {
            Some(r) => lens_names = r.lenses.clone(),
            None => {
                return Err(AppError::InvalidArguments(
                    "give --lens <name> or --rule <class>".into(),
                ))
            }
        }
    }
    let detail = a
        .detail
        .or(rule.map(|r| r.detail))
        .unwrap_or(Detail::Compact);

    // a rule that names a panel or a lens the store lacks is its own refusal; a caller's own choice is a plain not_found
    let (panel_from_rule, rule_class) = (
        matches!((&a.panel, &a.query), (None, None)),
        rule.map(|r| r.class.clone()).unwrap_or_default(),
    );
    if let PanelSrc::Saved(name) = &panel_src {
        if store.panel(name)?.is_none() {
            return Err(if panel_from_rule && rule.is_some() {
                Refusal::RulePanelUnknown {
                    rule: rule_class,
                    panel: name.clone(),
                }
                .into()
            } else {
                AppError::NotFound(format!("no panel named {name}"))
            });
        }
    }
    // a subject whose sources moved since `subject add` would brief stale blobs (final review, minor)
    if let Some(why) = super::verdict::sources_moved(&subject)? {
        return Err(Refusal::SubjectMoved {
            subject: subject.name.clone(),
            why,
        }
        .into());
    }
    let mut lenses: Vec<Lens> = Vec::new();
    for n in &lens_names {
        let lens = store.lens(n)?.ok_or_else(|| {
            if lenses_from_rule {
                Refusal::RuleLensUnknown {
                    rule: rule_class.clone(),
                    lens: n.clone(),
                }
                .into()
            } else {
                AppError::NotFound(format!("no lens named {n}"))
            }
        })?;
        // the lens name and every persona id become a file name below (I6): checked again here, before the join
        super::slug("lens name", &lens.name)?;
        lenses.push(lens);
    }
    let mut cmp_names = a.comparators.clone();
    cmp_names.sort();
    cmp_names.dedup();
    let mut cmps = Vec::new();
    for n in &cmp_names {
        let c = store
            .subject(n)?
            .ok_or_else(|| AppError::NotFound(format!("no comparator named {n}")))?;
        if !c.is_comparator {
            return Err(AppError::InvalidArguments(format!(
                "{n} is a subject, not a comparator; add it with `jurygraph comparator add`"
            )));
        }
        cmps.push(Cmp { subject: c });
    }

    // the draw: seeded from the subject's NAME and the panel's name, so a second round draws the same seats
    let (panel_name, seed_name) = match &panel_src {
        PanelSrc::Saved(n) => (n.clone(), n.clone()),
        // seeded by the parsed, normalised terms: spacing and case in the query never change who sits
        PanelSrc::Adhoc(q) => ("(query)".to_string(), format!("query:{}", query::normalise(&query::parse(q)?))),
    };
    let seed = match &a.draw {
        Some(h) => super::panel::parse_seed(h)?,
        None => hash::draw_seed(&subject.name, &seed_name),
    };
    let resolved = match &panel_src {
        PanelSrc::Saved(n) => super::panel::resolve_in(&store, n, Some(seed))?,
        PanelSrc::Adhoc(q) => {
            let terms = query::parse(q)?;
            super::panel::check_seats(super::panel::seats(&terms), a.allow_large)?;
            super::panel::draw_query(&query::normalise(&terms), &store.personas()?, Some(seed))?
        }
    };
    if let Some((t, _)) = resolved.terms.iter().find(|(_, got)| *got == 0) {
        return Err(Refusal::PanelTermEmpty {
            panel: panel_name.clone(),
            tag: t.tag.clone(),
        }
        .into());
    }

    // a stake per persona: --stake wins over the panel's term, a term over nothing
    let explicit: BTreeMap<String, Stake> = a.stakes.iter().cloned().collect();
    for k in explicit.keys() {
        if !resolved.ids.contains(k) {
            return Err(AppError::InvalidArguments(format!(
                "--stake names {k}, who is not in panel {panel_name}"
            )));
        }
    }
    let mut stakes: BTreeMap<String, Stake> = BTreeMap::new();
    let mut stake_overrides: Vec<Value> = Vec::new();
    for (id, term) in resolved.ids.iter().zip(&resolved.stakes) {
        let stake = match (explicit.get(id), term) {
            (Some(g), Some(t)) if g != t => {
                stake_overrides.push(
                    json!({"persona": id, "panel": t.name(), "given": g.name()}),
                );
                *g
            }
            (Some(g), _) => *g,
            (None, Some(t)) => *t,
            (None, None) => return Err(Refusal::PersonaWithoutStake { persona: id.clone() }.into()),
        };
        stakes.insert(id.clone(), stake);
    }
    let size = resolved.ids.len();
    let spend_arg = a.spend.or(rule.filter(|r| r.spend > 0).map(|r| r.spend));
    // a rule's `spend = 0` is the whole panel, capped at 12 (1000 with --allow-large): the rest are cut in draw order
    let cap = if a.allow_large { SEAT_HARD_MAX } else { SEAT_CEILING };
    let capped = rule.is_some() && spend_arg.is_none() && size > cap;
    let spend = if capped { cap } else { spend_arg.unwrap_or(size) };
    if spend == 0 {
        return Err(AppError::InvalidArguments(
            "--spend must be at least 1".into(),
        ));
    }
    if spend > size {
        return Err(Refusal::PanelTooSmall {
            panel: panel_name.clone(),
            got: size,
            spend,
        }
        .into());
    }
    let mut ordered: Vec<&String> = resolved.ids.iter().collect();
    ordered.sort_by_key(|id| stakes[*id]); // stable: panel order within a stake
    let selected: Vec<String> = ordered.into_iter().take(spend).cloned().collect();
    for id in &selected {
        super::slug("persona id", id)?;
    }
    // D10: 12 seats a brief unless the caller says so; never more than 1000
    super::panel::check_seats(selected.len(), a.allow_large)?;
    let large = selected.len() > SEAT_CEILING;

    // where the caller departs from the rule, or runs more than 12 seats, the run says so and says why
    let override_json: Option<Value> = match rule {
        // no rule, no departure; but a brief above 12 seats still says why, with `rule: null` (D12, review M9)
        None if large => {
            let reason = a.reason.as_deref().map(str::trim).unwrap_or("");
            if reason.is_empty() {
                return Err(Refusal::OverrideWithoutReason {
                    rule: "(none)".into(),
                    fields: "seats".into(),
                }
                .into());
            }
            Some(json!({
                "rule": Value::Null,
                "fields": {"seats": {"rule": SEAT_CEILING, "given": selected.len()}},
                "reason": reason,
                "by": a.by.as_deref().map(str::trim).filter(|b| !b.is_empty()).unwrap_or("agent"),
            }))
        }
        None => None,
        Some(r) => {
            let mut fields = serde_json::Map::new();
            match &panel_src {
                PanelSrc::Adhoc(q) => {
                    fields.insert(
                        "panel".into(),
                        json!({"rule": r.panel, "given": format!("query: {q}")}),
                    );
                }
                PanelSrc::Saved(p) if Some(p) != r.panel.as_ref() => {
                    fields.insert("panel".into(), json!({"rule": r.panel, "given": p}));
                }
                PanelSrc::Saved(_) => {}
            }
            let (mut given_l, mut rule_l) = (lens_names.clone(), r.lenses.clone());
            given_l.sort();
            rule_l.sort();
            if !lenses_from_rule && given_l != rule_l {
                fields.insert("lenses".into(), json!({"rule": r.lenses, "given": lens_names}));
            }
            if let Some(n) = a.spend {
                if n != r.spend && !(r.spend == 0 && n == size) {
                    fields.insert("spend".into(), json!({"rule": r.spend, "given": n}));
                }
            }
            if let Some(d) = a.detail {
                if d != r.detail {
                    fields.insert(
                        "detail".into(),
                        json!({"rule": detail_name(r.detail), "given": detail_name(d)}),
                    );
                }
            }
            if !stake_overrides.is_empty() {
                let pick = |k: &str| -> serde_json::Map<String, Value> {
                    stake_overrides
                        .iter()
                        .map(|o| (o["persona"].as_str().unwrap_or("").to_string(), o[k].clone()))
                        .collect()
                };
                fields.insert(
                    "stakes".into(),
                    json!({"rule": Value::Object(pick("panel")), "given": Value::Object(pick("given"))}),
                );
            }
            if a.draw.is_some() {
                let derived = super::panel::hex_seed(hash::draw_seed(&subject.name, &seed_name));
                if derived != super::panel::hex_seed(seed) {
                    fields.insert(
                        "draw".into(),
                        json!({"rule": derived, "given": super::panel::hex_seed(seed)}),
                    );
                }
            }
            if large {
                fields.insert(
                    "seats".into(),
                    json!({"rule": SEAT_CEILING, "given": selected.len()}),
                );
            }
            if fields.is_empty() {
                None
            } else {
                let reason = a.reason.as_deref().map(str::trim).unwrap_or("");
                if reason.is_empty() {
                    return Err(Refusal::OverrideWithoutReason {
                        rule: r.class.clone(),
                        fields: fields.keys().cloned().collect::<Vec<_>>().join(", "),
                    }
                    .into());
                }
                Some(json!({
                    "rule": r.class,
                    "fields": Value::Object(fields),
                    "reason": reason,
                    "by": a.by.as_deref().map(str::trim).filter(|b| !b.is_empty()).unwrap_or("agent"),
                }))
            }
        }
    };

    // the cost, before anyone spends it; an overflowing estimate is refused before a single run is written
    let briefs = selected
        .len()
        .checked_mul(lenses.len())
        .filter(|b| (*b as u64).checked_mul(tokens_per_brief).is_some())
        .ok_or(Refusal::PanelTooLarge {
            seats: usize::MAX,
            ceiling: SEAT_HARD_MAX,
        })?;
    let tokens = briefs as u64 * tokens_per_brief;
    let seed_hex = super::panel::hex_seed(seed);
    let sh = Shared {
        subject: &subject,
        cmps: &cmps,
        panel_name: &panel_name,
        resolved: &resolved,
        stakes: &stakes,
        selected: &selected,
        model: &a.model,
        detail,
        seed_hex: &seed_hex,
        override_json: override_json.as_ref(),
        wave: a.wave.unwrap_or(DEFAULT_WAVE),
    };
    let mut runs = Vec::new();
    for lens in &lenses {
        runs.push(one_run(root, &store, &sh, lens)?);
    }

    let mut out = if a.rule.is_none() && runs.len() == 1 {
        runs.remove(0)
    } else {
        json!({ "rule": a.rule, "runs": runs })
    };
    // the cost, before anyone spends it: briefs × the project's tokens per brief
    out["estimate"] = json!({"briefs": briefs, "tokens": tokens});
    if capped {
        out["capped"] = json!(true);
    }
    out["draw"] = json!({"seed": seed_hex});
    if large {
        out["large"] = json!(true);
    }
    if !stake_overrides.is_empty() {
        out["stake_overrides"] = json!(stake_overrides);
    }
    if let Some(o) = override_json {
        out["override"] = o;
    }
    Ok(out)
}
