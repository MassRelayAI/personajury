//! The reverse generators (D35): `trait suggest` turns a persona's prose into a brief and checks the agent's answer,
//! `trait observed` counts what a persona did in its verdicts. Neither calls a model and neither writes a trait back.
use super::{AppError, Result};
use crate::domain::hash::sha256_hex;
use crate::domain::observed::{self, RunData};
use crate::domain::traits::{check_held, TraitDef, TraitValue, MAX_PERSONA_TRAITS};
use crate::domain::*;
use crate::store::Store;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// The persona's prose fields in the order the brief prints them: (label, key of the profile or `stake_rules`, text).
fn prose_of(p: &Persona) -> Vec<(&'static str, String)> {
    let profile: Value = serde_json::from_str(&p.profile_json).unwrap_or(Value::Null);
    let get = |k: &str| profile.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    vec![
        ("Role", get("role")),
        ("Context", get("context")),
        ("Daily tasks", get("daily_tasks")),
        ("Fears", get("fears")),
        ("Never reads", get("never_reads")),
        ("Decides by", get("decides_by")),
        ("Stake rules", p.stake_rules.clone()),
    ]
}
/// The floor of a quote (D35 as ruled after the v0.6-B review): at least 3 words and at least 12 characters, folded.
pub const MIN_QUOTE_WORDS: usize = 3;
pub const MIN_QUOTE_CHARS: usize = 12;
/// Runs of whitespace folded to one space, the ends trimmed.
fn fold(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `trait suggest --persona ID [--set NAME]` (D35): write `.personajury/generate/trait-suggest-<id>-<hash8>.md`, a brief
/// that lays the persona's prose beside the anchors of the candidate traits and asks for values each with a verbatim
/// quote. `hash8` is the first 8 hex of sha256(persona hash, then the sorted `id@version` of the candidates).
pub fn suggest(root: &Path, persona: Option<&str>, set: Option<&str>) -> Result<Value> {
    let id = persona.ok_or_else(|| AppError::InvalidArguments("trait suggest needs --persona ID (or --check FILE)".into()))?;
    super::slug("persona id", id)?;
    let store = Store::open_existing(root)?;
    let p = store.persona(id)?.ok_or_else(|| AppError::NotFound(format!("no persona {id}")))?;
    let defs = store.trait_map()?;
    let ids: Vec<String> = match set {
        Some(name) => {
            let s = store.trait_set(name)?.ok_or_else(|| Refusal::TraitSetUnknown { name: name.to_string() })?;
            let mut v = s.traits;
            v.sort();
            v.dedup();
            v
        }
        None => defs.keys().cloned().collect(),
    };
    let cands: Vec<&TraitDef> = ids.iter().filter_map(|i| defs.get(i)).collect();
    let mut keyed: Vec<String> = cands.iter().map(|d| format!("{}@{}", d.id, d.version)).collect();
    keyed.sort();
    let hash8 = sha256_hex(format!("{}\n{}", p.hash, keyed.join("\n")).as_bytes())[..8].to_string();

    let mut t = String::new();
    t.push_str("personajury-trait-suggest 1\n\n");
    t.push_str(&format!("# Traits for {} ({}), from their own words\n\n", p.name, p.id));
    t.push_str("## The persona's prose\n\n");
    for (label, text) in prose_of(&p) {
        t.push_str(&format!("- {label}: {}\n", fold(&text)));
    }
    t.push_str("\n## Candidate traits\n\n");
    for d in &cands {
        let kind = match d.shape.as_str() {
            "band" => "band 1\u{2013}5".to_string(),
            "category" => if d.multi { "category, several allowed".to_string() } else { "category".to_string() },
            _ => format!("words, at most {} characters", d.max_chars.unwrap_or(80).min(80)),
        };
        t.push_str(&format!("### {} \u{2014} {} ({kind})\n\n", d.id, d.title));
        match d.shape.as_str() {
            "band" => {
                for (k, a) in &d.anchors {
                    t.push_str(&format!("- {k}: {a}\n"));
                }
            }
            "category" => {
                for o in &d.options {
                    t.push_str(&format!("- `{}`: {}\n", o.id, o.sentence));
                }
            }
            _ => t.push_str("- a short phrase that names the work; no digit, never \"You are\" or \"I am\"\n"),
        }
        t.push('\n');
    }
    t.push_str(
        "## Your task\n\n\
Propose a value only where a sentence of the prose above supports it, and copy that sentence (or a phrase of it, verbatim) as the \
evidence: at least 3 words and 12 characters, from one field (a quote never runs from one field into the next). Leave out every trait the prose does not support. At most 8. A band takes an integer from 1 to 5 (2 and 4 lean toward \
their neighbour); a category takes one option id. Answer with one JSON object:\n\n\
{ \"persona\": \"<id>\", \"traits\": { \"<trait id>\": <value>, ... }, \"evidence\": { \"<trait id>\": \"<verbatim quote>\", ... } }\n\n\
Then check it with `personajury trait suggest --check <file>`; the engine verifies every quote and answers the persona file with \
the traits set. Save that file and run `personajury persona add --file <file>`: the persona gets a new version.\n",
    );
    let dir = root.join(".personajury").join("generate");
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(dir.clone(), e))?;
    let rel = format!(".personajury/generate/trait-suggest-{}-{hash8}.md", p.id);
    let path = root.join(&rel);
    std::fs::write(&path, t.as_bytes()).map_err(|e| AppError::Io(path.clone(), e))?;
    Ok(json!({
        "persona": p.id, "path": rel, "hash": sha256_hex(t.as_bytes()), "bytes": t.len(),
        "candidates": cands.len(), "set": set,
    }))
}

/// `trait suggest --check FILE [--persona ID]` (D35): validate the answer as `persona add` would, require a quote for
/// every trait that is a substring of the persona's prose (whitespace folded), and answer the merged persona file as data.
/// It writes nothing: the caller saves the file and runs `persona add`.
pub fn suggest_check(root: &Path, file: &Path, persona: Option<&str>) -> Result<Value> {
    let text = super::read_text(file)?;
    suggest_check_text(root, &text, &file.display().to_string(), persona)
}
/// `suggest_check` over the answer's text (the MCP tool gives it inline); `label` names it in a refusal.
pub fn suggest_check_text(root: &Path, text: &str, label: &str, persona: Option<&str>) -> Result<Value> {
    let v: Value =
        serde_json::from_str(text).map_err(|e| AppError::InvalidArguments(format!("{label} is not a suggestion: {e}")))?;
    let bad = |why: &str| AppError::InvalidArguments(format!("{label} is not a suggestion: {why}"));
    let id = v.get("persona").and_then(Value::as_str).ok_or_else(|| bad("`persona` is missing"))?;
    if let Some(want) = persona {
        if want != id {
            return Err(AppError::InvalidArguments(format!("--persona {want} but the file is for {id}")));
        }
    }
    let traits: BTreeMap<String, TraitValue> = serde_json::from_value(v.get("traits").cloned().ok_or_else(|| bad("`traits` is missing"))?)
        .map_err(|e| bad(&format!("traits: {e}")))?;
    let evidence: BTreeMap<String, String> = match v.get("evidence") {
        None => BTreeMap::new(),
        Some(e) => serde_json::from_value(e.clone()).map_err(|e| bad(&format!("evidence: {e}")))?,
    };
    let store = Store::open_existing(root)?;
    let p = store.persona(id)?.ok_or_else(|| AppError::NotFound(format!("no persona {id}")))?;
    // as `persona add` would: the refused list and the ceiling, then the store's traits and their values
    let defs = store.trait_map()?;
    check_held(&traits, &defs)?;
    // each quote stands inside one prose field: a quote that runs from one field into the next is no sentence of the prose
    let fields: Vec<String> = prose_of(&p).iter().map(|(_, t)| fold(t)).collect();
    let mut quotes = BTreeMap::new();
    for tid in traits.keys() {
        let q = fold(evidence.get(tid).map(String::as_str).unwrap_or(""));
        if q.is_empty() {
            return Err(Refusal::SuggestionWithoutQuote { id: tid.clone() }.into());
        }
        if q.split(' ').count() < MIN_QUOTE_WORDS || q.chars().count() < MIN_QUOTE_CHARS {
            return Err(Refusal::QuoteTooShort { id: tid.clone(), quote: q }.into());
        }
        if !fields.iter().any(|f| f.contains(&q)) {
            return Err(Refusal::QuoteNotFound { id: tid.clone(), quote: q }.into());
        }
        quotes.insert(tid.clone(), q);
    }
    let mut merged: BTreeMap<String, TraitValue> = p.traits.clone().unwrap_or_default();
    for (tid, val) in &traits {
        merged.insert(tid.clone(), val.canonical());
    }
    if merged.len() > MAX_PERSONA_TRAITS {
        return Err(Refusal::TraitsTooMany { count: merged.len() }.into());
    }
    let profile: Value = serde_json::from_str(&p.profile_json).unwrap_or(Value::Null);
    let text_of = |k: &str| profile.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let file = PersonaFile {
        id: p.id.clone(),
        name: p.name.clone(),
        tags: p.tags.clone(),
        role: text_of("role"),
        context: text_of("context"),
        daily_tasks: text_of("daily_tasks"),
        fears: text_of("fears"),
        never_reads: text_of("never_reads"),
        decides_by: text_of("decides_by"),
        stake_rules: p.stake_rules.clone(),
        credential: profile.get("credential").and_then(Value::as_str).map(str::to_string),
        traits: Some(merged),
        serves: p.serves.clone(),
    };
    // D39: the sensitive categories the merged persona names (its prose, tags and traits): a label, never a refusal
    let mut texts: Vec<&str> = vec![&file.role, &file.context, &file.daily_tasks, &file.fears, &file.never_reads, &file.decides_by, &file.stake_rules];
    texts.extend(file.credential.as_deref());
    texts.extend(file.tags.iter().map(String::as_str));
    let mut sensitive = crate::domain::traits::sensitive_of(texts);
    sensitive.extend(crate::domain::traits::held_sensitive(file.traits.as_ref().expect("merged traits"), &defs));
    crate::domain::traits::order(&mut sensitive);
    Ok(json!({
        "persona_file": serde_json::to_value(&file).expect("persona file serialises"),
        "evidence": quotes,
        "sensitive": sensitive,
        "from_version": p.version,
    }))
}

/// The shipped mapping table, then the project's rows from `.personajury/observe-map.toml`. A project row adds a rule and
/// overrides nothing: an id the table already holds is refused.
pub fn rules(root: &Path) -> Result<Vec<observed::Rule>> {
    let mut rules = observed::parse_rules(crate::domain::catalog::OBSERVE_MAP)
        .map_err(|e| AppError::InvalidArguments(format!("embedded observe-map.toml: {e}")))?;
    let mine = root.join(".personajury").join("observe-map.toml");
    if mine.is_file() {
        let extra = observed::parse_rules(&super::read_text(&mine)?)
            .map_err(|e| AppError::InvalidArguments(format!("{}: {e}", mine.display())))?;
        for r in extra {
            if rules.iter().any(|x| x.id == r.id) {
                return Err(AppError::InvalidArguments(format!(
                    "{}: rule {} is already in the shipped table; a project row adds a rule and overrides none",
                    mine.display(),
                    r.id
                )));
            }
            rules.push(r);
        }
    }
    Ok(rules)
}

/// One run's accepted initial verdicts of every juror, as `observe` reads them.
fn run_data(store: &Store, id: i64) -> Result<RunData> {
    let run = super::verdict::load_run(store, id)?;
    let lens = super::verdict::load_lens(store, &run)?;
    let rows: Vec<VerdictRow> = store.verdict_rows(id, false)?.into_iter().filter(|r| !super::verdict::is_human(&r.persona)).collect();
    let picks = store
        .picks(id, false)?
        .into_iter()
        .filter(|p| p.about.is_none() && !super::verdict::is_human(&p.persona))
        .map(|p| (p.persona, p.choice))
        .collect();
    Ok(RunData {
        run: id,
        stakes: run.stakes.clone(),
        rows: super::verdict::score_rows(&run, &rows),
        scale_min: lens.scale[0],
        picks,
        subject: run.subject_name.clone(),
    })
}

/// `trait observed --persona ID [--persona ID ...] [--runs A,B,...]` (D35): counts of what the persona did in its accepted
/// initial verdicts, and per declared trait `consistent`, `contradicted`, `mixed` or `no_signal` against the mapping table.
/// No model, nothing written: any change to a persona goes through `persona add`. One persona answers its object; several
/// answer `{"personas": [...]}`.
pub fn observed(root: &Path, personas: &[String], runs: Option<&str>) -> Result<Value> {
    if personas.is_empty() {
        return Err(AppError::InvalidArguments("trait observed needs --persona ID".into()));
    }
    let wanted: Option<Vec<i64>> = match runs {
        None => None,
        Some(s) => Some(
            s.split(',')
                .map(|x| x.trim().parse::<i64>().map_err(|_| AppError::InvalidArguments(format!("--runs {s:?}: run ids separated by commas"))))
                .collect::<Result<Vec<_>>>()?,
        ),
    };
    let store = Store::open_existing(root)?;
    let rules = rules(root)?;
    let defs = store.trait_map()?;
    let all = store.runs()?;
    let mut cache: BTreeMap<i64, RunData> = BTreeMap::new();
    let mut out = Vec::new();
    for pid in personas {
        let p = store.persona(pid)?.ok_or_else(|| AppError::NotFound(format!("no persona {pid}")))?;
        let ids: Vec<i64> = match &wanted {
            Some(w) => {
                for id in w {
                    let run = all.iter().find(|r| r.id == *id).ok_or_else(|| AppError::NotFound(format!("no run {id}")))?;
                    if !run.personas.contains(pid) {
                        return Err(AppError::InvalidArguments(format!("persona {pid} did not sit in run {id}")));
                    }
                }
                w.clone()
            }
            None => all.iter().filter(|r| r.personas.contains(pid)).map(|r| r.id).collect(),
        };
        for id in &ids {
            if !cache.contains_key(id) {
                cache.insert(*id, run_data(&store, *id)?);
            }
        }
        let data: Vec<RunData> = ids.iter().filter_map(|i| cache.get(i)).cloned().collect();
        let signals = observed::observe(pid, &data);
        let declared = p.traits.clone().unwrap_or_default();
        out.push(json!({
            "persona": pid, "runs": signals.runs, "signals": signals.to_json(),
            "traits": observed::verdicts(&declared, &defs, &rules, &signals),
        }));
    }
    Ok(if out.len() == 1 { out.remove(0) } else { json!({"personas": out}) })
}
