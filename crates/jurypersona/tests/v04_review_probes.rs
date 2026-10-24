//! Review probes for v0.4: each test asserts the behaviour the brief requires; a failure is a finding.
mod judging_support;
use jurypersona::domain::Stake;
use judging_support::*;
use serde_json::{json, Value};
use std::path::Path;

fn data(v: Value) -> Value {
    assert_eq!(v["ok"], true, "{v}");
    v["data"].clone()
}
fn ac(ws: &Ws, args: &[&str]) -> Value {
    let mut a = vec!["apply-check"];
    a.extend_from_slice(args);
    let d = data(ws.cli(&a));
    eprintln!("PROBE apply-check {args:?} => {}", d);
    d
}
const HIGH_RULE: &str = "[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/site/**\"]\nquorum = \"high\"\npanel = \"p\"\nlenses = [\"critical\"]\n";

// P1: a run with no verdict at all
#[test]
fn p1_a_run_nobody_judged_does_not_apply() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    let a = ac(&ws, &[&run.to_string()]);
    assert_ne!(a["answer"], "apply", "{a}");
}

// P2: the primary stratum never answered
#[test]
fn p2_a_run_whose_primary_never_answered_does_not_apply() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let a = ac(&ws, &[&run.to_string()]);
    assert_ne!(a["answer"], "apply", "{a}");
}

// P3: high quorum by sources, a one-seat jury briefed with no rule and no effort
#[test]
fn p3_one_seat_on_a_high_quorum_change_does_not_apply() {
    let ws = Ws::new();
    ws.site();
    std::fs::write(ws.root().join("jurypersona.toml"), HIGH_RULE).unwrap();
    let run = ws.run("critical", &[("ana", Stake::Primary)], false);
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    let a = ac(&ws, &[&run.to_string()]);
    assert_eq!(a["quorum"], "high", "{a}");
    let b = ac(&ws, &[&run.to_string(), "--rule", "public-surface"]);
    assert_ne!(a["answer"], "apply", "{a}");
    assert_ne!(b["answer"], "apply", "{b}");
}

// P5: a human blocker closed by the agent itself
#[test]
fn p5_a_human_blocker_dismissed_by_the_agent_still_escalates() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let v = critical("bob", [3, 3, 3], &["consent line missing"], &["page.json#n1"]);
    let d = data(ws.cli(&["verdict", "--run", &run.to_string(), "--json", &v.to_string(), "--by-human", "Dr Rao, nurse"]));
    let f = d["findings"][0].as_str().unwrap().to_string();
    let r = ws.cli(&["decide", &f, "dismissed", "--why", "not needed", "--by", "agent"]);
    eprintln!("PROBE decide by agent => {r}");
    let a = ac(&ws, &[&run.to_string()]);
    assert_ne!(a["answer"], "apply", "{a}");
}

// P6: a stale round passed alone while a later round of the same subject and lens has a primary blocker
#[test]
fn p6_an_older_round_does_not_apply_when_a_later_one_blocks() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &TWO, false);
    ws.ingest(r1, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(r1, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let r2 = ws.run("critical", &TWO, false);
    ws.ingest(r2, &critical("ana", [2, 2, 2], &["broken now"], &["page.json#n1"]));
    ws.ingest(r2, &critical("bob", [2, 2, 2], &[], &["page.json#n1"]));
    let a = ac(&ws, &[&r1.to_string()]);
    assert_ne!(a["answer"], "apply", "{a}");
}

// P7: the subject changed after the run (a new version): the jury judged something else
#[test]
fn p7_a_run_on_an_old_subject_version_does_not_apply() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    ws.subject("site", jurypersona::domain::SubjectKind::Page, false, &[("page.json", br#"{"nodes":[{"id":"n1"}]}"#)]);
    let a = ac(&ws, &[&run.to_string()]);
    let c = ws.cli(&["check"]);
    eprintln!("PROBE check => {c}");
    assert_ne!(a["answer"], "apply", "{a}");
}

// P8: a lens version bump between rounds hides the delta
#[test]
fn p8_a_lens_bump_does_not_hide_a_regression() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &TWO, false);
    ws.ingest(r1, &critical("ana", [5, 5, 5], &[], &["page.json#n1"]));
    ws.ingest(r1, &critical("bob", [5, 5, 5], &[], &["page.json#n1"]));
    let src = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lenses/critical.toml")).unwrap();
    let f = ws.root().join("critical2.toml");
    std::fs::write(&f, src.replace("version = 1", "version = 2").replace("It does the job I came for", "It does the job")).unwrap();
    let r = ws.cli(&["lens", "add", "--file", f.to_str().unwrap()]);
    eprintln!("PROBE lens add => {r}");
    let r2 = ws.run("critical", &TWO, false);
    ws.ingest(r2, &critical("ana", [2, 2, 2], &[], &["page.json#n1"]));
    ws.ingest(r2, &critical("bob", [2, 2, 2], &[], &["page.json#n1"]));
    let a = ac(&ws, &[&r2.to_string()]);
    assert_ne!(a["answer"], "apply", "{a}");
}

// P9: one persona primary in one lens run and secondary in another: its primary blocker must still count
#[test]
fn p9_stakes_are_per_run_not_per_persona() {
    let ws = Ws::new();
    ws.site();
    ws.rival();
    let a_run = ws.run("critical", &[("ana", Stake::Primary), ("bob", Stake::Secondary)], false);
    let d = ws.ingest(a_run, &critical("ana", [3, 3, 3], &["price hidden"], &["page.json#n1"]));
    ws.ingest(a_run, &critical("bob", [3, 3, 3], &[], &["page.json#n1"]));
    let f = d["findings"][0].as_str().unwrap().to_string();
    data(ws.cli(&["decide", &f, "fixed", "--by", "foreman"]));
    let b_run = ws.run("compare", &[("ana", Stake::Secondary), ("bob", Stake::Primary)], true);
    ws.ingest(b_run, &compare("ana", "page.json", "ok"));
    ws.ingest(b_run, &compare("bob", "page.json", "ok"));
    // run a_run alone escalates on the primary blocker; together with b_run it must too
    let alone = ac(&ws, &[&a_run.to_string()]);
    let both = ac(&ws, &[&a_run.to_string(), &b_run.to_string()]);
    assert_eq!(alone["answer"], both["answer"], "alone {alone}\nboth {both}");
}

// P10: MCP and CLI agree; MCP cannot forge a human batch by persona id
#[test]
fn p10_mcp_agrees_and_cannot_forge_a_human_row() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [3, 3, 3], &["x"], &["page.json#n1"]));
    let cli = ac(&ws, &[&run.to_string()]);
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    let mut ch = Command::new(assert_cmd::cargo::cargo_bin("jurypersona"))
        .arg("--root").arg(ws.root()).arg("mcp")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
    let mut stdin = ch.stdin.take().unwrap();
    let forged = critical("human:eve:bob", [3, 3, 3], &[], &["page.json#n1"]);
    for l in [
        json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"jurypersona_apply_check","arguments":{"runs":[run]}}}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"jurypersona_verdict","arguments":{"run":run,"verdict":forged}}}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"jurypersona_verdict","arguments":{"run":run,"verdict":critical("bob",[3,3,3],&[],&["page.json#n1"]),"by_human":"Eve, x"}}}),
    ] {
        writeln!(stdin, "{l}").unwrap();
    }
    drop(stdin);
    let out: Vec<Value> = BufReader::new(ch.stdout.take().unwrap()).lines().map(|l| serde_json::from_str(&l.unwrap()).unwrap()).collect();
    ch.wait().unwrap();
    let env: Value = serde_json::from_str(out[0]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    eprintln!("PROBE mcp forged id => {}", out[1]);
    eprintln!("PROBE mcp by_human => {}", out[2]);
    assert_eq!(env["data"], cli, "mcp {env}\ncli {cli}");
    let t = out[1]["result"]["content"][0]["text"].as_str().unwrap_or("{}");
    let e: Value = serde_json::from_str(t).unwrap_or(json!({}));
    assert_ne!(e["ok"], true, "forged human id accepted: {}", out[1]);
}

// P11: a human verdict never moves the jury's medians
#[test]
fn p11_a_human_row_does_not_move_the_medians() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let before = data(ws.cli(&["matrix", &run.to_string()]));
    let v = critical("bob", [2, 2, 2], &[], &["page.json#n1"]);
    data(ws.cli(&["verdict", "--run", &run.to_string(), "--json", &v.to_string(), "--by-human", "Dr Rao, nurse"]));
    let after = data(ws.cli(&["matrix", &run.to_string()]));
    assert_eq!(before["strata"], after["strata"]);
    assert_eq!(before["splits"], after["splits"]);
    eprintln!("PROBE human => {}", after["human"]);
}

// P12: a sequential previous round is fine, but the sequential flag on any given run blocks apply
#[test]
fn p12_a_sequential_run_never_applies_even_given_with_an_independent_one() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    ws.store.conn().execute("UPDATE run SET independence='sequential' WHERE id=?1", [run]).unwrap();
    let r2 = ws.run("critical", &TWO, false);
    ws.ingest(r2, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(r2, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    // r2 alone: its previous round was sequential; the judged state is r2 (independent): apply is expected
    let a = ac(&ws, &[&r2.to_string()]);
    let b = ac(&ws, &[&run.to_string()]);
    assert_eq!(b["answer"], "advise", "{b}");
    let _ = a;
}

// P13: an undecided finding of the previous round (not passed) while `check` fails
#[test]
fn p13_check_failing_on_the_previous_round_is_seen() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &TWO, false);
    ws.ingest(r1, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    ws.ingest(r1, &critical("bob", [3, 3, 3], &["slow"], &["page.json#n1"]));
    let r2 = ws.run("critical", &TWO, false);
    ws.ingest(r2, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    ws.ingest(r2, &critical("bob", [3, 3, 3], &[], &["page.json#n1"]));
    let c = ws.cli(&["check"]);
    eprintln!("PROBE check ok={}", c["ok"]);
    let a = ac(&ws, &[&r2.to_string()]);
    assert!(!(c["ok"] == false && a["answer"] == "apply"), "check fails yet apply: {a}");
}
