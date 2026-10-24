//! Decision juries (D37): the questions of a subject and the ones that still need a person. A persona jury advises on a
//! question; it never decides one that falls in a human class (public price or words, legal, spending, irreversible or
//! outward), whatever it picked, nor one the jurors split on over money or trust.
use super::{AppError, Result};
use crate::domain::decision::{self, Question};
use crate::domain::*;
use crate::store::{Store, DIR};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The questions of a decision subject, read from its `questions.md` blob.
pub(crate) fn questions_of(root: &Path, s: &Subject) -> Result<Vec<Question>> {
    let att = s
        .attachments
        .iter()
        .find(|a| a.label == decision::QUESTIONS_LABEL)
        .ok_or_else(|| Refusal::InvalidQuestions { why: format!("a decision subject attaches {}", decision::QUESTIONS_LABEL) })?;
    let blob = root.join(DIR).join("blobs").join(&att.hash);
    let bytes = std::fs::read(&blob).map_err(|e| AppError::Io(blob.clone(), e))?;
    parse(&bytes)
}
pub(crate) fn parse(bytes: &[u8]) -> Result<Vec<Question>> {
    let text = std::str::from_utf8(bytes).map_err(|_| Refusal::InvalidQuestions { why: "the file is not UTF-8 text".into() })?;
    Ok(decision::parse_questions(text).map_err(|why| Refusal::InvalidQuestions { why })?)
}

/// The questions of a run that need a person, with why: `[{question, why: [{class, via, ...}]}]`, in question order.
pub(crate) fn needs_person(root: &Path, store: &Store, run: &RunRow, subject: &Subject) -> Result<Vec<Value>> {
    let questions = questions_of(root, subject)?;
    // the point-of-view files that fall under a human rule of the project (a path the jury may not decide)
    let mut source_hits: Vec<Value> = Vec::new();
    if let Ok(t) = std::fs::read_to_string(root.join("personajury.toml")) {
        let cfg = trigger::parse_config(&t)?;
        let paths: Vec<String> = subject
            .attachments
            .iter()
            .map(|x| super::persona::display_path(root, Path::new(&x.source_path)))
            .collect();
        for h in trigger::classify(&cfg, &paths).hits {
            if h.quorum == Quorum::Human || HUMAN_CLASSES.contains(&trigger::normalise_class(&h.class).as_str()) {
                source_hits.push(json!({"class": h.class, "via": "sources", "path": h.paths.first()}));
            }
        }
    }
    // who picked what, per question and stratum, from the jury's first word
    let mut choices: BTreeMap<(String, Stake), BTreeSet<String>> = BTreeMap::new();
    for p in store.picks(run.id, false)? {
        if super::verdict::is_human(&p.persona) {
            continue;
        }
        if let (Some(q), Some(stake)) = (p.about.clone(), run.stakes.get(&p.persona)) {
            choices.entry((q, *stake)).or_default().insert(p.choice);
        }
    }
    let mut out = Vec::new();
    for q in &questions {
        let mut why: Vec<Value> = decision::human_classes(q).into_iter().map(|c| json!({"class": c, "via": "question"})).collect();
        let split = choices.iter().any(|((id, _), set)| *id == q.id && set.len() > 1);
        if split {
            for t in decision::split_topics(q) {
                why.push(json!({"class": format!("split on {t}"), "via": "picks"}));
            }
        }
        why.extend(source_hits.iter().cloned());
        if !why.is_empty() {
            out.push(json!({"question": q.id, "text": q.text, "why": why}));
        }
    }
    Ok(out)
}
