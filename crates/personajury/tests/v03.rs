//! v0.3 (D11, D12): waves, reported usage, the allowed anchors in the brief, a reason for any large brief, a
//! comma list of comparators. Every refusal here was seen red first.
use assert_cmd::Command;
use jurygraph::store::Store;
use serde_json::{json, Value};
use std::path::Path;

fn jg(ws: &Path, args: &[&str]) -> Value {
    let out = Command::cargo_bin("jurygraph")
        .unwrap()
        .arg("--root")
        .arg(ws)
        .args(args)
        .output()
        .unwrap();
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!("{args:?}: not json ({e}): {}", String::from_utf8_lossy(&out.stdout))
    })
}
fn ok(v: Value) -> Value {
    assert_eq!(v["ok"], true, "{v}");
    v["data"].clone()
}
fn code(v: &Value) -> &str {
    assert_eq!(v["ok"], false, "{v}");
    v["error"]["code"].as_str().unwrap()
}
/// `n` personas `<prefix>-000 …` all tagged `tag`.
fn pool(ws: &Path, prefix: &str, tag: &str, n: usize) {
    let dir = ws.join("pool");
    std::fs::create_dir_all(&dir).unwrap();
    for i in 0..n {
        let id = format!("{prefix}-{i:03}");
        let f = dir.join(format!("{id}.json"));
        let body = json!({
            "id": id, "name": format!("Person {id}"), "tags": [tag],
            "role": "r", "context": "c", "daily_tasks": "d", "fears": "f",
            "never_reads": "n", "decides_by": "b", "stake_rules": "s"
        });
        std::fs::write(&f, body.to_string()).unwrap();
        ok(jg(ws, &["persona", "add", "--file", f.to_str().unwrap()]));
    }
}
fn doc(ws: &Path, name: &str) {
    let dir = ws.join("src");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(format!("{name}.md"));
    std::fs::write(&f, format!("# {name}\nline two\nline three\n")).unwrap();
    ok(jg(ws, &["subject", "add", name, "--kind", "document", "--attach", f.to_str().unwrap()]));
}
fn brief(w: &Path, extra: &[&str]) -> Value {
    let mut a = vec!["brief", "--subject", "pricing", "--lens", "recall", "--model", "m"];
    a.extend_from_slice(extra);
    jg(w, &a)
}
fn run_row(w: &Path, id: i64) -> jurygraph::domain::RunRow {
    Store::open_existing(w).unwrap().run(id).unwrap().unwrap()
}
fn sized(n: usize) -> tempfile::TempDir {
    let ws = tempfile::tempdir().unwrap();
    pool(ws.path(), "rdr", "reader", n);
    doc(ws.path(), "pricing");
    ws
}
fn lens_sizes(waves: &Value) -> Vec<usize> {
    waves.as_array().unwrap().iter().map(|w| w.as_array().unwrap().len()).collect()
}

// ---- --comparator takes a comma list, as --lens does ----

#[test]
fn comparator_takes_a_comma_list_like_lens() {
    let ws = sized(2);
    let w = ws.path();
    for n in ["acme", "bolt"] {
        let f = w.join(format!("{n}.md"));
        std::fs::write(&f, format!("# {n}\n")).unwrap();
        ok(jg(w, &["comparator", "add", n, "--kind", "prices", "--attach", f.to_str().unwrap()]));
    }
    let d = ok(brief(w, &["--query", "2 reader=bystander", "--comparator", "acme,bolt"]));
    let r = run_row(w, d["run"].as_i64().unwrap());
    assert_eq!(r.comparators.len(), 2, "{d}");
    // the repeated form still works
    let d = ok(brief(w, &["--query", "2 reader=bystander", "--comparator", "acme", "--comparator", "bolt"]));
    assert_eq!(run_row(w, d["run"].as_i64().unwrap()).comparators.len(), 2);
}

// ---- item 6: a large brief needs a reason, rule or not ----

#[test]
fn a_large_brief_without_a_rule_needs_a_reason_and_records_it() {
    let ws = sized(14);
    let w = ws.path();
    let r = brief(w, &["--query", "13 reader=bystander", "--allow-large"]);
    assert_eq!(code(&r), "override_without_reason", "{r}");
    assert!(r["error"]["message"].as_str().unwrap().contains("seats"), "{r}");
    // nothing was written for the refused brief
    assert!(Store::open_existing(w).unwrap().runs().unwrap().is_empty());
    let d = ok(brief(w, &["--query", "13 reader=bystander", "--allow-large", "--reason", "a wide read"]));
    assert_eq!(
        d["override"],
        json!({"rule": null, "fields": {"seats": {"rule": 12, "given": 13}}, "reason": "a wide read", "by": "agent"}),
        "{d}"
    );
    let row = run_row(w, d["run"].as_i64().unwrap());
    assert_eq!(row.override_json.unwrap(), d["override"]);
    // twelve seats and under: no reason, no record
    let d = ok(brief(w, &["--query", "12 reader=bystander"]));
    assert!(d["override"].is_null(), "{d}");
}

#[test]
fn the_matrix_names_a_large_brief_with_no_rule() {
    let ws = sized(14);
    let w = ws.path();
    let d = ok(brief(w, &["--query", "13 reader=bystander", "--allow-large", "--reason", "a wide read"]));
    let run = d["run"].as_i64().unwrap().to_string();
    let m = ok(jg(w, &["matrix", &run, "--table"]));
    assert_eq!(m["override"]["rule"], Value::Null, "{m}");
    let t = m["table"].as_str().unwrap();
    assert!(t.starts_with("override: seats 12\u{2192}13"), "{t}");
    assert!(t.lines().next().unwrap().ends_with("reason: a wide read"), "{t}");
}

#[test]
fn a_malformed_toml_is_refused_by_brief_not_ignored() {
    let ws = sized(2);
    let w = ws.path();
    std::fs::write(w.join("jurygraph.toml"), "[[rule]\nclass = ").unwrap();
    let r = brief(w, &["--query", "2 reader=bystander"]);
    assert_eq!(code(&r), "invalid_config", "{r}");
    std::fs::remove_file(w.join("jurygraph.toml")).unwrap();
    ok(brief(w, &["--query", "2 reader=bystander"]));
}

// ---- item 2: waves ----

#[test]
fn a_hundred_seats_come_in_waves_of_thirty_in_persona_order() {
    let ws = sized(100);
    let w = ws.path();
    let d = ok(brief(w, &["--query", "100 reader=bystander", "--allow-large", "--reason", "whole room"]));
    assert_eq!(lens_sizes(&d["waves"]), vec![30, 30, 30, 10], "{}", d["waves"]);
    let flat: Vec<&str> = d["waves"].as_array().unwrap().iter().flat_map(|w| w.as_array().unwrap()).map(|p| p.as_str().unwrap()).collect();
    let briefed: Vec<&str> = d["briefs"].as_array().unwrap().iter().map(|b| b["persona"].as_str().unwrap()).collect();
    assert_eq!(flat, briefed, "waves keep the order of the briefs");
    let d = ok(brief(w, &["--query", "100 reader=bystander", "--allow-large", "--reason", "x", "--wave", "40"]));
    assert_eq!(lens_sizes(&d["waves"]), vec![40, 40, 20]);
}

#[test]
fn twelve_seats_and_under_are_one_wave() {
    let ws = sized(12);
    let d = ok(brief(ws.path(), &["--query", "12 reader=bystander"]));
    assert_eq!(lens_sizes(&d["waves"]), vec![12], "{d}");
    let d = ok(brief(ws.path(), &["--query", "5 reader=bystander"]));
    assert_eq!(lens_sizes(&d["waves"]), vec![5], "{d}");
}

#[test]
fn a_wave_of_zero_is_refused() {
    let ws = sized(3);
    let r = brief(ws.path(), &["--query", "3 reader=bystander", "--wave", "0"]);
    assert_eq!(code(&r), "invalid_arguments", "{r}");
}

// ---- item 5: the allowed anchor_met values are in the brief ----

#[test]
fn the_brief_states_the_allowed_anchor_values() {
    let ws = sized(1);
    let d = ok(brief(ws.path(), &["--query", "1 reader=bystander"]));
    let p = ws.path().join(d["briefs"][0]["path"].as_str().unwrap());
    let text = std::fs::read_to_string(p).unwrap();
    assert!(text.contains("`anchor_met`: one of 1, 3, 5, at or below your `score`."), "{text}");
}

// ---- item 3: reported usage ----

fn verdict_json(persona: &str, usage: Option<Value>) -> String {
    let mut v = json!({"persona": persona, "lines": [
        {"dimension": "first-read", "score": 3, "anchor_met": 3, "evidence": "recalled"},
        {"dimension": "would-return", "score": 3, "anchor_met": 3, "evidence": "recalled"}]});
    if let Some(u) = usage {
        v["usage"] = u;
    }
    v.to_string()
}

#[test]
fn a_bad_usage_is_refused_as_invalid_usage() {
    let ws = sized(1);
    let w = ws.path();
    let d = ok(brief(w, &["--query", "1 reader=bystander"]));
    let run = d["run"].as_i64().unwrap().to_string();
    for bad in [
        json!({"input_tokens": -1, "output_tokens": 5}),
        json!({"input_tokens": 1.5, "output_tokens": 5}),
        json!({"input_tokens": "10", "output_tokens": 5}),
        json!({"input_tokens": 10}),
        json!({"input_tokens": 10, "output_tokens": 5, "cost": 1}),
        json!(12),
    ] {
        let r = jg(w, &["verdict", "--run", &run, "--json", &verdict_json("rdr-000", Some(bad.clone()))]);
        assert_eq!(code(&r), "invalid_usage", "{bad}: {r}");
    }
    // none of them stored a line
    let m = ok(jg(w, &["matrix", &run]));
    assert_eq!(m["strata"], json!([]), "nothing was ingested: {m}");
}

#[test]
fn reported_usage_shows_beside_the_estimate_in_matrix_and_doctor() {
    let ws = sized(2);
    let w = ws.path();
    let d = ok(brief(w, &["--query", "2 reader=bystander"]));
    let run = d["run"].as_i64().unwrap().to_string();
    // no usage reported yet
    let m = ok(jg(w, &["matrix", &run]));
    assert_eq!(m["usage"]["reported"], Value::Null, "{m}");
    assert_eq!(m["usage"]["estimate"], json!({"briefs": 2, "tokens": 30000}), "{m}");
    ok(jg(w, &["verdict", "--run", &run, "--json", &verdict_json("rdr-000", Some(json!({"input_tokens": 1200, "output_tokens": 300})))]));
    ok(jg(w, &["verdict", "--run", &run, "--json", &verdict_json("rdr-001", Some(json!({"input_tokens": 800, "output_tokens": 200})))]));
    let m = ok(jg(w, &["matrix", &run, "--table"]));
    assert_eq!(
        m["usage"]["reported"],
        json!({"input_tokens": 2000, "output_tokens": 500, "verdicts": 2, "by": "reported by the agent"}),
        "{m}"
    );
    assert_eq!(m["usage"]["estimate"], json!({"briefs": 2, "tokens": 30000}));
    assert!(m["table"].as_str().unwrap().contains("reported by the agent"), "{m}");
    let home = tempfile::tempdir().unwrap();
    let out = Command::cargo_bin("jurygraph")
        .unwrap()
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .arg("--root")
        .arg(w)
        .arg("doctor")
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["data"]["usage"]["reported"]["input_tokens"], 2000, "{v}");
    assert_eq!(v["data"]["usage"]["estimate"], json!({"briefs": 2, "tokens": 30000}), "{v}");
}

#[test]
fn a_verdict_without_usage_still_ingests_and_usage_is_not_a_line() {
    let ws = sized(1);
    let w = ws.path();
    let d = ok(brief(w, &["--query", "1 reader=bystander"]));
    let run = d["run"].as_i64().unwrap().to_string();
    let v = ok(jg(w, &["verdict", "--run", &run, "--json", &verdict_json("rdr-000", None)]));
    assert_eq!(v["lines"], 2);
    let m = ok(jg(w, &["matrix", &run]));
    assert_eq!(m["usage"]["reported"], Value::Null);
}

// ---- fix round 1 ----

#[test]
fn usage_above_i64_max_is_refused_and_sums_never_panic() {
    let ws = sized(2);
    let w = ws.path();
    let d = ok(brief(w, &["--query", "2 reader=bystander"]));
    let run = d["run"].as_i64().unwrap().to_string();
    let r = jg(w, &["verdict", "--run", &run, "--json", r#"{"persona":"rdr-000","usage":{"input_tokens":9223372036854775808,"output_tokens":1},"lines":[{"dimension":"first-read","score":3,"anchor_met":3,"evidence":"recalled"},{"dimension":"would-return","score":3,"anchor_met":3,"evidence":"recalled"}]}"#]);
    assert_eq!(code(&r), "invalid_usage", "{r}");
    let max = json!({"input_tokens": i64::MAX, "output_tokens": i64::MAX});
    ok(jg(w, &["verdict", "--run", &run, "--json", &verdict_json("rdr-000", Some(max.clone()))]));
    ok(jg(w, &["verdict", "--run", &run, "--json", &verdict_json("rdr-001", Some(max))]));
    let m = ok(jg(w, &["matrix", &run]));
    assert_eq!(m["usage"]["reported"]["input_tokens"], (i64::MAX as u64) * 2, "{m}");
}

#[test]
fn a_malformed_toml_is_refused_by_matrix_and_diagnosed_by_doctor() {
    let ws = sized(1);
    let w = ws.path();
    let d = ok(brief(w, &["--query", "1 reader=bystander"]));
    let run = d["run"].as_i64().unwrap().to_string();
    std::fs::write(w.join("jurygraph.toml"), "[[rule]\nclass = ").unwrap();
    let r = jg(w, &["matrix", &run]);
    assert_eq!(code(&r), "invalid_config", "{r}");
    let home = tempfile::tempdir().unwrap();
    let out = Command::cargo_bin("jurygraph")
        .unwrap()
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .arg("--root")
        .arg(w)
        .arg("doctor")
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], true, "doctor does not fail outright: {v}");
    let items = v["data"]["limitation_items"].as_array().unwrap();
    assert!(items.iter().any(|i| i["code"] == "invalid_config"), "{v}");
}

#[test]
fn an_empty_comparator_element_is_invalid_arguments() {
    let ws = sized(2);
    let w = ws.path();
    let f = w.join("acme.md");
    std::fs::write(&f, "# a\n").unwrap();
    ok(jg(w, &["comparator", "add", "acme", "--kind", "prices", "--attach", f.to_str().unwrap()]));
    for bad in ["acme,", ",acme", "acme,,acme"] {
        let r = brief(w, &["--query", "2 reader=bystander", "--comparator", bad]);
        assert_eq!(code(&r), "invalid_arguments", "{bad}: {r}");
    }
}
