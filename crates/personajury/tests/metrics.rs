//! The honest metrics (D34): agreement, shift, evidence totals, panel match and cost, as counts per stratum and never a mean.
mod judging_support;
#[path = "traits_support/mod.rs"]
mod support;
use judging_support::*;
use personajury::domain::Stake;
use serde_json::{json, Value};

const FOUR: [(&str, Stake); 4] = [("ana", Stake::Primary), ("bob", Stake::Primary), ("cat", Stake::Primary), ("dan", Stake::Primary)];
const THREE_P: [(&str, Stake); 3] = [("ana", Stake::Primary), ("bob", Stake::Primary), ("cara", Stake::Primary)];

fn matrix(ws: &Ws, run: i64) -> Value {
    let v = ws.cli(&["matrix", &run.to_string(), "--detail", "full"]);
    assert_eq!(v["ok"], true, "{v}");
    v["data"].clone()
}
fn cell<'a>(m: &'a Value, stratum: usize, dim: &str) -> &'a Value {
    m["strata"][stratum]["cells"].as_array().unwrap().iter().find(|c| c["dimension"] == dim).unwrap()
}

#[test]
fn within_one_counts_the_scores_within_one_point_of_the_median() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &FOUR, false);
    for (p, w) in [("ana", 2), ("bob", 2), ("cat", 3), ("dan", 5)] {
        ws.ingest(run, &critical(p, [w, 2, 2], &[], &["page.json#n1"]));
    }
    let m = matrix(&ws, run);
    let c = cell(&m, 0, "works");
    // scores 2, 2, 3, 5: the lower middle is 2, so 2, 2 and 3 are within one and 5 is not
    assert_eq!((c["median"].as_i64(), c["within_one"].as_i64()), (Some(2), Some(3)));
    assert_eq!(cell(&m, 0, "blocks")["within_one"], 4);
}

#[test]
fn a_stratum_counts_the_cells_on_which_every_juror_is_within_one() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &FOUR, false);
    for (p, w) in [("ana", 2), ("bob", 2), ("cat", 3), ("dan", 5)] {
        ws.ingest(run, &critical(p, [w, 2, 2], &[], &["page.json#n1"]));
    }
    let m = matrix(&ws, run);
    assert_eq!(m["strata"][0]["agreement"], json!({"cells": 3, "cells_all_within_one": 2}));
}

#[test]
fn evidence_totals_are_summed_per_stratum() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    ws.ingest(run, &told("bob", &[]));
    let m = matrix(&ws, run);
    assert_eq!(m["strata"][0]["evidence"], json!({"seen": 3, "told": 0, "recalled": 0}));
    assert_eq!(m["strata"][1]["evidence"], json!({"seen": 0, "told": 3, "recalled": 0}));
}

#[test]
fn panel_match_counts_asked_seated_and_answered_per_term() {
    use support::*;
    let ws = installed();
    let w = ws.path();
    for id in ["p1", "p2", "p3"] {
        add_ok(w, id, &["x"], Value::Null);
    }
    let f = w.join("page.md");
    std::fs::write(&f, "# page\n").unwrap();
    ok(jg(w, &["subject", "add", "s", "--kind", "document", "--attach", f.to_str().unwrap()]));
    let d = ok(jg(w, &["brief", "--subject", "s", "--query", "3 x=primary", "--lens", "critical", "--model", "m"]));
    let run = d["run"].as_i64().unwrap();
    for p in ["p1", "p2"] {
        let r = jg(w, &["verdict", "--run", &run.to_string(), "--json", &told(p, &[]).to_string()]);
        assert_eq!(r["ok"], true, "{r}");
    }
    let m = ok(jg(w, &["matrix", &run.to_string()]));
    assert_eq!(
        m["panel_match"],
        json!([{"run": run, "term": "3 x=primary", "tag": "x", "stake": "primary", "asked": 3, "seated": 3, "answered": 2}])
    );
}

#[test]
fn cost_lists_each_jurors_own_numbers_and_divides_nothing() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    let mut a = critical("ana", [3, 3, 3], &[], &["page.json#n1"]);
    a["usage"] = json!({"input_tokens": 1000, "output_tokens": 200});
    let mut b = critical("bob", [3, 3, 3], &[], &["page.json#n1"]);
    b["usage"] = json!({"input_tokens": 3000, "output_tokens": 400});
    ws.ingest(run, &a);
    ws.ingest(run, &b);
    let m = matrix(&ws, run);
    let c = &m["cost"];
    assert_eq!(c["jurors_reporting"], 2);
    assert_eq!(c["reported_total"], json!({"input_tokens": 4000, "output_tokens": 600}));
    let per: Vec<(String, i64, i64)> = c["per_juror"]
        .as_array()
        .unwrap()
        .iter()
        .map(|j| (j["persona"].as_str().unwrap().to_string(), j["input_tokens"].as_i64().unwrap(), j["output_tokens"].as_i64().unwrap()))
        .collect();
    assert_eq!(per, vec![("ana".to_string(), 1000, 200), ("bob".to_string(), 3000, 400)]);
    assert_eq!(c["estimate"]["briefs"], 2);
    let t = ws.cli(&["summary", &run.to_string(), "--width", "200"])["data"]["text"].as_str().unwrap().to_string();
    assert!(t.trim_end().ends_with("cost: reported 4000 in, 600 out over 2 jurors (estimate 30000)"), "{t}");
    assert!(t.contains("cost, seat ana (run 1): 1000 in, 200 out"), "{t}");
}

#[test]
fn cost_says_nothing_was_reported_when_nothing_was() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    let m = matrix(&ws, run);
    assert_eq!(m["cost"]["jurors_reporting"], 0);
    assert_eq!(m["cost"]["per_juror"], json!([]));
    let t = ws.cli(&["summary", &run.to_string(), "--width", "200"])["data"]["text"].as_str().unwrap().to_string();
    assert!(t.trim_end().ends_with("cost: reported \u{2014} (estimate 30000)"), "{t}");
    assert!(!t.contains("cost, seat"), "{t}");
}

fn persona(w: &std::path::Path, id: &str) {
    let f = w.join(format!("{id}.persona.json"));
    let body = json!({"id": id, "name": format!("Name {id}"), "tags": ["x"], "role": "r", "context": "c", "daily_tasks": "d",
        "fears": "f", "never_reads": "n", "decides_by": "b", "stake_rules": "s"});
    std::fs::write(&f, body.to_string()).unwrap();
    assert_eq!(cli_in(w, &["persona", "add", "--file", f.to_str().unwrap()])["ok"], true);
}
fn delib(p: &str, s: [i64; 3]) -> Value {
    let mut v = critical(p, s, &[], &["page.json#n1"]);
    v["phase"] = json!("deliberated");
    v
}

#[test]
fn shift_counts_who_moved_which_way_and_toward_or_away_from_the_first_median() {
    let ws = Ws::new();
    ws.site();
    for p in ["ana", "bob", "cara"] {
        persona(ws.root(), p);
    }
    let run = ws.run("critical", &THREE_P, false);
    ws.ingest(run, &critical("ana", [2, 3, 3], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [5, 3, 3], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("cara", [3, 3, 3], &[], &["page.json#n1"]));
    // no shift key before a deliberation
    assert!(matrix(&ws, run)["strata"][0].get("shift").is_none());
    assert_eq!(ws.cli(&["deliberate", &run.to_string()])["ok"], true);
    for v in [delib("ana", [3, 4, 3]), delib("bob", [4, 3, 3]), delib("cara", [3, 3, 3])] {
        let r = ws.cli(&["verdict", "--run", &run.to_string(), "--json", &v.to_string(), "--deliberated"]);
        assert_eq!(r["ok"], true, "{r}");
    }
    let m = matrix(&ws, run);
    let s = &m["strata"][0]["shift"];
    assert_eq!(
        (&s["moved_up"], &s["moved_down"], &s["held"], &s["toward_first_median"], &s["away_from_first_median"]),
        (&json!(2), &json!(1), &json!(6), &json!(2), &json!(1)),
        "{s}"
    );
    let works = s["cells"].as_array().unwrap().iter().find(|c| c["dimension"] == "works").unwrap();
    let step = |p: &str| works["personas"].as_array().unwrap().iter().find(|x| x["persona"] == p).unwrap().clone();
    assert_eq!(step("ana"), json!({"persona": "ana", "before": 2, "after": 3, "step": 1}));
    assert_eq!(step("bob"), json!({"persona": "bob", "before": 5, "after": 4, "step": -1}));
    assert_eq!(step("cara"), json!({"persona": "cara", "before": 3, "after": 3, "step": 0}));
}

#[test]
fn the_summary_carries_the_agreement_evidence_panel_and_cost_sentences_on_the_fixture_store() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let m = j(ws.path(), &["matrix", &a.to_string(), &b.to_string()])["data"].clone();
    let want: Vec<String> = m["strata"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            let name = s["stake"].as_str().unwrap();
            let name = format!("{}{}", name[..1].to_uppercase(), &name[1..]);
            format!("{name} jurors agree within one anchor on {} of {} scores", s["agreement"]["cells_all_within_one"], s["agreement"]["cells"])
        })
        .collect();
    let (mut seen, mut told, mut rec) = (0, 0, 0);
    for s in m["strata"].as_array().unwrap() {
        seen += s["evidence"]["seen"].as_i64().unwrap();
        told += s["evidence"]["told"].as_i64().unwrap();
        rec += s["evidence"]["recalled"].as_i64().unwrap();
    }
    let t = j(ws.path(), &["summary", &a.to_string(), &b.to_string(), "--width", "200"])["data"]["text"].as_str().unwrap().to_string();
    let flat = t.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flat.contains(&format!("{}.", want.join("; "))), "{want:?}\n{t}");
    assert!(flat.contains(&format!("Of {} verdict lines, {seen} were seen, {told} told and {rec} recalled.", seen + told + rec)), "{t}");
    assert!(flat.contains("panel: asked 3 builder, seated 3, answered 3; asked 2 company, seated 2, answered 2; asked 2 others, seated 2, answered 2"), "{t}");
    let last = t.trim_end().lines().last().unwrap();
    assert!(last.starts_with("cost: "), "{t}");
    // the sentences come before the table
    assert!(t.find("jurors agree within one anchor").unwrap() < t.find("Numbers, for checking").unwrap());
}

#[test]
fn the_summary_names_the_shift_in_the_primary_stratum_after_a_deliberation() {
    let ws = Ws::new();
    ws.site();
    for p in ["ana", "bob", "cara"] {
        persona(ws.root(), p);
    }
    let run = ws.run("critical", &THREE_P, false);
    ws.ingest(run, &critical("ana", [2, 3, 3], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [5, 3, 3], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("cara", [3, 3, 3], &[], &["page.json#n1"]));
    ws.cli(&["deliberate", &run.to_string()]);
    for v in [delib("ana", [3, 4, 3]), delib("bob", [4, 3, 3]), delib("cara", [3, 3, 3])] {
        ws.cli(&["verdict", "--run", &run.to_string(), "--json", &v.to_string(), "--deliberated"]);
    }
    let t = ws.cli(&["summary", &run.to_string(), "--width", "200"])["data"]["text"].as_str().unwrap().to_string();
    let flat = t.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flat.contains("After deliberation, 2 primary scores moved up, 1 down and 6 held; 2 moved toward the group's first median."), "{t}");
}

fn walk(v: &Value, bad: &mut Vec<String>) {
    const WORDS: [&str; 6] = ["mean", "average", "share", "rate", "percent", "probability"];
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                if k.split('_').any(|t| WORDS.contains(&t)) {
                    bad.push(k.clone());
                }
                walk(x, bad);
            }
        }
        Value::Array(a) => a.iter().for_each(|x| walk(x, bad)),
        _ => {}
    }
}

#[test]
fn no_key_of_the_matrix_is_a_mean_an_average_a_share_a_rate_a_percent_or_a_probability() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let m = j(ws.path(), &["matrix", &a.to_string(), &b.to_string(), "--detail", "full", "--phase", "both", "--table"]);
    let mut bad = vec![];
    walk(&m["data"], &mut bad);
    assert!(bad.is_empty(), "{bad:?}");
    let s = j(ws.path(), &["summary", &a.to_string(), &b.to_string()]);
    walk(&s["data"], &mut bad);
    assert!(bad.is_empty(), "{bad:?}");
}
