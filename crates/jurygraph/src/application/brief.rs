//! brief: deterministic, hashed, one file per persona × lens; creates the run (plan Task 4)
use super::{AppError, Result};
use crate::domain::hash::sha256_hex;
use crate::domain::*;
use crate::store::{now_iso, Store};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// Per-attachment cap of a full brief (D3).
const CAP: usize = 65_536;

#[derive(Debug, Clone)]
pub struct BriefArgs {
    pub subject: String,
    pub panel: String,
    pub lens: String,
    pub spend: Option<usize>,
    pub stakes: Vec<(String, Stake)>,
    pub model: String,
    pub comparators: Vec<String>,
    pub detail: Detail,
}

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
    t.push_str("\n## How to cite\n\n- `<label>`: an attachment of the subject\n- `<label>#<node id>`: a node of an aria-json walk\n- `<label>#L<n>`: line n of a diff\n- `cmp:<name>/<label>` (and `#…` as above): an attachment of a comparator\n");
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
    t.push_str("- `anchor_met` is one of the anchor scores at or below `score`.\n");
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

pub fn compose(root: &Path, a: &BriefArgs) -> Result<Value> {
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
    if store.panel(&a.panel)?.is_none() {
        return Err(AppError::NotFound(format!("no panel named {}", a.panel)));
    }
    // a subject whose sources moved since `subject add` would brief stale blobs (final review, minor)
    if let Some(why) = super::verdict::sources_moved(&subject)? {
        return Err(Refusal::SubjectMoved {
            subject: subject.name.clone(),
            why,
        }
        .into());
    }
    let lens = store
        .lens(&a.lens)?
        .ok_or_else(|| AppError::NotFound(format!("no lens named {}", a.lens)))?;
    // the lens name and every persona id become a file name below (I6): checked again here, before the join
    super::slug("lens name", &lens.name)?;
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
    let resolved = super::panel::resolve_in(&store, &a.panel)?;
    let stakes: BTreeMap<String, Stake> = a.stakes.iter().cloned().collect();
    for k in stakes.keys() {
        if !resolved.ids.contains(k) {
            return Err(AppError::InvalidArguments(format!(
                "--stake names {k}, who is not in panel {}",
                a.panel
            )));
        }
    }
    for id in &resolved.ids {
        if !stakes.contains_key(id) {
            return Err(Refusal::PersonaWithoutStake {
                persona: id.clone(),
            }
            .into());
        }
    }
    let size = resolved.ids.len();
    let spend = a.spend.unwrap_or(size);
    if spend == 0 {
        return Err(AppError::InvalidArguments(
            "--spend must be at least 1".into(),
        ));
    }
    if spend > size {
        return Err(Refusal::PanelTooSmall {
            panel: a.panel.clone(),
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

    // the same list `verdict` checks `about` and a pick against: subject labels, then `cmp:<name>` per comparator
    let mut options: Vec<String> = subject
        .attachments
        .iter()
        .map(|x| x.label.clone())
        .collect();
    options.extend(cmps.iter().map(|c| format!("cmp:{}", c.subject.name)));

    let run = RunRow {
        id: 0,
        subject_name: subject.name.clone(),
        subject_version: subject.version,
        subject_hash: subject.hash.clone(),
        panel_name: a.panel.clone(),
        panel_hash: resolved.hash.clone(),
        lens_name: lens.name.clone(),
        lens_version: lens.version,
        model: a.model.clone(),
        spend: spend as i64,
        stakes: stakes.clone(),
        personas: selected.clone(),
        comparators: cmps
            .iter()
            .map(|c| (c.subject.name.clone(), c.subject.version))
            .collect(),
        briefs: vec![],
        created_at: now_iso(),
    };
    // compose every brief first (the text names no run), so nothing can fail between the run row and its files
    let mut texts = Vec::new();
    for id in &selected {
        let persona = store
            .persona(id)?
            .ok_or_else(|| AppError::NotFound(format!("no persona {id}")))?;
        let stake = stakes[id];
        let text = compose_text(
            root, &persona, stake, &lens, &subject, &cmps, &options, a.detail,
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
        "subject": {"name": subject.name, "version": subject.version, "hash": subject.hash},
        "lens": {"name": lens.name, "version": lens.version},
        "model": a.model,
        "spend": spend,
        // what the panel query asked for and what it resolved to: under-resolution is visible, not silent
        "panel": {
            "name": a.panel,
            "personas": resolved.ids.len(),
            "terms": resolved.terms.iter().map(|(t, got)| json!({"tag": t.tag, "wanted": t.count, "got": got})).collect::<Vec<_>>(),
        },
        "detail": if a.detail == Detail::Full { "full" } else { "compact" },
        "briefs": briefs,
        "bytes_total": bytes_total,
    }))
}
