//! Decision juries (D37): a questions file and point-of-view files, the `decide` lens, a pick per question, and the
//! questions that still need a person.
#[path = "traits_support/mod.rs"]
mod support;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use support::*;

fn fx(rel: &str) -> String {
    repo().join("fixtures/decision").join(rel).to_string_lossy().into_owned()
}
const QUERY: &str = "2 builder=primary, 1 company=secondary, 1 investor=bystander";
const WHO: [&str; 4] = ["builder-ana", "builder-mei", "company-george", "investor-claire"];

fn world() -> tempfile::TempDir {
    let ws = installed();
    for p in ["builder-ana", "builder-mei", "company-george", "investor-claire"] {
        let f = repo().join(format!("fixtures/personas/{p}.json"));
        ok(jg(ws.path(), &["persona", "add", "--file", f.to_str().unwrap()]));
    }
    ok(jg(ws.path(), &["subject", "add", "next-release", "--kind", "decision", "--attach", &fx("questions.md"), &fx("owner-notes.md"), &fx("accepted-decisions.md")]));
    ws
}
fn brief(w: &Path) -> Value {
    jg(w, &["brief", "--subject", "next-release", "--query", QUERY, "--lens", "decide", "--model", "m"])
}
fn verdict_file(p: &str) -> PathBuf {
    repo().join(format!("fixtures/decision/verdicts/{p}.json"))
}
fn ingest_all(w: &Path, run: i64) {
    for p in WHO {
        let r = jg(w, &["verdict", "--run", &run.to_string(), "--file", verdict_file(p).to_str().unwrap()]);
        assert_eq!(r["ok"], true, "{p}: {r}");
    }
}
fn run_id(w: &Path) -> i64 {
    let r = ok(brief(w))["run"].as_i64().unwrap();
    ingest_all(w, r);
    r
}
fn verdict_json(p: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(verdict_file(p)).unwrap()).unwrap()
}
fn send(w: &Path, run: i64, v: &Value) -> Value {
    jg(w, &["verdict", "--run", &run.to_string(), "--json", &v.to_string()])
}

#[test]
fn a_decision_subject_needs_a_parseable_questions_file() {
    let ws = installed();
    let w = ws.path();
    let notes = fx("owner-notes.md");
    assert_eq!(code(&jg(w, &["subject", "add", "d", "--kind", "decision", "--attach", &notes])), "invalid_questions");
    let bad = w.join("questions.md");
    std::fs::write(&bad, "## Q1 One option only?\n- just this\n").unwrap();
    let r = jg(w, &["subject", "add", "d", "--kind", "decision", "--attach", bad.to_str().unwrap(), &notes]);
    assert_eq!(code(&r), "invalid_questions");
    assert!(r["error"]["message"].as_str().unwrap().contains("Q1"), "{r}");
    std::fs::write(&bad, "## Q1 Two?\n- a\n- b\n## Q1 Again?\n- a\n- b\n").unwrap();
    assert_eq!(code(&jg(w, &["subject", "add", "d", "--kind", "decision", "--attach", bad.to_str().unwrap()])), "invalid_questions");
}

#[test]
fn the_brief_lists_each_question_with_its_options_and_records_the_questions_mode() {
    let ws = world();
    let d = ok(brief(ws.path()));
    assert_eq!(d["options"]["mode"], "questions");
    assert_eq!(d["options"]["list"], json!(["Q1", "Q2", "Q3"]));
    let t = std::fs::read_to_string(ws.path().join(d["briefs"][0]["path"].as_str().unwrap())).unwrap();
    assert!(t.contains("### Q2 \u{2014} What should the Team plan cost per seat each month?"), "{t}");
    assert!(t.contains("- 12 dollars\n"), "{t}");
    assert!(t.contains("`picks` is required: one entry per question"), "{t}");
    assert!(!t.contains("## Pick"), "{t}");
    // the example in the brief is a verdict the engine accepts
    let start = t.find("```json\n").unwrap() + 8;
    let end = start + t[start..].find("\n```").unwrap();
    let mut ex: Value = serde_json::from_str(&t[start..end]).unwrap();
    ex["persona"] = json!("builder-ana");
    let r = send(ws.path(), d["run"].as_i64().unwrap(), &ex);
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn a_decision_needs_a_lens_that_asks_for_a_pick() {
    let ws = world();
    let r = jg(ws.path(), &["brief", "--subject", "next-release", "--query", QUERY, "--lens", "critical", "--model", "m"]);
    assert_eq!(code(&r), "invalid_arguments");
}

#[test]
fn every_question_is_picked_once_from_its_own_options() {
    let ws = world();
    let run = ok(brief(ws.path()))["run"].as_i64().unwrap();
    let w = ws.path();
    let mut v = verdict_json("builder-ana");
    v["picks"].as_array_mut().unwrap().pop();
    assert_eq!(code(&send(w, run, &v)), "question_pick_invalid");
    let mut v = verdict_json("builder-ana");
    v["picks"][1]["choice"] = json!("13 dollars");
    let r = send(w, run, &v);
    assert_eq!(code(&r), "question_pick_invalid");
    assert_eq!(r["data"]["question"], "Q2");
    let mut v = verdict_json("builder-ana");
    v["picks"][2] = v["picks"][0].clone();
    assert_eq!(code(&send(w, run, &v)), "question_pick_invalid");
    let mut v = verdict_json("builder-ana");
    v["pick"] = json!({"choice": "x", "reason": "y"});
    assert_eq!(code(&send(w, run, &v)), "question_pick_invalid");
    let mut v = verdict_json("builder-ana");
    v["picks"][0]["reason"] = json!(" ");
    assert_eq!(code(&send(w, run, &v)), "pick_without_reason");
    let mut v = verdict_json("builder-ana");
    v["lines"][0]["about"] = json!("Q9");
    assert_eq!(code(&send(w, run, &v)), "unknown_about");
}

#[test]
fn picks_group_by_question_then_stratum() {
    let ws = world();
    let run = run_id(ws.path());
    let p = ok(jg(ws.path(), &["pick", &run.to_string()]));
    let qs = p["questions"].as_array().unwrap();
    assert_eq!(qs.len(), 3);
    assert_eq!(qs[1]["about"], "Q2");
    assert_eq!(qs[1]["question"], "What should the Team plan cost per seat each month?");
    let primary = &qs[1]["strata"][0];
    assert_eq!(primary["stake"], "primary");
    assert_eq!(primary["choices"].as_array().unwrap().len(), 2, "a split in the primary stratum: {primary}");
    assert_eq!(qs[0]["strata"][0]["choices"][0]["count"], 2);
}

#[test]
fn the_summary_shows_each_questions_picks_and_who_needs_a_person() {
    let ws = world();
    let run = run_id(ws.path());
    let t = ok(jg(ws.path(), &["summary", &run.to_string(), "--width", "200"]))["text"].as_str().unwrap().to_string();
    let flat = t.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flat.contains("On Q1 (Which order should the onboarding checklist follow?), primary chose Install first, account second, 2 of 2; secondary chose Install first, account second, 1 of 1; bystander chose Account first, install second, 1 of 1."), "{t}");
    assert!(flat.contains("On Q2 (What should the Team plan cost per seat each month?), primary was split, 1 of 2 each for the top choices;"), "{t}");
    assert!(flat.contains("Needs a person: Q2 (public price, split on money) and Q3 (spending)."), "{t}");
    assert!(!flat.contains("Q1 (public"), "{t}");
}

#[test]
fn apply_check_answers_human_for_a_question_in_a_human_class() {
    let ws = world();
    let run = run_id(ws.path());
    let a = ok(jg(ws.path(), &["apply-check", &run.to_string()]));
    assert_eq!(a["answer"], "human", "{a}");
    let hits: Vec<(String, String)> = a["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["code"] == "human_class" && r["via"] == "question")
        .map(|r| (r["question"].as_str().unwrap().to_string(), r["class"].as_str().unwrap().to_string()))
        .collect();
    assert!(hits.contains(&("Q2".into(), "public price".into())), "{hits:?}");
    assert!(hits.contains(&("Q3".into(), "spending".into())), "{hits:?}");
    assert!(!hits.iter().any(|(q, _)| q == "Q1"), "{hits:?}");
}

#[test]
fn a_question_with_no_human_class_and_no_split_stays_with_the_jury() {
    let ws = installed();
    let w = ws.path();
    for p in ["builder-ana", "builder-mei", "company-george", "investor-claire"] {
        let f = repo().join(format!("fixtures/personas/{p}.json"));
        ok(jg(w, &["persona", "add", "--file", f.to_str().unwrap()]));
    }
    let q = w.join("questions.md");
    std::fs::write(&q, "## Q1 Which order should the checklist follow?\n- Install first\n- Account first\n").unwrap();
    ok(jg(w, &["subject", "add", "tiny", "--kind", "decision", "--attach", q.to_str().unwrap(), &fx("owner-notes.md")]));
    let run = ok(jg(w, &["brief", "--subject", "tiny", "--query", QUERY, "--lens", "decide", "--model", "m"]))["run"].as_i64().unwrap();
    for p in WHO {
        let mut v = verdict_json(p);
        v["lines"] = json!([v["lines"][0].clone()]);
        v["picks"] = json!([{"about": "Q1", "choice": "Install first", "reason": "r"}]);
        let r = send(w, run, &v);
        assert_eq!(r["ok"], true, "{p}: {r}");
    }
    let s = ok(jg(w, &["summary", &run.to_string()]))["text"].as_str().unwrap().to_string();
    assert!(!s.contains("Needs a person"), "{s}");
    let a = ok(jg(w, &["apply-check", &run.to_string()]));
    assert_ne!(a["answer"], "human", "{a}");
}
