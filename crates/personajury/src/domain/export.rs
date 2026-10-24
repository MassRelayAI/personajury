//! export (D40): raw verdict rows, one per verdict line, as ndjson or RFC 4180 csv. Pure: the rows come in, text goes out.
//! Never a pooled or derived number: every column is a stored fact or a count of one (`blockers`).
use super::traits::TraitValue;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The most rows the MCP tool answers; past it the answer says `truncated` (the CLI has no cap).
pub const MCP_ROW_CAP: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRow {
    pub run: i64,
    pub subject: String,
    pub subject_version: i64,
    pub lens: String,
    pub lens_version: i64,
    pub persona: String,
    pub stratum: super::Stake,
    /// the model the brief was for; none for a person's row
    pub model: Option<String>,
    pub dimension: String,
    pub about: Option<String>,
    pub score: i64,
    pub anchor_met: i64,
    pub evidence: String,
    pub citations: Vec<String>,
    /// how many blockers the line raised, not their text
    pub blockers: usize,
    /// `initial` or `deliberated`
    pub phase: &'static str,
    pub by_human: Option<String>,
    /// the traits the persona held in the briefed version (a person's: the ones they declared)
    pub traits: BTreeMap<String, TraitValue>,
    pub sensitive: Vec<String>,
}

/// run, stratum, persona, lens, dimension, about; then phase, so a cell's initial row comes before its deliberated one.
pub fn sort(rows: &mut [ExportRow]) {
    let phase = |r: &ExportRow| r.phase != "initial";
    rows.sort_by(|a, b| {
        (a.run, a.stratum, &a.persona, &a.lens, &a.dimension, &a.about, phase(a))
            .cmp(&(b.run, b.stratum, &b.persona, &b.lens, &b.dimension, &b.about, phase(b)))
    });
}

const FIXED: [&str; 17] = [
    "run", "subject", "subject_version", "lens", "lens_version", "persona", "stratum", "model", "dimension", "about", "score",
    "anchor_met", "evidence", "citations", "blockers", "phase", "by_human",
];

fn trait_json(v: &TraitValue) -> Value {
    match v.canonical() {
        TraitValue::Int(i) => json!(i),
        TraitValue::Str(s) => json!(s),
        TraitValue::List(l) => json!(l),
    }
}
fn trait_cell(v: &TraitValue) -> String {
    match v.canonical() {
        TraitValue::Int(i) => i.to_string(),
        TraitValue::Str(s) => s,
        TraitValue::List(l) => l.join(";"),
    }
}

/// One JSON object per line, keys in a fixed order: the columns, `trait.<id>` for the traits the persona holds (sorted),
/// then `sensitive`. A trait the persona does not hold has no key.
pub fn ndjson(rows: &[ExportRow]) -> String {
    let mut out = String::new();
    for r in rows {
        let mut m = serde_json::Map::new();
        m.insert("run".into(), json!(r.run));
        m.insert("subject".into(), json!(r.subject));
        m.insert("subject_version".into(), json!(r.subject_version));
        m.insert("lens".into(), json!(r.lens));
        m.insert("lens_version".into(), json!(r.lens_version));
        m.insert("persona".into(), json!(r.persona));
        m.insert("stratum".into(), json!(r.stratum.name()));
        m.insert("model".into(), json!(r.model));
        m.insert("dimension".into(), json!(r.dimension));
        m.insert("about".into(), json!(r.about));
        m.insert("score".into(), json!(r.score));
        m.insert("anchor_met".into(), json!(r.anchor_met));
        m.insert("evidence".into(), json!(r.evidence));
        m.insert("citations".into(), json!(r.citations));
        m.insert("blockers".into(), json!(r.blockers));
        m.insert("phase".into(), json!(r.phase));
        m.insert("by_human".into(), json!(r.by_human));
        for (id, v) in &r.traits {
            m.insert(format!("trait.{id}"), trait_json(v));
        }
        m.insert("sensitive".into(), json!(r.sensitive));
        out.push_str(&Value::Object(m).to_string());
        out.push('\n');
    }
    out
}

/// A csv field: quoted when it holds a comma, a quote, a CR or an LF, with quotes doubled (RFC 4180).
pub fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// RFC 4180: a header, then one record per row, every record ended by CRLF. The `trait.<id>` columns are the union of the
/// rows' trait ids, sorted; a cell is empty where the persona holds none. Lists and citations are joined with `;`.
pub fn csv(rows: &[ExportRow]) -> String {
    let ids: std::collections::BTreeSet<&String> = rows.iter().flat_map(|r| r.traits.keys()).collect();
    let mut header: Vec<String> = FIXED.iter().map(|s| s.to_string()).collect();
    header.extend(ids.iter().map(|i| format!("trait.{i}")));
    header.push("sensitive".into());
    let mut out = header.iter().map(|h| csv_field(h)).collect::<Vec<_>>().join(",");
    out.push_str("\r\n");
    for r in rows {
        let mut cells: Vec<String> = vec![
            r.run.to_string(),
            r.subject.clone(),
            r.subject_version.to_string(),
            r.lens.clone(),
            r.lens_version.to_string(),
            r.persona.clone(),
            r.stratum.name().into(),
            r.model.clone().unwrap_or_default(),
            r.dimension.clone(),
            r.about.clone().unwrap_or_default(),
            r.score.to_string(),
            r.anchor_met.to_string(),
            r.evidence.clone(),
            r.citations.join(";"),
            r.blockers.to_string(),
            r.phase.into(),
            r.by_human.clone().unwrap_or_default(),
        ];
        cells.extend(ids.iter().map(|i| r.traits.get(*i).map(trait_cell).unwrap_or_default()));
        cells.push(r.sensitive.join(";"));
        out.push_str(&cells.iter().map(|c| csv_field(c)).collect::<Vec<_>>().join(","));
        out.push_str("\r\n");
    }
    out
}
