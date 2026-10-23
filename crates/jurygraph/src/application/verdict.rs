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

/// What the agent reported beside what the project estimates, for these runs (all runs when empty), D12. The engine
/// measures no token: `reported` is whatever verdicts carried, labelled so; `estimate` is briefs x `tokens_per_brief`.
pub(crate) fn usage_summary(root: &Path, store: &Store, run_ids: &[i64]) -> Result<Value> {
    let per = tokens_per_brief(root)?;
    usage_summary_with(store, run_ids, per)
}

/// The project's tokens per brief: the toml's, the default when there is no toml, a refusal when it is malformed.
pub(crate) fn tokens_per_brief(root: &Path) -> Result<u64> {
    match std::fs::read_to_string(root.join("jurygraph.toml")) {
        Ok(t) => Ok(crate::domain::trigger::parse_config(&t)?
            .tokens_per_brief
            .unwrap_or(super::brief::DEFAULT_TOKENS_PER_BRIEF)),
        Err(_) => Ok(super::brief::DEFAULT_TOKENS_PER_BRIEF),
    }
}

pub(crate) fn usage_summary_with(store: &Store, run_ids: &[i64], per: u64) -> Result<Value> {
    let runs = store.runs()?;
    let briefs: u64 = runs
        .iter()
        .filter(|r| run_ids.is_empty() || run_ids.contains(&r.id))
        .map(|r| r.briefs.len() as u64)
        .sum();
    let reported = match store.reported_usage(run_ids)? {
        Some((i, o, n)) => json!({
            "input_tokens": i, "output_tokens": o, "verdicts": n, "by": "reported by the agent",
        }),
        None => Value::Null,
    };
    Ok(json!({
        "reported": reported,
        "estimate": {"briefs": briefs, "tokens": briefs.saturating_mul(per)},
    }))
}

/// The one line `matrix --table` adds when verdicts reported usage; none otherwise (the recorded table is unchanged).
pub(crate) fn usage_line(u: &Value) -> Option<String> {
    let r = u["reported"].as_object()?;
    Some(format!(
        "usage (reported by the agent, over {} verdicts): {} input, {} output tokens; the project's estimate is {} tokens for {} briefs\n",
        r["verdicts"], r["input_tokens"], r["output_tokens"], u["estimate"]["tokens"], u["estimate"]["briefs"]
    ))
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
    ingest_by(root, run, input, None)
}

/// The id a human's rows are stored under: `human:<slug of the name>:<seat>` (D19). The seat is the persona of the
/// run whose stratum the person speaks in; two people on one seat are two ids.
pub(crate) fn human_id(by_human: &str, seat: &str) -> String {
    let name = by_human.split(',').next().unwrap_or("").trim().to_lowercase();
    let mut slug = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    format!("human:{}:{seat}", slug.trim_matches('-'))
}
pub(crate) fn is_human(persona: &str) -> bool {
    persona.starts_with("human:")
}

/// Ingest a verdict. With `by_human` ("<name, credential>") the batch is a person's (D19): the same refusals, stored
/// under `human:<name>:<seat>` beside the jury's rows, never merged with a persona's cell.
pub fn ingest_by(root: &Path, run: i64, input: &str, by_human: Option<&str>) -> Result<Value> {
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
    let mut input: VerdictInput = serde_json::from_str(input)
        .map_err(|e| AppError::InvalidArguments(format!("verdict JSON: {e}")))?;
    let mut stakes: BTreeMap<String, Stake> = run
        .stakes
        .iter()
        .filter(|(p, _)| run.personas.contains(p))
        .map(|(p, s)| (p.clone(), *s))
        .collect();
    // the seat a person speaks from; the persona id the rows are stored under
    let seat = input.persona.clone();
    if let Some(by) = by_human {
        let by = by.trim();
        if by.is_empty() || by.chars().count() > CREDENTIAL_MAX {
            return Err(AppError::InvalidArguments(
                "--by-human names who entered the verdict: \"<name>, <credential>\", at most 500 characters".into(),
            ));
        }
        if human_id(by, &seat).starts_with("human::") {
            return Err(AppError::InvalidArguments("--by-human needs a name before the comma".into()));
        }
        if input.usage.is_some() {
            return Err(AppError::InvalidArguments("a human verdict carries no usage".into()));
        }
        // the seat must be a persona of the run: it gives the person's stratum
        let stake = *stakes
            .get(&seat)
            .ok_or_else(|| Refusal::PersonaWithoutStake { persona: seat.clone() })?;
        input.persona = human_id(by, &seat);
        stakes.insert(input.persona.clone(), stake);
    }
    let cx = VerdictContext {
        lens,
        subject_name: run.subject_name.clone(),
        stakes,
        evidence,
        options,
        dismissed: store.dismissed_citations(&run.subject_name, &input.persona)?,
    };
    let valid = refuse::validate(&input, &cx)?;
    let persona_version = run
        .briefs
        .iter()
        .find(|b| b.persona == seat)
        .map(|b| b.persona_version)
        .ok_or_else(|| {
            AppError::InvalidArguments(format!(
                "run {} holds no brief for {}",
                run.id, valid.persona
            ))
        })?;
    let batch = store.next_batch(run.id, &valid.persona)?;
    store.insert_verdict_by(run.id, batch, persona_version, &run.subject_name, &valid, by_human.map(str::trim))?;
    let mut ids: Vec<&str> = Vec::new();
    for f in &valid.findings {
        if !ids.contains(&f.id.as_str()) {
            ids.push(&f.id);
        }
    }
    let mut out = json!({
        "run": run.id,
        "persona": valid.persona,
        "stake": valid.stake,
        "batch": batch,
        "lines": valid.lines.len(),
        "findings": ids,
    });
    if let Some(by) = by_human {
        out["by_human"] = json!(by.trim());
        out["seat"] = json!(seat);
    }
    Ok(out)
}
