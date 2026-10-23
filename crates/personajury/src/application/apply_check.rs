//! apply-check: the verdict the agent obeys last (D13). `apply | escalate | human | advise`, with the reasons as data.
//! It computes from the store and the project's rules and applies nothing itself. `apply` only when every condition is
//! proven; anything unknown, missing or lowered answers `advise` (or worse), never `apply`.
use super::brief::quorum_word as quorum_name;
use super::verdict::{load_run, open, score_rows};
use super::{AppError, Result};
use crate::domain::apply::{self, Answer};
use crate::domain::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

/// A reason in the answer: its code, what it does to the answer (`human`, `escalate`, `advise`, `info`) and its data.
fn reason(code: &str, effect: &str, mut data: Value) -> Value {
    data["code"] = json!(code);
    data["effect"] = json!(effect);
    data
}

/// A jury quorum (`low` or `high`) from its word; anything else is no jury quorum.
fn jury_quorum(q: Option<&str>) -> Option<Quorum> {
    match q {
        Some("low") => Some(Quorum::Low),
        Some("high") => Some(Quorum::High),
        _ => None,
    }
}
fn jury(q: Quorum) -> Option<Quorum> {
    matches!(q, Quorum::Low | Quorum::High).then_some(q)
}

/// The fields of a run's recorded override that lower the jury below its rule (C4); raising them is fine. A panel,
/// query, stake or draw that replaced the rule's is not proven equal, so it counts as lowered.
fn lowered_fields(ovr: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    let Some(fields) = ovr["fields"].as_object() else { return out };
    for (k, f) in fields {
        let lowered = match k.as_str() {
            // a rule's `spend = 0` is the whole panel: any recorded spend is then fewer seats
            "spend" => match (f["rule"].as_i64(), f["given"].as_i64()) {
                (Some(0), _) => true,
                (Some(r), Some(g)) => g < r,
                _ => true,
            },
            "lenses" => {
                let given: Vec<&str> = f["given"].as_array().map_or(vec![], |a| a.iter().filter_map(Value::as_str).collect());
                f["rule"].as_array().is_none_or(|a| a.iter().filter_map(Value::as_str).any(|l| !given.contains(&l)))
            }
            "detail" => f["rule"] == "full" && f["given"] != "full",
            "panel" | "stakes" | "draw" => true,
            // `seats` above the ceiling raises; `quorum` is `rule_below_sources`
            _ => false,
        };
        if lowered {
            let mut v = f.clone();
            v["field"] = json!(k);
            out.push(v);
        }
    }
    out
}
fn is_human_class(class: &str, quorum: Quorum) -> bool {
    quorum == Quorum::Human || HUMAN_CLASSES.contains(&trigger::normalise_class(class).as_str())
}

/// The runs to judge: the ones named, or (`--latest`) every run of each subject that has runs (`--subject` narrows it to
/// one). Each group is judged on its own; `latest` answers the worst of them.
pub fn runs_to_judge(root: &Path, named: &[i64], latest: bool, subject: Option<&str>) -> Result<Vec<Vec<i64>>> {
    match (named.is_empty(), latest) {
        (false, true) => Err(AppError::InvalidArguments("name runs or pass --latest, not both".into())),
        (true, false) => Err(AppError::InvalidArguments("name at least one run, or pass --latest".into())),
        (false, false) if subject.is_some() => {
            Err(AppError::InvalidArguments("--subject narrows --latest; named runs already name their subject".into()))
        }
        (false, false) => Ok(vec![named.to_vec()]),
        (true, true) => {
            let all = open(root)?.runs()?;
            if all.is_empty() {
                return Err(AppError::NotFound("the store holds no run".into()));
            }
            let mut by_subject: std::collections::BTreeMap<&str, Vec<i64>> = std::collections::BTreeMap::new();
            for r in &all {
                if subject.is_none_or(|s| s == r.subject_name) {
                    by_subject.entry(r.subject_name.as_str()).or_default().push(r.id);
                }
            }
            if by_subject.is_empty() {
                return Err(AppError::NotFound(format!("no run of subject {}", subject.unwrap_or(""))));
            }
            Ok(by_subject.into_values().collect())
        }
    }
}

/// The worst first: `human` beats `escalate` beats `advise` beats `apply` (an unknown word counts as worst).
fn severity(answer: &Value) -> u8 {
    match answer.as_str() {
        Some("apply") => 0,
        Some("advise") => 1,
        Some("escalate") => 2,
        _ => 3,
    }
}

/// `apply-check --latest`: judge every subject that has runs, each on its own, and answer the worst. One group (one
/// subject, or `--subject`) answers exactly as naming its runs would, plus `subjects`.
pub fn worst(root: &Path, groups: &[Vec<i64>], rule: Option<&str>, calibration: Option<&Path>) -> Result<Value> {
    let mut answers = Vec::new();
    for g in groups {
        answers.push(run(root, g, rule, calibration)?);
    }
    let w = answers.iter().max_by_key(|a| severity(&a["answer"])).cloned().ok_or_else(|| AppError::NotFound("the store holds no run".into()))?;
    if answers.len() == 1 {
        let mut out = w;
        out["subjects"] = json!([out.clone()]);
        return Ok(out);
    }
    let mut runs: Vec<Value> = Vec::new();
    let mut reasons: Vec<Value> = Vec::new();
    for a in &answers {
        runs.extend(a["runs"].as_array().cloned().unwrap_or_default());
        for r in a["reasons"].as_array().into_iter().flatten() {
            let mut r = r.clone();
            r["subject"] = a["subject"].clone();
            reasons.push(r);
        }
    }
    let mut out = json!({
        "answer": w["answer"],
        "worst": w["subject"],
        "runs": runs,
        "reasons": reasons,
        "subjects": answers,
    });
    if !w["next_quorum"].is_null() {
        out["next_quorum"] = w["next_quorum"].clone();
    }
    Ok(out)
}

pub fn run(root: &Path, runs: &[i64], rule: Option<&str>, calibration: Option<&Path>) -> Result<Value> {
    if runs.is_empty() {
        return Err(AppError::InvalidArguments("name at least one run".into()));
    }
    let store = open(root)?;
    let mut given = Vec::new();
    for id in runs {
        let r = load_run(&store, *id)?;
        if let Some(first) = given.first().filter(|f: &&RunRow| f.subject_name != r.subject_name) {
            return Err(AppError::InvalidArguments(format!(
                "apply-check is over one subject: run {} is on {}, run {id} is on {}",
                first.id, first.subject_name, r.subject_name
            )));
        }
        if !given.iter().any(|g| g.id == r.id) {
            given.push(r);
        }
    }
    // the judged state is the latest given run of each lens; earlier given runs are the rounds before it
    let mut judged: Vec<RunRow> = Vec::new();
    for r in &given {
        match judged.iter_mut().find(|j| j.lens_name == r.lens_name) {
            Some(j) if j.id < r.id => *j = r.clone(),
            Some(_) => {}
            None => judged.push(r.clone()),
        }
    }
    judged.sort_by_key(|r| r.id);
    let subject = store
        .subject_at(&judged[0].subject_name, judged[0].subject_version)?
        .ok_or_else(|| AppError::NotFound(format!("no subject {}", judged[0].subject_name)))?;

    // ---- the quorum: the highest of the runs' recorded quorum, the caller's rule and the sources' (C2, C3) ----
    let cfg_path = root.join("personajury.toml");
    let cfg: Option<TriggerConfig> = match std::fs::read_to_string(&cfg_path) {
        Ok(t) => Some(trigger::parse_config(&t)?),
        Err(_) => None,
    };
    let mut reasons: Vec<Value> = Vec::new();
    let mut caller_q: Option<Quorum> = None;
    if let Some(class) = rule {
        let cfg = cfg
            .as_ref()
            .ok_or_else(|| AppError::NotFound(format!("no rule {class}: {} does not exist", cfg_path.display())))?;
        let r = cfg
            .rules
            .iter()
            .find(|r| r.class == class)
            .ok_or_else(|| AppError::NotFound(format!("no rule {class} in {}", cfg_path.display())))?;
        if is_human_class(&r.class, r.quorum) {
            reasons.push(reason("human_class", "human", json!({"class": r.class, "via": "rule"})));
        }
        caller_q = jury(r.quorum);
    }
    let mut sources_q: Option<Quorum> = None;
    let mut sources_rule: Option<String> = None;
    if let Some(cfg) = &cfg {
        let paths: Vec<String> = subject
            .attachments
            .iter()
            .map(|x| super::persona::display_path(root, Path::new(&x.source_path)))
            .collect();
        let c = trigger::classify(cfg, &paths);
        for hit in &c.hits {
            if is_human_class(&hit.class, hit.quorum)
                && !reasons.iter().any(|r| r["code"] == "human_class" && r["class"] == json!(hit.class))
            {
                reasons.push(reason(
                    "human_class",
                    "human",
                    json!({"class": hit.class, "via": "sources", "path": hit.paths.first()}),
                ));
            }
            if let Some(q) = jury(hit.quorum).filter(|q| Some(*q) > sources_q) {
                sources_q = Some(q);
                sources_rule = Some(hit.class.clone());
            }
        }
    }
    let run_q: Option<Quorum> = judged.iter().filter_map(|r| jury_quorum(r.quorum.as_deref())).max();
    let effective = run_q.max(caller_q).max(sources_q);
    let quorum: Option<&'static str> = effective.map(quorum_name);
    let quorum_source = match effective {
        None => "unknown",
        q if q == sources_q => "sources",
        q if q == run_q => "run",
        _ => "rule",
    };
    if effective.is_none() {
        reasons.push(reason(
            "quorum_unknown",
            "advise",
            json!({"remedy": "brief under a rule (`brief --rule <class>`) or add a rule whose paths hold the subject's sources"}),
        ));
    }
    // a rule below the quorum the sources fall under lowers the jury: it advises, never applies (C2)
    if let Some(sq) = sources_q {
        if let (Some(class), Some(cq)) = (rule, caller_q.filter(|cq| *cq < sq)) {
            reasons.push(reason(
                "rule_below_sources",
                "advise",
                json!({"via": "rule", "rule": class, "quorum": quorum_name(cq), "sources": quorum_name(sq), "sources_rule": sources_rule}),
            ));
        }
        for r in &judged {
            let Some(class) = r.rule.as_deref() else { continue };
            let rq = cfg.as_ref().and_then(|c| c.rules.iter().find(|x| x.class == class)).map(|x| x.quorum);
            let recorded = r.override_json.as_ref().is_some_and(|o| !o["fields"]["quorum"].is_null());
            if recorded || rq.is_none_or(|q| q < sq) {
                reasons.push(reason(
                    "rule_below_sources",
                    "advise",
                    json!({"via": "run", "run": r.id, "rule": class, "quorum": rq.map(quorum_name), "sources": quorum_name(sq), "sources_rule": sources_rule}),
                ));
            }
        }
    }

    // D20: the jury does not rank like the people it was set beside: it advises, it never applies
    if let Some(p) = calibration {
        let text = super::read_text(p)?;
        let c: Value = serde_json::from_str(&text)
            .map_err(|e| AppError::InvalidArguments(format!("{}: not a calibration file ({e})", p.display())))?;
        let Some(ok) = c["calibrated"].as_bool() else {
            return Err(AppError::InvalidArguments(format!("{}: not a calibration file (no `calibrated`)", p.display())));
        };
        // the file must be the calibration of this subject, lens and lens version, from one of the runs given (M1)
        let jury_run = c["jury_run"].as_i64().and_then(|id| given.iter().find(|r| r.id == id));
        let mismatch = match jury_run {
            None => Some(format!("its jury_run {} is not one of the runs given", c["jury_run"])),
            Some(r) if c["subject"].as_str() != Some(subject.name.as_str()) => {
                Some(format!("calibrated on subject {}, not on {}", c["subject"], r.subject_name))
            }
            Some(r) if c["lens"].as_str() != Some(r.lens_name.as_str()) => {
                Some(format!("calibrated on lens {}, run {} is {}", c["lens"], r.id, r.lens_name))
            }
            Some(r) if c["lens_version"].as_i64() != Some(r.lens_version) => {
                Some(format!("calibrated on lens version {}, run {} is v{}", c["lens_version"], r.id, r.lens_version))
            }
            Some(_) => None,
        };
        if let Some(why) = mismatch {
            return Err(AppError::CalibrationMismatch(format!(
                "{}: {why}; calibrate this run (`calibrate <jury-run> <human-run> --out F`)",
                p.display()
            )));
        }
        if !ok {
            reasons.push(reason(
                "uncalibrated",
                "advise",
                json!({"calibration": p.display().to_string(), "outside_primary": c["outside_primary"],
                       "remedy": "the jury ranks differently from the people it was compared with on the primary stratum; ask a person, or calibrate again"}),
            ));
        }
    }

    // ---- every briefed seat of every given run has an accepted verdict (C1) ----
    let mut missing: Vec<Value> = Vec::new();
    for r in &given {
        let answered: BTreeSet<String> = store
            .verdict_rows(r.id, false)?
            .into_iter()
            .filter(|x| !super::verdict::is_human(&x.persona))
            .map(|x| x.persona)
            .collect();
        let seats: Vec<&String> = r.personas.iter().filter(|p| !answered.contains(*p)).collect();
        if !seats.is_empty() {
            missing.push(json!({"run": r.id, "seats": seats}));
        }
    }
    if !missing.is_empty() {
        reasons.push(reason(
            "incomplete_jury",
            "advise",
            json!({"missing": missing, "remedy": "ingest a verdict for every briefed seat, then ask again"}),
        ));
    }

    // ---- the verdicts of the judged runs, each with its own stakes (C5) ----
    let mut primary_runs: Vec<Value> = Vec::new();
    let mut primary_personas: BTreeSet<String> = BTreeSet::new();
    let mut primary_count = 0usize;
    let mut splits: Vec<Value> = Vec::new();
    for r in &judged {
        let rows = score_rows(r, &store.verdict_rows(r.id, false)?);
        let blocked: Vec<&ScoreRow> = rows
            .iter()
            .filter(|x| r.stakes.get(&x.persona) == Some(&Stake::Primary) && !x.blockers.is_empty())
            .collect();
        if !blocked.is_empty() {
            primary_count += blocked.iter().map(|x| x.blockers.len()).sum::<usize>();
            primary_personas.extend(blocked.iter().map(|x| x.persona.clone()));
            primary_runs.push(json!(r.id));
        }
        for s in apply::splits(&rows, &r.stakes) {
            splits.push(json!({"run": r.id, "lens": s.lens, "dimension": s.dimension, "about": s.about, "strata": s.strata}));
        }
    }
    if primary_count > 0 {
        reasons.push(reason(
            "primary_blocker",
            "escalate",
            json!({"blockers": primary_count, "personas": primary_personas, "runs": primary_runs}),
        ));
    }
    // a blocker a person raised (`verdict --by-human`): whatever the stratum, until a recorded decision closes it
    let human_rows: Vec<VerdictRow> = {
        let mut v = Vec::new();
        for r in &judged {
            v.extend(store.verdict_rows(r.id, false)?.into_iter().filter(|x| super::verdict::is_human(&x.persona)));
        }
        v
    };
    let mut human_open: Vec<Value> = Vec::new();
    for f in super::verdict::findings_of(&subject.name, &human_rows) {
        if store.latest_decision(&f.id)?.is_none() {
            human_open.push(json!({"finding": f.id, "persona": f.persona, "dimension": f.dimension, "text": f.text}));
        }
    }
    if !human_open.is_empty() {
        reasons.push(reason(
            "human_blocker",
            "escalate",
            json!({"findings": human_open, "remedy": "a person closes it with `decide <finding> fixed|dismissed|row --by-human <name>`"}),
        ));
    }
    // strata that disagree, within one run
    for s in splits {
        reasons.push(reason("split", "escalate", s));
    }
    // `check` failing for this subject: a finding of any of its runs with no decision, or a fix that did not land (I3)
    let all = store.runs()?;
    let of_subject: BTreeSet<i64> = all.iter().filter(|r| r.subject_name == subject.name).map(|r| r.id).collect();
    let check_data = match super::check::run(root, false) {
        Ok(d) => d,
        Err(AppError::CheckFailed(_, d)) => d,
        Err(e) => return Err(e),
    };
    let open_findings: Vec<Value> = check_data["unresolved"]
        .as_array()
        .map(|a| a.iter().filter(|u| u["run"].as_i64().is_some_and(|r| of_subject.contains(&r))).cloned().collect())
        .unwrap_or_default();
    if !open_findings.is_empty() {
        reasons.push(reason(
            "check_failed",
            "escalate",
            json!({"unresolved": open_findings.iter().map(|u| json!({"finding": u["finding"], "run": u["run"], "reason": u["reason"]})).collect::<Vec<_>>()}),
        ));
    }
    // D15: every juror ran on one model; informational, it changes no answer
    let models: BTreeSet<&str> = judged.iter().flat_map(|r| r.briefs.iter().map(|b| b.model.as_str())).collect();
    if models.len() == 1 {
        reasons.push(reason(
            "single_model",
            "info",
            json!({"model": models.iter().next(), "note": "one model judged every seat; a second model makes the jury less correlated"}),
        ));
    }
    // D18: one context read every brief: it advises, it never applies
    let sequential: Vec<i64> = given.iter().filter(|r| r.independence == "sequential").map(|r| r.id).collect();
    if !sequential.is_empty() {
        reasons.push(reason(
            "independence_sequential",
            "advise",
            json!({"runs": sequential, "remedy": "re-run the same quorum with one context per brief"}),
        ));
    }
    // D17: a cheap jury on a change that needs the high quorum advises, it never applies
    if effective == Some(Quorum::High) {
        for r in &judged {
            let cut = !r.trimmed.is_empty();
            if r.effort.as_deref() == Some("low") || cut {
                reasons.push(reason(
                    "effort_below_quorum",
                    "advise",
                    json!({
                        "run": r.id, "effort": r.effort, "quorum": quorum, "trimmed": r.trimmed,
                        "remedy": "run the same quorum again at a higher effort",
                    }),
                ));
            }
        }
    }
    // C4: a run whose recorded plan lowers seats, lenses, effort or detail below its rule, or that left the rule
    let toml_effort = cfg.as_ref().and_then(|c| c.effort).unwrap_or(Effort::Medium);
    for r in &judged {
        let mut fields: Vec<Value> = r.override_json.as_ref().map(lowered_fields).unwrap_or_default();
        if let Some(e) = r.effort.as_deref().and_then(Effort::parse).filter(|e| *e < toml_effort) {
            fields.push(json!({"field": "effort", "rule": toml_effort.name(), "given": e.name()}));
        }
        if r.rule.is_none() && (sources_q.is_some() || caller_q.is_some()) {
            fields.push(json!({
                "field": "rule", "rule": sources_rule.as_deref().or(rule), "given": Value::Null,
                "why": "briefed without the rule; its plan is not proven against it",
            }));
        }
        if !fields.is_empty() {
            reasons.push(reason(
                "overridden_down",
                "advise",
                json!({"run": r.id, "fields": fields, "remedy": "brief again under the rule, at or above its plan"}),
            ));
        }
    }
    // I1: a run whose subject moved, or that a later round of the same subject and lens superseded
    for r in &judged {
        let why = match super::verdict::moved_why(&store, r)? {
            Some(w) => Some(w),
            None => all
                .iter()
                .filter(|p| p.id > r.id && p.subject_name == r.subject_name && p.lens_name == r.lens_name)
                .map(|p| format!("superseded by run {}", p.id))
                .next_back(),
        };
        if let Some(why) = why {
            reasons.push(reason("stale_run", "advise", json!({"run": r.id, "why": why})));
        }
    }
    // a delta against the previous round of the same subject and lens (any lens version, I2)
    for r in &judged {
        let prev = all
            .iter()
            .filter(|p| p.id < r.id && p.subject_name == r.subject_name && p.lens_name == r.lens_name)
            .max_by_key(|p| p.id);
        let Some(prev) = prev else { continue };
        let unavailable = |why: String| {
            reason("delta_unavailable", "advise", json!({"lens": r.lens_name, "a": prev.id, "b": r.id, "why": why}))
        };
        if prev.lens_version != r.lens_version {
            reasons.push(unavailable(format!(
                "lens {} v{} then v{}: the rounds are not comparable",
                r.lens_name, prev.lens_version, r.lens_version
            )));
            continue;
        }
        match super::delta::show(root, prev.id, r.id) {
            Err(e) => reasons.push(unavailable(e.to_string())),
            Ok(d) => {
                if d["landed"] != json!(true) {
                    reasons.push(reason(
                        "delta_not_landed",
                        "escalate",
                        json!({"lens": r.lens_name, "a": prev.id, "b": r.id}),
                    ));
                }
                let new_cells = d["moves"].as_array().map_or(0, |m| m.iter().filter(|x| x["before"].is_null()).count());
                if new_cells > 0 {
                    reasons.push(unavailable(format!("{new_cells} cell(s) the previous round does not have")));
                }
            }
        }
    }

    let has = |effect: &str| reasons.iter().any(|r| r["effect"] == effect);
    let answer = Answer::from_facts(has("human"), has("escalate"), has("advise"));
    let mut out = json!({
        "answer": answer,
        "runs": judged.iter().map(|r| r.id).collect::<Vec<_>>(),
        "subject": subject.name,
        "quorum": quorum,
        "quorum_source": quorum_source,
        "reasons": reasons,
    });
    if answer == Answer::Escalate {
        out["next_quorum"] = json!(apply::next_quorum(quorum.unwrap_or("low")));
    }
    Ok(out)
}
