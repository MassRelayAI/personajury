//! verdict: ingest the JSON a persona wrote; every refusal of report §5 applies (plan Task 5)
use super::{AppError, Result};
use crate::domain::*;
use crate::store::{Store, DIR};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The store of an existing workspace, or `not_found`.
pub(crate) fn open(root: &Path) -> Result<Store> {
    Ok(Store::open_existing(root)?)
}

pub(crate) fn load_run(store: &Store, id: i64) -> Result<RunRow> {
    store
        .run(id)?
        .ok_or_else(|| AppError::NotFound(format!("no run {id}")))
}

pub(crate) fn load_lens(store: &Store, run: &RunRow) -> Result<Lens> {
    store
        .lens_at(&run.lens_name, run.lens_version)?
        .ok_or_else(|| {
            AppError::NotFound(format!("no lens {} v{}", run.lens_name, run.lens_version))
        })
}

fn subject_of(store: &Store, name: &str, version: i64) -> Result<Subject> {
    store
        .subject_at(name, version)?
        .ok_or_else(|| AppError::NotFound(format!("no subject {name} v{version}")))
}

/// Why a run's subject moved since the brief, if it did (R7, I2): a later subject version whose hash differs is
/// `new subject version <n>`; otherwise the pinned version's sources are rehashed, a missing one is
/// `source_missing: <path>`, a changed one `<label> changed`, the first in label order reported.
pub(crate) fn moved_why(store: &Store, run: &RunRow) -> Result<Option<String>> {
    if let Some(latest) = store.subject(&run.subject_name)? {
        if latest.hash != run.subject_hash {
            return Ok(Some(format!("new subject version {}", latest.version)));
        }
    }
    let s = subject_of(store, &run.subject_name, run.subject_version)?;
    sources_moved(&s)
}

/// Rehash a subject version's sources: why they moved, if they did.
pub(crate) fn sources_moved(s: &Subject) -> Result<Option<String>> {
    for a in &s.attachments {
        match std::fs::read(&a.source_path) {
            Err(_) => return Ok(Some(format!("source_missing: {}", a.source_path))),
            Ok(bytes) if hash::sha256_hex(&bytes) != a.hash => {
                return Ok(Some(format!("{} changed", a.label)))
            }
            Ok(_) => {}
        }
    }
    Ok(None)
}

fn collect_ids(nodes: &Value, out: &mut BTreeSet<String>) {
    if let Some(list) = nodes.as_array() {
        for n in list {
            if let Some(id) = n.get("id").and_then(Value::as_str) {
                out.insert(id.to_string());
            }
            if let Some(ch) = n.get("children") {
                collect_ids(ch, out);
            }
        }
    }
}

/// What each attachment of a subject version offers to citation resolution, read from the blob store.
fn views(root: &Path, s: &Subject) -> Result<BTreeMap<String, AttachmentView>> {
    let mut out = BTreeMap::new();
    for a in &s.attachments {
        let blob = root.join(DIR).join("blobs").join(&a.hash);
        let bytes = std::fs::read(&blob).map_err(|e| AppError::Io(blob.clone(), e))?;
        let mut v = AttachmentView {
            hash: a.hash.clone(),
            media: a.media.clone(),
            ..Default::default()
        };
        match a.media.as_str() {
            "aria-json" => {
                if let Ok(doc) = serde_json::from_slice::<Value>(&bytes) {
                    if let Some(nodes) = doc.get("nodes") {
                        collect_ids(nodes, &mut v.aria_ids);
                    }
                }
            }
            "diff" => v.lines = String::from_utf8_lossy(&bytes).lines().count(),
            // a text file: its line locators are checked; bytes that are not UTF-8 stay unchecked
            "file" => {
                if let Some(t) = std::str::from_utf8(&bytes).ok().filter(|_| !bytes.contains(&0)) {
                    v.lines = t.lines().count();
                    v.text = true;
                }
            }
            _ => {}
        }
        out.insert(a.label.clone(), v);
    }
    Ok(out)
}

/// The findings of verdict rows: one per blocker, deduplicated by id, in row order.
pub(crate) fn findings_of(subject_name: &str, rows: &[VerdictRow]) -> Vec<Finding> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for r in rows {
        for b in &r.blockers {
            let id = hash::finding_id(
                subject_name,
                &r.persona,
                &r.dimension,
                r.about.as_deref(),
                b,
                &r.citation_hash,
            );
            if seen.insert(id.clone()) {
                out.push(Finding {
                    id,
                    persona: r.persona.clone(),
                    dimension: r.dimension.clone(),
                    about: r.about.clone(),
                    text: b.clone(),
                    citation_hash: r.citation_hash.clone(),
                });
            }
        }
    }
    out
}

pub(crate) fn score_rows(run: &RunRow, rows: &[VerdictRow]) -> Vec<ScoreRow> {
    rows.iter()
        .map(|r| ScoreRow {
            persona: r.persona.clone(),
            lens: run.lens_name.clone(),
            dimension: r.dimension.clone(),
            about: r.about.clone(),
            score: r.score,
            anchor_met: r.anchor_met,
            evidence: r.evidence,
            citations: r.citations.clone(),
            blockers: r.blockers.clone(),
        })
        .collect()
}

pub fn ingest(root: &Path, run: i64, input: &str) -> Result<Value> {
    let store = open(root)?;
    let run = load_run(&store, run)?;
    if let Some(why) = moved_why(&store, &run)? {
        return Err(Refusal::SubjectMoved {
            subject: run.subject_name.clone(),
            why,
        }
        .into());
    }
    let lens = load_lens(&store, &run)?;
    let subject = subject_of(&store, &run.subject_name, run.subject_version)?;
    let mut evidence = RunEvidence {
        attachments: views(root, &subject)?,
        comparators: BTreeMap::new(),
    };
    for (name, version) in &run.comparators {
        let c = subject_of(&store, name, *version)?;
        evidence.comparators.insert(name.clone(), views(root, &c)?);
    }
    // what a line may be about and a pick may choose: the subject's labels, then `cmp:<name>` per run comparator
    let mut options: Vec<String> = evidence.attachments.keys().cloned().collect();
    options.sort();
    options.extend(run.comparators.iter().map(|(n, _)| format!("cmp:{n}")));
    let input: VerdictInput = serde_json::from_str(input)
        .map_err(|e| AppError::InvalidArguments(format!("verdict JSON: {e}")))?;
    let cx = VerdictContext {
        lens,
        subject_name: run.subject_name.clone(),
        stakes: run
            .stakes
            .iter()
            .filter(|(p, _)| run.personas.contains(p))
            .map(|(p, s)| (p.clone(), *s))
            .collect(),
        evidence,
        options,
        dismissed: store.dismissed_citations(&run.subject_name, &input.persona)?,
    };
    let valid = refuse::validate(&input, &cx)?;
    let persona_version = run
        .briefs
        .iter()
        .find(|b| b.persona == valid.persona)
        .map(|b| b.persona_version)
        .ok_or_else(|| {
            AppError::InvalidArguments(format!(
                "run {} holds no brief for {}",
                run.id, valid.persona
            ))
        })?;
    let batch = store.next_batch(run.id, &valid.persona)?;
    store.insert_verdict(run.id, batch, persona_version, &run.subject_name, &valid)?;
    let mut ids: Vec<&str> = Vec::new();
    for f in &valid.findings {
        if !ids.contains(&f.id.as_str()) {
            ids.push(&f.id);
        }
    }
    Ok(json!({
        "run": run.id,
        "persona": valid.persona,
        "stake": valid.stake,
        "batch": batch,
        "lines": valid.lines.len(),
        "findings": ids,
    }))
}
