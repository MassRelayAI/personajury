//! Decision juries (D37): the questions of a subject and the ones that still need a person. A persona jury advises on a
//! question; it never decides one declared in a human class or whose text hits the stem table, whatever it picked, nor
//! one whose strata's top picks differ on money, spending, trust, legal or public claims.
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
    Ok(decision::parse_questions(text).map_err(|e| match e {
        decision::QuestionsError::ClassMissing(why) => Refusal::QuestionClassMissing { why },
        decision::QuestionsError::Invalid(why) => Refusal::InvalidQuestions { why },
    })?)
}

/// The point-of-view files of a decision subject (every attachment but the questions), scanned with the stem table (I1):
/// `[{path, stems: [{group, stem}]}]`, in attachment order. **Informational**: a hit is listed for the person who reads
/// the summary and never changes an answer; what decides is the questions' declared class, their own text and the
/// project's path rules.
pub(crate) fn sources_mention(root: &Path, subject: &Subject) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    for a in subject.attachments.iter().filter(|a| a.label != decision::QUESTIONS_LABEL) {
        let blob = root.join(DIR).join("blobs").join(&a.hash);
        let bytes = std::fs::read(&blob).map_err(|e| AppError::Io(blob.clone(), e))?;
        let Ok(text) = std::str::from_utf8(&bytes) else { continue };
        let hits = decision::stem_matches(text);
        if !hits.is_empty() {
            out.push(json!({"path": a.label, "stems": hits.iter().map(|(g, s)| json!({"group": g, "stem": s})).collect::<Vec<_>>()}));
        }
    }
    Ok(out)
}

/// The questions of a run that a jury does not settle, with why: `[{question, text, why: [{code, effect, class, via, ...}]}]`,
/// in question order. `effect` is `human` (declared human class, `class_mismatch`, `split_on_sensitive`, a source path
/// under a human rule) or `advise` (`split_on_question`: the strata's top picks differ on a question of a jury class).
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
                source_hits.push(json!({"code": "human_class", "effect": "human", "class": h.class, "via": "sources", "path": h.paths.first()}));
            }
        }
    }
    // who picked what, per question and stratum, from the jury's frozen first word
    let mut counts: BTreeMap<(String, Stake), BTreeMap<String, usize>> = BTreeMap::new();
    for p in store.picks(run.id, false)? {
        if super::verdict::is_human(&p.persona) {
            continue;
        }
        if let (Some(q), Some(stake)) = (p.about.clone(), run.stakes.get(&p.persona)) {
            *counts.entry((q, *stake)).or_default().entry(p.choice).or_default() += 1;
        }
    }
    let mut out = Vec::new();
    for q in &questions {
        let mut why: Vec<Value> = Vec::new();
        if decision::is_human_class(&q.class) {
            why.push(json!({"code": "human_class", "effect": "human", "class": q.class, "via": "declared"}));
        } else {
            for (group, stem) in decision::question_stems(q) {
                why.push(json!({"code": "class_mismatch", "effect": "human", "class": q.class, "group": group, "stem": stem, "via": "question"}));
            }
        }
        // C4: a split is the strata's top picks differing; a stratum tied at the top has no single top pick and counts
        let tops: BTreeSet<&String> = counts
            .iter()
            .filter(|((id, _), _)| *id == q.id)
            .flat_map(|(_, c)| {
                let max = c.values().copied().max().unwrap_or(0);
                c.iter().filter(move |(_, n)| **n == max).map(|(k, _)| k)
            })
            .collect();
        if tops.len() > 1 {
            let sensitive = decision::sensitive_classes(q);
            if sensitive.is_empty() {
                why.push(json!({"code": "split_on_question", "effect": "advise", "class": q.class, "via": "picks"}));
            }
            for c in sensitive {
                why.push(json!({"code": "split_on_sensitive", "effect": "human", "class": c, "via": "picks"}));
            }
        }
        why.extend(source_hits.iter().cloned());
        if !why.is_empty() {
            out.push(json!({"question": q.id, "text": q.text, "class": q.class, "why": why}));
        }
    }
    Ok(out)
}
