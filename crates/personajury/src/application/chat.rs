//! The chat pack (D41): `brief --chat-pack F` writes one markdown file a web chat can answer, and
//! `verdict --run N --from-chat F` ingests the saved answer, one juror at a time, through the normal checks.
use super::{AppError, Result};
use crate::domain::chat;
use serde_json::{json, Value};
use std::path::Path;

/// The one run a pack is for: a brief answer holds one run, or a `runs` list (a rule) that must hold exactly one.
fn the_run(out: &Value) -> Result<&Value> {
    match out.get("runs").and_then(Value::as_array) {
        None => Ok(out),
        Some(r) if r.len() == 1 => Ok(&r[0]),
        Some(r) => {
            let ids: Vec<String> = r.iter().map(|x| x["run"].to_string()).collect();
            Err(AppError::InvalidArguments(format!(
                "--chat-pack takes one lens: this rule briefed runs {}; no pack was written, brief again with --lens <one lens>",
                ids.join(", ")
            )))
        }
    }
}

/// Write the pack for a brief's answer to `path`: an instruction block, then every brief in the answer's order.
pub fn write_pack(root: &Path, out: &Value, path: &Path) -> Result<()> {
    let run = the_run(out)?;
    let id = run["run"].as_i64().unwrap_or(0);
    let briefs = run["briefs"].as_array().cloned().unwrap_or_default();
    let ids: Vec<&str> = briefs.iter().filter_map(|b| b["persona"].as_str()).collect();
    let mut t = String::new();
    t.push_str(&format!(
        "# PersonaJury chat pack\n\nRun {id}: {} under the {} lens. {} jurors, in this order: {}.\n\n",
        run["subject"]["name"].as_str().unwrap_or(""),
        run["lens"]["name"].as_str().unwrap_or(""),
        ids.len(),
        ids.join(", ")
    ));
    t.push_str("## How to answer\n\n");
    t.push_str(
        "Each brief below is one juror's whole instructions. You play every juror, one after another, and each must be answered \
separately: judge as that person only, and do not let one juror's view shape the next.\n\n\
- For each juror write a heading line `### <persona-id>` (the id is in the brief's first lines) and directly under it ONE fenced \
```json block holding that juror's verdict, in the shape its brief gives. Nothing outside that block is read.\n\
- Write one heading and one block per juror; do not merge jurors into one block.\n",
    );
    t.push_str(&format!(
        "- Save your whole answer as a file and give it to `personajury verdict --run {id} --from-chat <file>`: each block is checked \
like any verdict, and a refused juror is named so you can ask that one again.\n\n---\n\n"
    ));
    for (i, b) in briefs.iter().enumerate() {
        let rel = b["path"].as_str().unwrap_or("");
        let p = root.join(rel);
        let text = std::fs::read_to_string(&p).map_err(|e| AppError::Io(p.clone(), e))?;
        t.push_str(&format!("## Brief {} of {}: {}\n\n{}\n---\n\n", i + 1, briefs.len(), b["persona"].as_str().unwrap_or(""), text.trim_end()));
    }
    std::fs::write(path, t.as_bytes()).map_err(|e| AppError::Io(path.to_path_buf(), e))
}

fn refused(persona: &str, code: &str, message: String, data: Option<Value>) -> Value {
    let mut v = json!({"persona": persona, "code": code, "message": message});
    if let Some(d) = data {
        v["data"] = d;
    }
    v
}

/// Ingest a saved chat answer for `run`. The jurors are taken in the run's order; a block that is malformed, under the wrong
/// heading or refused by the normal checks is reported and the others still land. All in: `ok`; else `chat_answer_incomplete`
/// with the same lists as data.
pub fn ingest(root: &Path, run: i64, file: &Path) -> Result<Value> {
    let text = super::read_text(file)?;
    let store = super::verdict::open(root)?;
    let r = super::verdict::load_run(&store, run)?;
    let blocks = chat::parse(&text, &r.personas);
    if blocks.is_empty() {
        return Err(AppError::InvalidArguments(format!(
            "no juror found in {}: expected `### <persona-id>` followed by a ```json block, for {}",
            file.display(),
            r.personas.join(", ")
        )));
    }
    let (mut ingested, mut refusals, mut missing) = (Vec::new(), Vec::new(), Vec::new());
    for p in &r.personas {
        let Some(block) = blocks.iter().find(|b| &b.persona == p) else {
            missing.push(p.clone());
            continue;
        };
        let body = match &block.body {
            Ok(b) => b,
            Err(why) => {
                refusals.push(refused(p, "chat_block_malformed", why.clone(), None));
                continue;
            }
        };
        let mut v: Value = match serde_json::from_str(body) {
            Ok(v) => v,
            Err(e) => {
                refusals.push(refused(p, "chat_block_malformed", format!("the json block is not valid json: {e}"), None));
                continue;
            }
        };
        match v.get("persona") {
            None => {
                if let Some(o) = v.as_object_mut() {
                    o.insert("persona".into(), json!(p));
                }
            }
            Some(Value::String(s)) if s == p => {}
            Some(other) => {
                refusals.push(refused(p, "chat_block_persona_mismatch", format!("the block under `### {p}` says persona {other}"), None));
                continue;
            }
        }
        match super::verdict::ingest(root, run, &v.to_string()) {
            Ok(d) => ingested.push(d),
            Err(e) => refusals.push(refused(p, e.code(), e.to_string(), e.data())),
        }
    }
    let data = json!({"run": run, "ingested": ingested, "refused": refusals, "missing": missing});
    if refusals.is_empty() && missing.is_empty() {
        return Ok(data);
    }
    let names = |v: &[Value]| v.iter().filter_map(|x| x["persona"].as_str()).collect::<Vec<_>>().join(", ");
    Err(AppError::ChatIncomplete(
        format!(
            "{} of {} jurors ingested; refused: [{}]; missing: [{}]",
            ingested.len(),
            r.personas.len(),
            names(&refusals),
            missing.join(", ")
        ),
        data,
    ))
}
