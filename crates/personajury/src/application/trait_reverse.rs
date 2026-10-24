//! The reverse generators (D35): `trait suggest` turns a persona's prose into a brief and checks the agent's answer,
//! `trait observed` counts what a persona did in its verdicts. Neither calls a model and neither writes a trait back.
use super::{AppError, Result};
use crate::domain::hash::sha256_hex;
use crate::domain::traits::{check_map_shape, TraitDef, TraitValue, MAX_PERSONA_TRAITS};
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
evidence. Leave out every trait the prose does not support. At most 8. A band takes an integer from 1 to 5 (2 and 4 lean toward \
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
    let v: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::InvalidArguments(format!("{} is not a suggestion: {e}", file.display())))?;
    let bad = |why: &str| AppError::InvalidArguments(format!("{} is not a suggestion: {why}", file.display()));
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
    check_map_shape(&traits)?;
    let defs = store.trait_map()?;
    for (tid, val) in &traits {
        let def = defs.get(tid).ok_or_else(|| Refusal::UnknownTrait { id: tid.clone() })?;
        def.check_value(val)?;
    }
    let prose = fold(&prose_of(&p).iter().map(|(_, t)| fold(t)).collect::<Vec<_>>().join(" "));
    let mut quotes = BTreeMap::new();
    for tid in traits.keys() {
        let q = fold(evidence.get(tid).map(String::as_str).unwrap_or(""));
        if q.is_empty() {
            return Err(Refusal::SuggestionWithoutQuote { id: tid.clone() }.into());
        }
        if !prose.contains(&q) {
            return Err(Refusal::QuoteNotInProse { id: tid.clone(), quote: q }.into());
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
    Ok(json!({
        "persona_file": serde_json::to_value(&file).expect("persona file serialises"),
        "evidence": quotes,
        "from_version": p.version,
    }))
}
