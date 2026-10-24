//! `deliberate RUN` and `verdict --deliberated` (D33): one advise-only round that measures persuasion.
mod judging_support;
use judging_support::*;
use personajury::domain::Stake;
use serde_json::{json, Value};
use std::path::Path;

const LOW: &str = "[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/site/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n";
const SEATS: [(&str, Stake); 3] = [("ana", Stake::Primary), ("cara", Stake::Primary), ("bob", Stake::Secondary)];

fn persona(w: &Path, id: &str, name: &str) {
    let f = w.join(format!("{id}.persona.json"));
    let body = json!({
        "id": id, "name": name, "tags": ["x"], "role": "r", "context": "c", "daily_tasks": "d", "fears": "f",
        "never_reads": "n", "decides_by": "b", "stake_rules": "s"
    });
    std::fs::write(&f, body.to_string()).unwrap();
    let r = cli_in(w, &["persona", "add", "--file", f.to_str().unwrap()]);
    assert_eq!(r["ok"], true, "{r}");
}
/// The site, three personas with distinct names, a run of the three, and the three initial verdicts.
fn world() -> (Ws, i64) {
    let ws = Ws::new();
    ws.site();
    persona(ws.root(), "ana", "Ana Alpha");
    persona(ws.root(), "cara", "Cara Gamma");
    persona(ws.root(), "bob", "Bob Beta");
    let run = ws.run("critical", &SEATS, false);
    ws.store.conn().execute("UPDATE run SET rule_class = 'public-surface', quorum = 'low' WHERE id = ?1", [run]).unwrap();
    std::fs::write(ws.root().join("personajury.toml"), LOW).unwrap();
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("cara", [2, 3, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    (ws, run)
}
fn rs(i: i64) -> String {
    i.to_string()
}
fn deliberate(ws: &Ws, run: i64) -> Value {
    ws.cli(&["deliberate", &rs(run)])
}
fn dfile(ws: &Ws, persona: &str, run: i64) -> String {
    std::fs::read_to_string(ws.root().join(format!(".personajury/runs/{run}/deliberate-{persona}--critical.md"))).unwrap()
}
fn code(v: &Value) -> String {
    assert_eq!(v["ok"], false, "{v}");
    v["error"]["code"].as_str().unwrap().to_string()
}
fn delib(persona: &str, scores: [i64; 3]) -> Value {
    let mut v = critical(persona, scores, &[], &["page.json#n1"]);
    v["phase"] = json!("deliberated");
    v
}
fn ingest_d(ws: &Ws, run: i64, v: &Value) -> Value {
    ws.cli(&["verdict", "--run", &rs(run), "--json", &v.to_string(), "--deliberated"])
}

#[test]
fn an_incomplete_jury_cannot_deliberate() {
    let ws = Ws::new();
    ws.site();
    persona(ws.root(), "ana", "Ana Alpha");
    persona(ws.root(), "bob", "Bob Beta");
    let run = ws.run("critical", &[("ana", Stake::Primary), ("bob", Stake::Secondary)], false);
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    let r = deliberate(&ws, run);
    assert_eq!(code(&r), "incomplete_jury_for_deliberation");
    assert_eq!(r["data"]["missing"], json!(["bob"]));
}

#[test]
fn one_round_only() {
    let (ws, run) = world();
    let d = deliberate(&ws, run);
    assert_eq!(d["ok"], true, "{d}");
    let again = deliberate(&ws, run);
    assert_eq!(code(&again), "already_deliberated");
    assert_eq!(again["data"]["run"], run);
}

#[test]
fn the_answer_lists_a_file_per_seat_with_hashes_and_waves() {
    let (ws, run) = world();
    let d = deliberate(&ws, run)["data"].clone();
    assert_eq!(d["briefs"].as_array().unwrap().len(), 3);
    let b = &d["briefs"][0];
    let text = dfile(&ws, b["persona"].as_str().unwrap(), run);
    assert_eq!(b["hash"], personajury::domain::hash::sha256_hex(text.as_bytes()));
    assert_eq!(b["bytes"], text.len());
    assert_eq!(d["waves"], json!([["ana", "cara", "bob"]]));
}

#[test]
fn the_wave_size_splits_the_waves() {
    let (ws, run) = world();
    let d = ws.cli(&["deliberate", &rs(run), "--wave", "2"])["data"].clone();
    assert_eq!(d["waves"], json!([["ana", "cara"], ["bob"]]));
}

#[test]
fn the_file_is_the_same_bytes_in_two_workspaces() {
    let (w1, r1) = world();
    let (w2, r2) = world();
    deliberate(&w1, r1);
    deliberate(&w2, r2);
    for p in ["ana", "cara", "bob"] {
        assert_eq!(dfile(&w1, p, r1), dfile(&w2, p, r2), "{p}");
    }
}

#[test]
fn the_file_names_no_persona_but_the_readers_own() {
    let (ws, run) = world();
    deliberate(&ws, run);
    let t = dfile(&ws, "ana", run);
    assert!(t.starts_with("personajury-deliberation 1\n\n# Ana Alpha reconsiders site under the critical lens\n"), "{t}");
    for other in ["Cara Gamma", "Bob Beta", "cara", "bob"] {
        assert!(!t.contains(other), "{other} leaked:\n{t}");
    }
    assert!(t.contains("## Your first verdict"));
    assert!(t.contains("## What the other jurors said"));
    assert!(t.contains("Juror 1") && t.contains("Juror 2"));
    // persona-id order: cara then bob would be "bob" first (b < c): Juror 1 is the secondary seat
    let j1 = t.find("Juror 1").unwrap();
    assert!(t[j1..].starts_with("Juror 1 (secondary)"), "{t}");
    assert!(t.contains("Juror 2 (primary)"));
    assert!(!t.contains("## Model"));
    assert!(t.contains("## What to do") && t.contains("\"phase\": \"deliberated\""));
}

#[test]
fn a_deliberated_verdict_is_stored_with_its_phase() {
    let (ws, run) = world();
    deliberate(&ws, run);
    let r = ingest_d(&ws, run, &delib("cara", [4, 4, 4]));
    assert_eq!(r["ok"], true, "{r}");
    let phases: Vec<(i64, Option<String>)> = ws
        .store
        .conn()
        .prepare("SELECT DISTINCT batch, phase FROM verdict WHERE persona_id = 'cara' ORDER BY batch")
        .unwrap()
        .query_map([], |x| Ok((x.get(0)?, x.get(1)?)))
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    assert_eq!(phases, vec![(1, None), (2, Some("deliberated".to_string()))]);
}

#[test]
fn the_phase_and_the_flag_must_agree_and_the_round_must_exist() {
    let (ws, run) = world();
    // the flag before any deliberation
    assert_eq!(code(&ingest_d(&ws, run, &delib("cara", [4, 4, 4]))), "no_deliberation");
    deliberate(&ws, run);
    // a phase with no flag
    let r = ws.cli(&["verdict", "--run", &rs(run), "--json", &delib("cara", [4, 4, 4]).to_string()]);
    assert_eq!(code(&r), "phase_without_flag");
    // the flag with no phase
    let r = ws.cli(&["verdict", "--run", &rs(run), "--json", &critical("cara", [4, 4, 4], &[], &["page.json#n1"]).to_string(), "--deliberated"]);
    assert_eq!(code(&r), "deliberated_without_phase");
    // a persona with no initial verdict
    assert_eq!(code(&ingest_d(&ws, run, &delib("ghost", [4, 4, 4]))), "deliberated_without_initial");
}

#[test]
fn a_deliberated_verdict_meets_the_same_evidence_rules() {
    let (ws, run) = world();
    deliberate(&ws, run);
    let mut v = delib("cara", [4, 4, 4]);
    v["lines"][0]["citations"] = json!([]);
    assert_eq!(code(&ingest_d(&ws, run, &v)), "seen_without_citation");
}

fn cells(m: &Value) -> Value {
    json!(m["strata"].as_array().unwrap().iter().map(|s| s["cells"].clone()).collect::<Vec<_>>())
}

#[test]
fn the_matrix_default_is_the_initial_batch_and_phase_shows_the_other() {
    let (ws, run) = world();
    let before = ws.cli(&["matrix", &rs(run), "--detail", "full"])["data"].clone();
    deliberate(&ws, run);
    assert_eq!(ingest_d(&ws, run, &delib("cara", [4, 4, 4]))["ok"], true);
    let after = ws.cli(&["matrix", &rs(run), "--detail", "full"])["data"].clone();
    assert_eq!(cells(&before)[0][0]["scores"], cells(&after)[0][0]["scores"]);
    assert_eq!(cells(&after)[0][0]["scores"]["cara"], 2);
    let d = ws.cli(&["matrix", &rs(run), "--detail", "full", "--phase", "deliberated"])["data"].clone();
    assert_eq!(cells(&d)[0][0]["scores"]["cara"], 4);
    assert_eq!(cells(&d)[0][0]["n"], 1);
    let both = ws.cli(&["matrix", &rs(run), "--detail", "full", "--phase", "both"])["data"].clone();
    assert_eq!(both["phases"]["initial"], after["strata"]);
    assert_eq!(both["phases"]["deliberated"], d["strata"]);
    assert_eq!(code(&ws.cli(&["matrix", &rs(run), "--phase", "later"])), "invalid_arguments");
}

#[test]
fn check_history_labels_the_deliberated_batch() {
    let (ws, run) = world();
    deliberate(&ws, run);
    ingest_d(&ws, run, &delib("cara", [4, 4, 4]));
    let v = ws.cli(&["check", "--history"]);
    let text = v.to_string();
    assert!(text.contains("deliberated"), "{text}");
}

#[test]
fn apply_check_answers_advise_deliberated_where_it_answered_apply() {
    let (ws, run) = world();
    let a = ws.cli(&["apply-check", &rs(run)])["data"].clone();
    assert_eq!(a["answer"], "apply", "{a}");
    deliberate(&ws, run);
    ingest_d(&ws, run, &delib("cara", [4, 4, 4]));
    let a = ws.cli(&["apply-check", &rs(run)])["data"].clone();
    assert_eq!(a["answer"], "advise", "{a}");
    let r = a["reasons"].as_array().unwrap().iter().find(|r| r["code"] == "deliberated").expect("a deliberated reason");
    assert_eq!(r["effect"], "advise");
    assert!(r["remedy"].as_str().unwrap().contains("independent word"));
}

#[test]
fn deliberation_alone_without_a_deliberated_verdict_leaves_apply() {
    let (ws, run) = world();
    deliberate(&ws, run);
    let a = ws.cli(&["apply-check", &rs(run)])["data"].clone();
    assert_eq!(a["answer"], "apply", "{a}");
}
