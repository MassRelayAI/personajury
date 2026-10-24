//! D35: `trait observed` counts what a persona did in its accepted initial verdicts and reads the mapping table. No model.
#[path = "judging_support/mod.rs"]
mod fx;
#[path = "traits_support/mod.rs"]
mod support;
use personajury::domain::Stake;
use serde_json::{json, Value};

/// A critical verdict: `scores` per line, `blockers` blocker strings on every line, evidence `kind`.
fn crit(persona: &str, scores: [i64; 3], blockers: usize, kind: &str) -> Value {
    let lines: Vec<Value> = ["works", "blocks", "missing"]
        .iter()
        .zip(scores)
        .map(|(d, s)| {
            let b: Vec<String> = (0..blockers).map(|i| format!("{d} problem {i}")).collect();
            let mut l = json!({"dimension": d, "score": s, "anchor_met": if s >= 3 { 3 } else { 1 }, "evidence": kind, "blockers": b});
            if kind == "seen" {
                l["citations"] = json!(["page.json#n1"]);
            }
            l
        })
        .collect();
    json!({"persona": persona, "lines": lines})
}
fn comp(persona: &str, choice: &str, scores: [i64; 3]) -> Value {
    let lines: Vec<Value> = ["clarity", "price", "trust"]
        .iter()
        .zip(scores)
        .map(|(d, s)| {
            json!({"dimension": d, "score": s, "anchor_met": if s >= 3 { 3 } else { 1 }, "evidence": "seen",
                   "citations": ["page.json#n1", "cmp:rival/rival.json#r1"], "blockers": if s <= 1 { json!(["too low"]) } else { json!([]) }})
        })
        .collect();
    json!({"persona": persona, "lines": lines, "pick": {"choice": choice, "reason": "because"}})
}

const PRIMARY: [&str; 4] = ["ana", "cat", "dan", "eve"];

/// Four primary jurors and one secondary; each holds `ease_demanded` 5 and a mid `situation.time_pressure`.
fn world() -> fx::Ws {
    let ws = fx::Ws::new();
    ws.site();
    ws.rival();
    support::ok(support::jg(ws.root(), &["trait", "install"]));
    for p in PRIMARY.iter().chain(&["bob"]) {
        support::add_ok(ws.root(), p, &["t"], json!({"adoption.ease_demanded": 5, "situation.time_pressure": 3}));
    }
    ws
}
fn stakes() -> Vec<(&'static str, Stake)> {
    vec![("ana", Stake::Primary), ("cat", Stake::Primary), ("dan", Stake::Primary), ("eve", Stake::Primary), ("bob", Stake::Secondary)]
}
fn observed(ws: &fx::Ws, who: &str) -> Value {
    support::ok(support::jg(ws.root(), &["trait", "observed", "--persona", who]))
}
fn rule<'a>(t: &'a Value, id: &str) -> &'a Value {
    t["rules"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap()
}

/// ana scores below the stratum median, cat at it, dan above; eve at it. bob sits alone in a stratum.
fn severity_run(ws: &fx::Ws) -> i64 {
    let run = ws.run("critical", &stakes(), false);
    for (p, s) in [("ana", 2), ("cat", 3), ("dan", 5), ("eve", 3), ("bob", 1)] {
        ws.ingest(run, &crit(p, [s, s, s], if s == 1 { 1 } else { 0 }, "seen"));
    }
    run
}

#[test]
fn severity_is_below_at_or_above_the_stratum_median_and_names_the_punished_cell() {
    let ws = world();
    severity_run(&ws);
    let ana = observed(&ws, "ana");
    assert_eq!(ana["runs"], json!([1]));
    let sev = ana["signals"]["severity"].as_array().unwrap();
    assert_eq!(sev.len(), 3);
    assert!(sev.iter().all(|c| c["lens"] == "critical" && c["below"] == 1 && c["at"] == 0 && c["above"] == 0), "{sev:?}");
    assert_eq!(ana["signals"]["punished"], json!({"lens": "critical", "dimension": "blocks"}), "ties go to the first in id order");
    let t = &ana["traits"].as_array().unwrap().iter().find(|t| t["trait"] == "adoption.ease_demanded").unwrap().clone();
    assert_eq!(t["verdict"], "consistent");
    assert_eq!(rule(t, "critical-severity")["counts"], json!({"below": 3, "at": 0, "above": 0}));
    let cat = observed(&ws, "cat");
    assert_eq!(cat["signals"]["severity"][0]["at"], 1);
    assert_eq!(cat["signals"]["punished"], Value::Null, "nothing below: no punished cell");
    let t = cat["traits"].as_array().unwrap().iter().find(|t| t["trait"] == "adoption.ease_demanded").unwrap().clone();
    assert_eq!(t["verdict"], "no_signal");
    let dan = observed(&ws, "dan");
    assert_eq!(dan["signals"]["severity"][0]["above"], 1);
    let t = dan["traits"].as_array().unwrap().iter().find(|t| t["trait"] == "adoption.ease_demanded").unwrap().clone();
    assert_eq!(t["verdict"], "contradicted");
    // a mid band predicts nothing, whatever the counts say
    let tp = ana["traits"].as_array().unwrap().iter().find(|t| t["trait"] == "situation.time_pressure").unwrap().clone();
    assert_eq!((tp["verdict"].clone(), tp["rules"].clone()), (json!("no_signal"), json!([])));
}

#[test]
fn a_stratum_with_fewer_than_two_other_scores_adds_no_severity() {
    let ws = world();
    severity_run(&ws);
    let bob = observed(&ws, "bob");
    assert_eq!(bob["signals"]["severity"], json!([]));
    assert_eq!(bob["signals"]["blockers"]["runs"], json!([]));
    assert_eq!(bob["signals"]["evidence"], json!({"seen": 3, "told": 0, "recalled": 0}), "evidence needs no peer");
}

#[test]
fn blockers_and_bottom_scores_are_counts_against_the_stratum_median() {
    let ws = world();
    let run = ws.run("critical", &stakes(), false);
    for (p, s, b) in [("ana", 1, 2), ("cat", 3, 0), ("dan", 3, 0), ("eve", 3, 1), ("bob", 3, 0)] {
        ws.ingest(run, &crit(p, [s, s, s], b, "seen"));
    }
    let ana = observed(&ws, "ana");
    assert_eq!(ana["signals"]["blockers"]["more"], 1);
    assert_eq!(ana["signals"]["blockers"]["runs"][0], json!({"run": 1, "count": 6, "stratum_median": 0, "word": "more"}));
    assert_eq!(ana["signals"]["bottom"], json!({"more": 1, "same": 0, "fewer": 0}));
    let sk = ana["traits"].as_array().unwrap().iter().find(|t| t["trait"] == "adoption.ease_demanded").unwrap().clone();
    assert_eq!(sk["verdict"], "consistent", "scoring below the stratum bears on adoption.ease_demanded high");
}

#[test]
fn evidence_counts_seen_told_and_recalled_over_its_lines() {
    let ws = world();
    let run = ws.run("critical", &stakes(), false);
    ws.ingest(run, &crit("ana", [3, 3, 3], 0, "told"));
    for p in ["cat", "dan", "eve", "bob"] {
        ws.ingest(run, &crit(p, [3, 3, 3], 0, "seen"));
    }
    assert_eq!(observed(&ws, "ana")["signals"]["evidence"], json!({"seen": 0, "told": 3, "recalled": 0}));
    assert_eq!(observed(&ws, "cat")["signals"]["evidence"], json!({"seen": 3, "told": 0, "recalled": 0}));
}

#[test]
fn picks_are_classed_and_set_against_the_strata_majority() {
    let ws = world();
    let run = ws.run("compare", &stakes(), true);
    for (p, c) in [("ana", "page.json"), ("cat", "page.json"), ("dan", "page.json"), ("eve", "cmp:rival"), ("bob", "shot.png")] {
        ws.ingest(run, &comp(p, c, [3, 3, 3]));
    }
    let ana = observed(&ws, "ana");
    assert_eq!(ana["signals"]["pick"], json!({"subject": 0, "comparator": 0, "attachment": 1, "with_majority": 1, "against_majority": 0}));
    let eve = observed(&ws, "eve");
    assert_eq!(eve["signals"]["pick"], json!({"subject": 0, "comparator": 1, "attachment": 0, "with_majority": 0, "against_majority": 1}));
    assert_eq!(observed(&ws, "bob")["signals"]["pick"]["with_majority"], 0, "a stratum of one has no majority to follow");
}

#[test]
fn a_tied_stratum_pick_counts_as_neither() {
    let ws = world();
    let run = ws.run("compare", &stakes(), true);
    for (p, c) in [("ana", "page.json"), ("cat", "page.json"), ("dan", "cmp:rival"), ("eve", "cmp:rival"), ("bob", "page.json")] {
        ws.ingest(run, &comp(p, c, [3, 3, 3]));
    }
    let ana = observed(&ws, "ana");
    assert_eq!((ana["signals"]["pick"]["with_majority"].clone(), ana["signals"]["pick"]["against_majority"].clone()), (json!(0), json!(0)));
    assert_eq!(ana["signals"]["pick"]["attachment"], 1);
}

#[test]
fn a_persona_without_traits_answers_an_empty_list_and_the_same_json_twice() {
    let ws = world();
    support::add_ok(ws.root(), "zed", &["t"], serde_json::Value::Null);
    severity_run(&ws);
    assert_eq!(observed(&ws, "zed")["traits"], json!([]));
    assert_eq!(observed(&ws, "ana"), observed(&ws, "ana"));
}

#[test]
fn several_personas_answer_a_list_and_runs_filters() {
    let ws = world();
    severity_run(&ws);
    let two = support::ok(support::jg(ws.root(), &["trait", "observed", "--persona", "ana", "--persona", "dan"]));
    assert_eq!(two["personas"].as_array().unwrap().len(), 2);
    assert_eq!(two["personas"][1]["persona"], "dan");
    let none = support::ok(support::jg(ws.root(), &["trait", "observed", "--persona", "ana", "--runs", "1"]));
    assert_eq!(none["runs"], json!([1]));
    let sofa = support::jg(ws.root(), &["trait", "observed", "--persona", "ana", "--runs", "9"]);
    assert_eq!(support::code(&sofa), "not_found");
    let ghost = support::jg(ws.root(), &["trait", "observed", "--persona", "ghost"]);
    assert_eq!(support::code(&ghost), "not_found");
    let bad = support::jg(ws.root(), &["trait", "observed", "--persona", "ana", "--runs", "x"]);
    assert_eq!(support::code(&bad), "invalid_arguments");
}

#[test]
fn it_writes_no_trait_back() {
    let ws = world();
    severity_run(&ws);
    let before = support::ok(support::jg(ws.root(), &["persona", "list"]));
    observed(&ws, "ana");
    assert_eq!(support::ok(support::jg(ws.root(), &["persona", "list"])), before);
}

#[test]
fn a_project_row_adds_a_rule_and_overrides_none() {
    let ws = world();
    severity_run(&ws);
    let dir = ws.root().join(".personajury");
    std::fs::write(
        dir.join("observe-map.toml"),
        "[[rule]]\nid = \"mine\"\nsignal = \"severity\"\nlens = \"critical\"\ndimension = \"works\"\ndirection = \"below\"\ntrait = \"adoption.ease_demanded\"\npole = \"high\"\n",
    )
    .unwrap();
    let ana = observed(&ws, "ana");
    let t = ana["traits"].as_array().unwrap().iter().find(|t| t["trait"] == "adoption.ease_demanded").unwrap().clone();
    assert_eq!(rule(&t, "mine")["counts"], json!({"below": 1, "at": 0, "above": 0}));
    assert!(rule(&t, "critical-severity")["counts"]["below"] == 3, "the shipped rule is untouched");
    std::fs::write(dir.join("observe-map.toml"), "[[rule]]\nid = \"critical-severity\"\nsignal = \"blockers\"\ndirection = \"more\"\ntrait = \"a.b\"\npole = \"high\"\n").unwrap();
    let v = support::jg(ws.root(), &["trait", "observed", "--persona", "ana"]);
    assert_eq!(support::code(&v), "invalid_arguments");
    std::fs::write(dir.join("observe-map.toml"), "[[rule]]\nid = \"x\"\nsignal = \"blockers\"\ndirection = \"below\"\ntrait = \"a.b\"\npole = \"high\"\n").unwrap();
    let v = support::jg(ws.root(), &["trait", "observed", "--persona", "ana"]);
    assert_eq!(support::code(&v), "invalid_arguments");
}
