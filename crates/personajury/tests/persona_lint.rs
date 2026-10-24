//! D36: `persona lint` (near_copy, confounded) and `label_echo` on verdict ingest and in the matrix. Warnings, never refusals.
#[path = "judging_support/mod.rs"]
mod fx;
#[path = "traits_support/mod.rs"]
mod support;
use personajury::domain::Stake;
use serde_json::{json, Value};

fn lint(ws: &std::path::Path, ids: &[&str]) -> Value {
    let mut a = vec!["persona", "lint"];
    a.extend(ids);
    support::ok(support::jg(ws, &a))
}
fn has(d: &Value, w: Value) -> bool {
    d["warnings"].as_array().unwrap().contains(&w)
}

#[test]
fn near_copy_fires_on_the_same_map_and_a_shared_tag_not_on_different_maps() {
    let ws = support::installed();
    let w = ws.path();
    let same = json!({"trust.marketing_skepticism": 5, "situation.time_pressure": 2});
    support::add_ok(w, "a", &["nurse", "night"], same.clone());
    support::add_ok(w, "b", &["nurse"], same.clone());
    support::add_ok(w, "c", &["clerk"], same.clone());
    support::add_ok(w, "d", &["nurse"], json!({"trust.marketing_skepticism": 5, "situation.time_pressure": 3}));
    support::add_ok(w, "e", &["nurse"], Value::Null);
    support::add_ok(w, "f", &["nurse"], Value::Null);
    let d = lint(w, &[]);
    assert!(has(&d, json!({"code": "near_copy", "a": "a", "b": "b"})), "{d}");
    assert!(!has(&d, json!({"code": "near_copy", "a": "a", "b": "c"})), "no shared tag");
    assert!(!has(&d, json!({"code": "near_copy", "a": "a", "b": "d"})), "different maps");
    assert!(!has(&d, json!({"code": "near_copy", "a": "e", "b": "f"})), "two personas without traits are not copies of a trait map");
    // identical tags with no overlap to speak of: both untagged is the same tags
    support::add_ok(w, "g", &[], same.clone());
    support::add_ok(w, "h", &[], same);
    assert!(has(&lint(w, &[]), json!({"code": "near_copy", "a": "g", "b": "h"})), "identical maps and identical (empty) tags");
}

#[test]
fn confounded_needs_two_with_the_tag_in_one_band_and_one_without_it_in_another() {
    let ws = support::installed();
    let w = ws.path();
    let t = "trust.marketing_skepticism";
    support::add_ok(w, "n1", &["nurse"], json!({ t: 5 }));
    support::add_ok(w, "n2", &["nurse"], json!({ t: 4 }));
    support::add_ok(w, "c1", &["clerk"], json!({ t: 1 }));
    let d = lint(w, &[]);
    assert!(has(&d, json!({"code": "confounded", "trait": t, "tag": "nurse", "band": "high"})), "{d}");
    // not with one holder only, nor when the others hold the same band
    assert!(!has(&d, json!({"code": "confounded", "trait": t, "tag": "clerk", "band": "low"})));
    support::add_ok(w, "c1", &["clerk"], json!({ t: 5 }));
    assert!(!has(&lint(w, &[]), json!({"code": "confounded", "trait": t, "tag": "nurse", "band": "high"})));
    // naming a persona narrows the answer to warnings about it
    support::add_ok(w, "c1", &["clerk"], json!({ t: 1 }));
    assert!(has(&lint(w, &["n1"]), json!({"code": "confounded", "trait": t, "tag": "nurse", "band": "high"})));
    assert_eq!(lint(w, &["c1"])["warnings"], json!([]));
    assert_eq!(support::code(&support::jg(w, &["persona", "lint", "ghost"])), "not_found");
}

// ---- label_echo ----
fn world() -> fx::Ws {
    let ws = fx::Ws::new();
    ws.site();
    support::ok(support::jg(ws.root(), &["trait", "install"]));
    for p in ["ana", "bob"] {
        support::add_ok(ws.root(), p, &["t"], json!({"trust.marketing_skepticism": 5}));
    }
    ws
}
fn verdict(persona: &str, note: Option<&str>, blocker: Option<&str>) -> Value {
    let lines: Vec<Value> = ["works", "blocks", "missing"]
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let mut l = json!({"dimension": d, "score": 3, "anchor_met": 3, "evidence": "seen", "citations": ["page.json#n1"]});
            if i == 1 {
                if let Some(n) = note {
                    l["note"] = json!(n);
                }
                if let Some(b) = blocker {
                    l["blockers"] = json!([b]);
                }
            }
            l
        })
        .collect();
    json!({"persona": persona, "lines": lines})
}
/// The run's briefs rendered every trait the persona holds (`label_echo` reads only the traits a brief rendered).
fn briefed_with_traits(ws: &fx::Ws, run: i64) {
    ws.store.set_run_traits(run, Some(&json!({"mode": "all"})), None).unwrap();
}
fn two() -> Vec<(&'static str, Stake)> {
    vec![("ana", Stake::Primary), ("bob", Stake::Primary)]
}

#[test]
fn a_note_that_echoes_the_label_warns_and_the_verdict_is_still_accepted() {
    let ws = world();
    let run = ws.run("critical", &two(), false);
    briefed_with_traits(&ws, run);
    let d = ws.ingest(run, &verdict("ana", Some("As a marketing skeptic I do not believe the headline."), None));
    assert_eq!(d["warnings"], json!([{"code": "label_echo", "line": 2, "trait": "trust.marketing_skepticism"}]));
    assert_eq!(d["lines"], 3, "accepted");
}
#[test]
fn a_blocker_echo_warns_and_so_does_the_title_in_any_case() {
    let ws = world();
    let run = ws.run("critical", &two(), false);
    briefed_with_traits(&ws, run);
    let d = ws.ingest(run, &verdict("ana", None, Some("Marketing Skepticism makes this a no")));
    assert_eq!(d["warnings"][0]["line"], 2);
    let title = "disbelieves advertising and claims by default";
    let d = ws.ingest(run, &verdict("bob", Some(&format!("I {} so no", title.to_uppercase())), None));
    assert_eq!(d["warnings"][0]["trait"], "trust.marketing_skepticism");
}
#[test]
fn a_note_without_the_words_has_no_warnings_key() {
    let ws = world();
    let run = ws.run("critical", &two(), false);
    briefed_with_traits(&ws, run);
    let d = ws.ingest(run, &verdict("ana", Some("The price is hidden behind a form."), None));
    assert!(d.get("warnings").is_none(), "{d}");
}
#[test]
fn matrix_counts_label_echo_per_stratum_and_only_when_there_is_one() {
    let ws = world();
    let run = ws.run("critical", &[("ana", Stake::Primary), ("bob", Stake::Secondary)], false);
    briefed_with_traits(&ws, run);
    ws.ingest(run, &verdict("ana", Some("a marketing skeptic would stop here"), None));
    ws.ingest(run, &verdict("bob", Some("plain note"), None));
    let m = ws.cli(&["matrix", &run.to_string()])["data"].clone();
    let strata = m["strata"].as_array().unwrap();
    let p = strata.iter().find(|s| s["stake"] == "primary").unwrap();
    let s = strata.iter().find(|s| s["stake"] == "secondary").unwrap();
    assert_eq!(p["warnings"], json!({"label_echo": 1}));
    assert!(s.get("warnings").is_none());
}
