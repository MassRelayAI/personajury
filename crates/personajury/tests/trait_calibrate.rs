//! D36: a human's own traits (`verdict --by-human --traits`), `calibrate --slice`, and `trait report`.
#[path = "judging_support/mod.rs"]
mod fx;
#[path = "traits_support/mod.rs"]
mod support;
use personajury::domain::Stake;
use serde_json::{json, Value};
use std::path::Path;

const T: &str = "consumer.price_consciousness";

fn crit(persona: &str, s: i64) -> Value {
    let lines: Vec<Value> = ["works", "blocks", "missing"]
        .iter()
        .map(|d| json!({"dimension": d, "score": s, "anchor_met": if s >= 3 { 3 } else { 1 }, "evidence": "seen",
                        "citations": ["page.json#n1"], "blockers": if s <= 1 { json!(["x"]) } else { json!([]) }}))
        .collect();
    json!({"persona": persona, "lines": lines})
}
fn personas() -> Vec<(&'static str, Stake)> {
    vec![("j1", Stake::Primary), ("j2", Stake::Primary), ("j3", Stake::Primary), ("j4", Stake::Primary)]
}
fn world() -> fx::Ws {
    let ws = fx::Ws::new();
    ws.site();
    support::ok(support::jg(ws.root(), &["trait", "install"]));
    for (p, band) in [("j1", 1), ("j2", 1), ("j3", 5), ("j4", 5)] {
        support::add_ok(ws.root(), p, &["t"], json!({ T: band }));
    }
    ws
}
fn human(ws: &fx::Ws, run: i64, name: &str, seat: &str, score: i64, traits: Option<Value>) -> Value {
    let mut args: Vec<String> = ["verdict", "--run", &run.to_string(), "--json", &crit(seat, score).to_string(), "--by-human", &format!("{name}, nurse")]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Some(t) = traits {
        let f = ws.root().join(format!("traits-{name}.json"));
        std::fs::write(&f, t.to_string()).unwrap();
        args.push("--traits".into());
        args.push(f.to_string_lossy().into_owned());
    }
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    ws.cli(&a)
}
/// A jury run and a human run. Jury: j1, j2 hold 1 and score `jl`; j3, j4 hold 5 and score `jh`. People: `pl` for two
/// people holding 1 and `ph` for `high_people` people holding 5 (seated on j3 and j4).
fn built(jl: i64, jh: i64, pl: i64, ph: i64, high_people: usize) -> (fx::Ws, i64, i64) {
    let ws = world();
    let jury = ws.run("critical", &personas(), false);
    for (p, s) in [("j1", jl), ("j2", jl), ("j3", jh), ("j4", jh)] {
        ws.ingest(jury, &crit(p, s));
    }
    let hr = ws.run("critical", &personas(), false);
    for (n, seat) in [("ann", "j1"), ("bea", "j2")] {
        assert_eq!(human(&ws, hr, n, seat, pl, Some(json!({ T: 1 })))["ok"], true);
    }
    for (n, seat) in [("cy", "j3"), ("di", "j4")].into_iter().take(high_people) {
        assert_eq!(human(&ws, hr, n, seat, ph, Some(json!({ T: 5 })))["ok"], true);
    }
    (ws, jury, hr)
}
fn slice(ws: &fx::Ws, j: i64, h: i64) -> Value {
    let v = ws.cli(&["calibrate", &j.to_string(), &h.to_string(), "--slice", T]);
    assert_eq!(v["ok"], true, "{v}");
    v["data"].clone()
}
fn word(d: &Value) -> String {
    let s = &d["slices"].as_array().unwrap()[0];
    assert_eq!((s["trait"].as_str(), s["stratum"].as_str()), (Some(T), Some("primary")));
    s["word"].as_str().unwrap().to_string()
}

#[test]
fn identical_separation_predicts() {
    let (ws, j, h) = built(2, 4, 2, 4, 2);
    let d = slice(&ws, j, h);
    assert_eq!(word(&d), "predicts");
    let s = &d["slices"][0];
    assert_eq!(s["separates_in_jury"], json!({"cells": 3, "total": 3}));
    assert_eq!(s["separates_in_people"], json!({"cells": 3, "total": 3}));
    let low = s["groups"].as_array().unwrap().iter().find(|g| g["band"] == "low").unwrap();
    assert_eq!((low["jury_n"].clone(), low["people_n"].clone()), (json!(2), json!(2)));
    assert!(low["cells"].as_array().unwrap().iter().all(|c| c["within_one_rank"] == true));
}
#[test]
fn a_jury_that_separates_where_the_people_do_not_is_played() {
    let (ws, j, h) = built(2, 4, 3, 3, 2);
    assert_eq!(word(&slice(&ws, j, h)), "played");
}
#[test]
fn people_who_separate_where_the_jury_does_not_are_missed() {
    let (ws, j, h) = built(3, 3, 2, 4, 2);
    assert_eq!(word(&slice(&ws, j, h)), "missed");
}
#[test]
fn neither_separating_is_inert() {
    let (ws, j, h) = built(3, 3, 3, 3, 2);
    assert_eq!(word(&slice(&ws, j, h)), "inert");
}
#[test]
fn a_group_of_one_makes_the_stratum_thin() {
    let (ws, j, h) = built(2, 4, 2, 4, 1);
    let d = slice(&ws, j, h);
    assert_eq!(word(&d), "thin");
    let high = d["slices"][0]["groups"].as_array().unwrap().iter().find(|g| g["band"] == "high").unwrap().clone();
    assert_eq!((high["jury_n"].clone(), high["people_n"].clone()), (json!(2), json!(1)));
}
#[test]
fn slices_leave_calibrated_and_apply_check_alone() {
    let (ws, j, h) = built(2, 4, 2, 4, 2);
    let plain = ws.cli(&["calibrate", &j.to_string(), &h.to_string()])["data"].clone();
    let sliced = slice(&ws, j, h);
    assert_eq!(plain["calibrated"], sliced["calibrated"]);
    assert_eq!(plain["strata"], sliced["strata"]);
    assert!(plain.get("slices").is_none());
    let (a, b) = (ws.root().join("a.json"), ws.root().join("b.json"));
    ws.cli(&["calibrate", &j.to_string(), &h.to_string(), "--out", a.to_str().unwrap()]);
    ws.cli(&["calibrate", &j.to_string(), &h.to_string(), "--slice", T, "--out", b.to_str().unwrap()]);
    assert!(std::fs::read_to_string(&b).unwrap().contains("\"slices\""));
    let ra = ws.cli(&["apply-check", &j.to_string(), "--calibration", a.to_str().unwrap()]);
    let rb = ws.cli(&["apply-check", &j.to_string(), "--calibration", b.to_str().unwrap()]);
    assert_eq!(ra["ok"], true, "{ra}");
    assert_eq!(ra["data"]["decision"], rb["data"]["decision"]);
    assert_eq!(ra["data"]["reasons"], rb["data"]["reasons"]);
}
#[test]
fn slice_refusals() {
    let (ws, j, h) = built(2, 4, 2, 4, 2);
    let v = ws.cli(&["calibrate", &j.to_string(), &h.to_string(), "--slice", "nope.nothing"]);
    assert_eq!(support::code(&v), "unknown_trait");
    let v = ws.cli(&["calibrate", &j.to_string(), &h.to_string(), "--slice", "situation.job"]);
    assert_eq!(support::code(&v), "invalid_arguments", "words are not grouped");
}
#[test]
fn a_side_without_the_trait_is_unset_and_left_out() {
    let ws = world();
    let jury = ws.run("critical", &personas(), false);
    for (p, s) in [("j1", 2), ("j2", 2), ("j3", 4), ("j4", 4)] {
        ws.ingest(jury, &crit(p, s));
    }
    let hr = ws.run("critical", &personas(), false);
    human(&ws, hr, "ann", "j1", 2, None);
    human(&ws, hr, "bea", "j2", 2, Some(json!({ T: 1 })));
    let d = slice(&ws, jury, hr);
    let g = d["slices"][0]["people_unset"].clone();
    assert_eq!(g, json!(["human:ann:j1"]));
}

// ---- human traits ----
#[test]
fn human_traits_are_validated_like_a_persona_s_and_shown_in_the_matrix() {
    let ws = world();
    let hr = ws.run("critical", &personas(), false);
    for (traits, want) in [
        (json!({"nope.nothing": 3}), "unknown_trait"),
        (json!({ T: 9 }), "trait_value_off_scale"),
        (json!({"x.gender": 3}), "refused_trait_id"),
        (json!({"choice.decision_style": "moody"}), "trait_option_unknown"),
    ] {
        assert_eq!(support::code(&human(&ws, hr, "ann", "j1", 3, Some(traits))), want);
    }
    let many: serde_json::Map<String, Value> = (0..9).map(|i| (format!("t.t{i}"), json!(3))).collect();
    assert_eq!(support::code(&human(&ws, hr, "ann", "j1", 3, Some(Value::Object(many)))), "traits_too_many");
    assert_eq!(human(&ws, hr, "ann", "j1", 3, Some(json!({ T: 4 })))["ok"], true);
    human(&ws, hr, "bea", "j2", 3, None);
    let m = ws.cli(&["matrix", &hr.to_string()])["data"].clone();
    let hs = m["human"].as_array().unwrap();
    assert_eq!(hs.iter().find(|h| h["persona"] == "human:ann:j1").unwrap()["traits"], json!({ T: 4 }));
    assert!(hs.iter().find(|h| h["persona"] == "human:bea:j2").unwrap().get("traits").is_none());
}
#[test]
fn traits_without_by_human_are_refused() {
    let ws = world();
    let run = ws.run("critical", &personas(), false);
    let f = ws.root().join("t.json");
    std::fs::write(&f, json!({ T: 4 }).to_string()).unwrap();
    let v = ws.cli(&["verdict", "--run", &run.to_string(), "--json", &crit("j1", 3).to_string(), "--traits", f.to_str().unwrap()]);
    assert_eq!(support::code(&v), "invalid_arguments");
}

// ---- trait report ----
fn out(ws: &fx::Ws, j: i64, h: i64, name: &str) -> String {
    let p = ws.root().join(name);
    ws.cli(&["calibrate", &j.to_string(), &h.to_string(), "--slice", T, "--out", p.to_str().unwrap()]);
    p.to_string_lossy().into_owned()
}
#[test]
fn trait_report_counts_the_words_across_files_per_trait_and_stratum() {
    let (ws, j, h) = built(2, 4, 2, 4, 2);
    let a = out(&ws, j, h, "a.json");
    let b = out(&ws, j, h, "b.json");
    // a third file from another world: the jury plays
    let (ws2, j2, h2) = built(2, 4, 3, 3, 2);
    let c = out(&ws2, j2, h2, "c.json");
    let r = ws.cli(&["trait", "report", "--calibration", &a, &b, &c]);
    assert_eq!(r["ok"], true, "{r}");
    let rows = r["data"]["report"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "one row per trait and stratum, no total across strata");
    assert_eq!(
        rows[0],
        json!({"trait": T, "stratum": "primary", "predicts": 2, "played": 1, "missed": 0, "inert": 0, "thin": 0, "files": 3})
    );
    assert!(r["data"]["table"].as_str().unwrap().contains("| trait | stratum | predicts | played | missed | inert | thin | files |"));
}
#[test]
fn a_file_that_is_not_a_calibration_is_refused() {
    let ws = world();
    let f = ws.root().join("nope.json");
    std::fs::write(&f, "{\"hello\": 1}").unwrap();
    let v = ws.cli(&["trait", "report", "--calibration", f.to_str().unwrap()]);
    assert_eq!(support::code(&v), "not_a_calibration");
    assert_eq!(v["data"]["path"], f.to_str().unwrap());
    let g = ws.root().join("garbage.json");
    std::fs::write(&g, "not json").unwrap();
    assert_eq!(support::code(&ws.cli(&["trait", "report", "--calibration", g.to_str().unwrap()])), "not_a_calibration");
    let _ = Path::new("");
}
