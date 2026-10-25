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
    std::fs::write(&bad, "## Q1 [class: product] One option only?\n- just this\n").unwrap();
    let r = jg(w, &["subject", "add", "d", "--kind", "decision", "--attach", bad.to_str().unwrap(), &notes]);
    assert_eq!(code(&r), "invalid_questions");
    assert!(r["error"]["message"].as_str().unwrap().contains("Q1"), "{r}");
    std::fs::write(&bad, "## Q1 [class: product] Two?\n- a\n- b\n## Q1 [class: product] Again?\n- a\n- b\n").unwrap();
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
    assert!(flat.contains("Needs a person: Q2 (money, split on money, split on spending) and Q3 (spending, split on money, split on spending, split on trust)."), "{t}");
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
        .filter(|r| r["code"] == "human_class" && r["via"] == "declared")
        .map(|r| (r["question"].as_str().unwrap().to_string(), r["class"].as_str().unwrap().to_string()))
        .collect();
    assert!(hits.contains(&("Q2".into(), "money".into())), "{hits:?}");
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
    std::fs::write(&q, "## Q1 [class: product] Which order should the checklist follow?\n- Install first\n- Account first\n").unwrap();
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

// ---- the v0.6-C review fixes (C2 to C4, I1): a strict questions file, a declared class, a split between strata ----

const LOW_RULE: &str = "[[rule]]\nclass = \"decisions\"\npaths = [\"**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"decide\"]\n";

/// A decision run under a `low` rule (so `apply` is reachable), four clean verdicts and the given picks per persona:
/// `picks(persona)` answers `[(question, choice)]`. The reviewer's `dec.sh`, as a test.
fn decided(qs: &str, extra: &[(&str, &str)], picks: impl Fn(&str) -> Vec<(String, String)>) -> (tempfile::TempDir, i64) {
    let ws = installed();
    let w = ws.path();
    for p in WHO {
        let f = repo().join(format!("fixtures/personas/{p}.json"));
        ok(jg(w, &["persona", "add", "--file", f.to_str().unwrap()]));
    }
    std::fs::write(w.join("personajury.toml"), LOW_RULE).unwrap();
    std::fs::write(w.join("questions.md"), qs).unwrap();
    let mut attach = vec![w.join("questions.md").to_string_lossy().into_owned()];
    for (name, body) in extra {
        std::fs::write(w.join(name), body).unwrap();
        attach.push(w.join(name).to_string_lossy().into_owned());
    }
    ok(jg(w, &["panel", "add", "p", "--query", QUERY]));
    let mut args = vec!["subject", "add", "d", "--kind", "decision", "--attach"];
    args.extend(attach.iter().map(|s| s.as_str()));
    ok(jg(w, &args));
    let run = ok(jg(w, &["brief", "--subject", "d", "--rule", "decisions", "--model", "m"]))["runs"][0]["run"].as_i64().unwrap();
    for p in WHO {
        let picks: Vec<Value> = picks(p).into_iter().map(|(q, c)| json!({"about": q, "choice": c, "reason": "r"})).collect();
        let v = json!({"persona": p, "picks": picks, "lines": [{"dimension": "confidence", "about": "Q1", "score": 4, "anchor_met": 3,
            "evidence": "seen", "citations": ["questions.md#L1"], "blockers": [], "note": "n"}]});
        let r = send(w, run, &v);
        assert_eq!(r["ok"], true, "{p}: {r}");
    }
    (ws, run)
}
fn everyone<'a>(choices: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Vec<(String, String)> + 'a {
    move |_| choices.iter().map(|(q, c)| (q.to_string(), c.to_string())).collect()
}
fn check(w: &Path, run: i64) -> Value {
    ok(jg(w, &["apply-check", &run.to_string()]))
}
fn reasons_of<'a>(a: &'a Value, code: &str) -> Vec<&'a Value> {
    a["reasons"].as_array().unwrap().iter().filter(|r| r["code"] == code).collect()
}
fn refused_questions(body: &str) -> Value {
    let ws = installed();
    let q = ws.path().join("questions.md");
    std::fs::write(&q, body).unwrap();
    jg(ws.path(), &["subject", "add", "d", "--kind", "decision", "--attach", q.to_str().unwrap()])
}

const NEUTRAL: &str = "## Q1 [class: product] Which order should the checklist follow?\n- Install first\n- Account first\n";

#[test]
fn the_neutral_baseline_applies() {
    let (ws, run) = decided(NEUTRAL, &[], everyone(&[("Q1", "Install first")]));
    assert_eq!(check(ws.path(), run)["answer"], "apply");
}

/// C2: a line the grammar does not know is refused with its line number; nothing is silently dropped.
#[test]
fn a_questions_file_outside_the_grammar_is_refused_naming_the_line() {
    let cases: [(&str, &str); 9] = [
        // a third-level heading under Q1: its options joined Q1 and the question vanished
        ("## Q1 [class: product] Which order?\n- Install first\n- Account first\n### Q2 [class: product] What should it cost?\n- 9 dollars\n- 12 dollars\n", "line 4"),
        ("## Q1 [class: product] Which order?\n- a\n- b\nQ2 Should we charge more?\n- yes\n- no\n", "line 4"),
        ("## Q1 [class: product] Which order?\n- a\n- b\n**Q3 Should we delete user data?**\n- yes\n- no\n", "line 4"),
        ("# Open questions\n\n## Q1 [class: product] Which order?\n- a\n- b\n", "line 1"),
        ("- a stray option\n## Q1 [class: product] Which order?\n- a\n- b\n", "line 1"),
        ("## Q1 [class: product] One?\n- a\n- b\n## Q01 [class: product] Two?\n- a\n- b\n", "line 4"),
        ("## Q1 [class: product] One?\n- a\n- b\n## Q1 [class: product] Again?\n- a\n- b\n", "line 4"),
        ("## Q2 [class: product] Starts at two?\n- a\n- b\n", "line 1"),
        ("## Q1 [class: product] One?\n- a\n- b\n#### q2 [class: product] lower case?\n- a\n- b\n", "line 4"),
    ];
    for (body, line) in cases {
        let r = refused_questions(body);
        assert_eq!(code(&r), "invalid_questions", "{body}\n{r}");
        assert!(r["error"]["message"].as_str().unwrap().contains(line), "{body}\n{r}");
    }
    // a question with one option names its line too
    let r = refused_questions("## Q1 [class: product] One?\n- a\n\n## Q2 [class: product] Two?\n- a\n- b\n");
    assert_eq!(code(&r), "invalid_questions");
    assert!(r["error"]["message"].as_str().unwrap().contains("line 1"), "{r}");
    // blank lines anywhere are fine; ids run 1, 2, 3
    let ok_body = "\n## Q1 [class: product] One?\n\n- a\n- b\n\n## Q2 [class: copy] Two?\n- a\n- b\n\n## Q3 [class: taste] Three?\n- c\n- d\n";
    assert_eq!(refused_questions(ok_body)["ok"], true);
}

/// C3: every question declares a class; a missing one is refused, an unknown one is refused.
#[test]
fn every_question_declares_a_class() {
    let r = refused_questions("## Q1 Which order?\n- a\n- b\n");
    assert_eq!(code(&r), "question_class_missing");
    assert!(r["error"]["message"].as_str().unwrap().contains("line 1"), "{r}");
    let r = refused_questions("## Q1 [class: vibes] Which order?\n- a\n- b\n");
    assert_eq!(code(&r), "invalid_questions");
    assert!(r["error"]["message"].as_str().unwrap().contains("vibes"), "{r}");
}

/// C3: a declared human class answers `human`, whatever the jury picked.
#[test]
fn a_declared_human_class_answers_human() {
    for class in ["money", "permission", "deploy", "secret", "licence", "public-claim", "brand", "legal", "spending", "irreversible"] {
        let qs = format!("## Q1 [class: {class}] Which order should the checklist follow?\n- Install first\n- Account first\n");
        let (ws, run) = decided(&qs, &[], everyone(&[("Q1", "Install first")]));
        let a = check(ws.path(), run);
        assert_eq!(a["answer"], "human", "{class}: {a}");
        let r = reasons_of(&a, "human_class");
        assert!(r.iter().any(|r| r["class"] == class && r["via"] == "declared" && r["question"] == "Q1"), "{class}: {a}");
    }
}

/// C3: the reviewer's questions, each declared a jury class, escalate to `human` by the stem table (`class_mismatch`).
#[test]
fn a_jury_class_whose_text_hits_a_stem_answers_human_with_the_stem() {
    let qs = "## Q1 [class: product] How much should we charge for the Team plan? ($10/$20)\n- 10\n- 20\n\
## Q2 [class: copy] Should we email every customer about the change?\n- yes\n- no\n\
## Q3 [class: ops] Should we drop the old accounts table?\n- yes\n- no\n\
## Q4 [class: ops] Can we ship the beta to everyone on Monday?\n- yes\n- no\n\
## Q5 [class: product] Should we raise the monthly fee?\n- yes\n- no\n\
## Q6 [class: ops] Should we sign the two new contracts?\n- yes\n- no\n\
## Q7 [class: taste] Which colour for the button?\n- blue\n- green\n";
    let picks = [("Q1", "10"), ("Q2", "yes"), ("Q3", "yes"), ("Q4", "yes"), ("Q5", "yes"), ("Q6", "yes"), ("Q7", "blue")];
    let (ws, run) = decided(qs, &[], everyone(&picks));
    let a = check(ws.path(), run);
    assert_eq!(a["answer"], "human", "{a}");
    let hits: Vec<(String, String)> = reasons_of(&a, "class_mismatch")
        .iter()
        .map(|r| {
            assert_eq!(r["effect"], "human", "{r}");
            (r["question"].as_str().unwrap().to_string(), r["stem"].as_str().unwrap().to_string())
        })
        .collect();
    for (q, stem) in [("Q1", "charg"), ("Q2", "email"), ("Q3", "drop"), ("Q4", "ship"), ("Q5", "fee"), ("Q6", "contract")] {
        assert!(hits.contains(&(q.to_string(), stem.to_string())), "{q} {stem}: {hits:?}");
    }
    assert!(!hits.iter().any(|(q, _)| q == "Q7"), "{hits:?}");
}

/// C4: a split between strata on a money or trust question answers `human`; on any other class, `advise`.
#[test]
fn a_split_between_strata_on_trust_answers_human_and_on_another_class_advise() {
    let trust = "## Q1 [class: product] Should the app keep a security log of every sign-in?\n- yes\n- no\n";
    let split = |p: &str| vec![("Q1".to_string(), if p.starts_with("builder") { "yes" } else { "no" }.to_string())];
    let (ws, run) = decided(trust, &[], split);
    let a = check(ws.path(), run);
    assert_eq!(a["answer"], "human", "{a}");
    let r = reasons_of(&a, "split_on_sensitive");
    assert!(r.iter().any(|r| r["question"] == "Q1" && r["class"] == "trust" && r["effect"] == "human"), "{a}");
    // the same split on a neutral question advises
    let (ws, run) = decided(NEUTRAL, &[], |p: &str| {
        vec![("Q1".to_string(), if p.starts_with("builder") { "Install first" } else { "Account first" }.to_string())]
    });
    let a = check(ws.path(), run);
    assert_eq!(a["answer"], "advise", "{a}");
    assert!(reasons_of(&a, "split_on_question").iter().any(|r| r["question"] == "Q1" && r["effect"] == "advise"), "{a}");
    // a declared money question with the strata split names both reasons
    let money = "## Q1 [class: money] What should the plan cost?\n- 9 dollars\n- 12 dollars\n";
    let (ws, run) = decided(money, &[], |p: &str| {
        vec![("Q1".to_string(), if p.starts_with("builder") { "9 dollars" } else { "12 dollars" }.to_string())]
    });
    let a = check(ws.path(), run);
    assert_eq!(a["answer"], "human");
    assert!(reasons_of(&a, "split_on_sensitive").iter().any(|r| r["class"] == "money"), "{a}");
}

/// I1: the point-of-view files are scanned with the same stems; the hits are listed for the person and change no answer.
#[test]
fn source_text_that_hits_a_stem_is_listed_and_changes_no_answer() {
    let notes = "The legal team must approve any change to pricing. This is irreversible once published.\n";
    let (ws, run) = decided(NEUTRAL, &[("notes.md", notes)], everyone(&[("Q1", "Install first")]));
    let a = check(ws.path(), run);
    assert_eq!(a["answer"], "apply", "{a}");
    let r = reasons_of(&a, "sources_mention");
    assert_eq!(r.len(), 1, "{a}");
    assert_eq!(r[0]["effect"], "info");
    let s = ok(jg(ws.path(), &["summary", &run.to_string(), "--width", "200"]))["text"].as_str().unwrap().to_string();
    let flat = s.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flat.contains("Sources mention: "), "{s}");
    for stem in ["legal", "pric", "irreversib", "publish"] {
        assert!(flat.contains(stem), "{stem}: {s}");
    }
    assert!(!flat.contains("Needs a person"), "{s}");
}

#[test]
fn a_summary_of_two_runs_tallies_each_question_once_over_the_latest_pick_of_each_persona() {
    let ws = world();
    let w = ws.path();
    let first = run_id(w);
    // a second run on the same subject where only one persona answered again
    let second = ok(brief(w))["run"].as_i64().unwrap();
    let r = jg(w, &["verdict", "--run", &second.to_string(), "--file", verdict_file("builder-ana").to_str().unwrap()]);
    assert_eq!(r["ok"], true, "{r}");
    let t = ok(jg(w, &["summary", &first.to_string(), &second.to_string(), "--width", "200"]))["text"].as_str().unwrap().to_string();
    let flat = t.split_whitespace().collect::<Vec<_>>().join(" ");
    assert_eq!(flat.matches("On Q1 (").count(), 1, "{t}");
    assert!(flat.contains("On Q1 (Which order should the onboarding checklist follow?), primary chose Install first, account second, 2 of 2;"), "{t}");
    assert_eq!(flat.matches("Needs a person:").count(), 1, "{t}");
}
