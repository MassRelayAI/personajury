//! `brief --seats auto`: seat the personas that serve the subject's audience, plus exactly one control (adopter feedback).
#[path = "traits_support/mod.rs"]
mod support;
use serde_json::{json, Value};
use std::path::Path;
use support::*;

fn persona(w: &Path, id: &str, serves: &[&str], traits: Value) {
    let mut b = persona_body(id, &["x"], traits);
    b["serves"] = json!(serves);
    ok(add_persona(w, &b));
}
fn subject(w: &Path, audience: Option<&str>) {
    let f = w.join("page.md");
    std::fs::write(&f, "# page\nline\n").unwrap();
    let mut a = vec!["subject", "add", "s", "--kind", "document", "--attach", f.to_str().unwrap()];
    if let Some(x) = audience {
        a.extend(["--audience", x]);
    }
    ok(jg(w, &a));
}
fn auto(w: &Path, extra: &[&str]) -> Value {
    let mut a = vec!["brief", "--subject", "s", "--seats", "auto", "--lens", "critical", "--model", "m"];
    a.extend(extra);
    jg(w, &a)
}
fn world() -> tempfile::TempDir {
    let ws = installed();
    let w = ws.path();
    for d in ["d1", "d2", "d3", "d4", "d5"] {
        persona(w, d, &["docs"], Value::Null);
    }
    persona(w, "m1", &["marketing"], Value::Null);
    persona(w, "m2", &["marketing"], Value::Null);
    persona(w, "plain", &[], Value::Null);
    subject(w, Some("docs, Developers"));
    ws
}
fn seats(d: &Value) -> Vec<(String, String)> {
    d["seats"].as_array().unwrap().iter().map(|s| (s["persona"].as_str().unwrap().into(), s["why"].as_str().unwrap().into())).collect()
}

#[test]
fn seats_every_serving_persona_and_exactly_one_control() {
    let ws = world();
    let d = ok(auto(ws.path(), &[]));
    let s = seats(&d);
    assert_eq!(s.len(), 6, "{s:?}");
    for id in ["d1", "d2", "d3", "d4", "d5"] {
        assert!(s.contains(&(id.to_string(), "serves docs".to_string())), "{s:?}");
    }
    let control: Vec<_> = s.iter().filter(|(_, w)| w == "control: unrelated").collect();
    assert_eq!(control.len(), 1);
    assert!(["m1", "m2", "plain"].contains(&control[0].0.as_str()));
    // serving seats are primary, the control a bystander
    let stakes: Vec<(&str, &str)> = d["briefs"].as_array().unwrap().iter().map(|b| (b["persona"].as_str().unwrap(), b["stake"].as_str().unwrap())).collect();
    assert!(stakes.contains(&("d1", "primary")) && stakes.contains(&(control[0].0.as_str(), "bystander")));
    // recorded on the run, and the same draw twice
    let rec = personajury::store::Store::open(ws.path()).unwrap().run_traits(d["run"].as_i64().unwrap()).unwrap().1.unwrap();
    assert_eq!(rec, d["seats"]);
    assert_eq!(seats(&ok(auto(ws.path(), &[]))), s);
    assert_eq!(d["panel"]["name"], "(auto)");
}

#[test]
fn effort_caps_the_whole_jury_and_the_control_sits_inside_the_cap() {
    let ws = world();
    let d = ok(auto(ws.path(), &["--effort", "low"]));
    let s = seats(&d);
    assert_eq!(s.iter().filter(|(_, w)| w.starts_with("serves")).count(), 2);
    assert_eq!(s.iter().filter(|(_, w)| w.starts_with("control")).count(), 1);
}

#[test]
fn the_trait_cover_picks_within_the_matched_set() {
    let ws = installed();
    let w = ws.path();
    for (i, v) in [1, 1, 1, 5, 5, 5].iter().enumerate() {
        persona(w, &format!("e{i}"), &["docs"], json!({"expertise.domain": v}));
    }
    persona(w, "other", &["marketing"], json!({"expertise.domain": 3}));
    subject(w, Some("docs"));
    let d = ok(auto(w, &["--effort", "low", "--traits", "docs"]));
    let row = d["coverage"]["one_way"].as_array().unwrap().iter().find(|r| r["trait"] == "expertise.domain" && r["stratum"] == "primary").unwrap();
    assert!(row["seated"]["low"].as_i64().unwrap() >= 1 && row["seated"]["high"].as_i64().unwrap() >= 1, "{row}");
    let rec = personajury::store::Store::open(w).unwrap().run_traits(d["run"].as_i64().unwrap()).unwrap().0.unwrap();
    assert_eq!(rec["draw"], "cover");
}

#[test]
fn the_refusals() {
    let ws = world();
    let w = ws.path();
    // no audience on the subject
    let f = w.join("other.md");
    std::fs::write(&f, "# o\n").unwrap();
    ok(jg(w, &["subject", "add", "bare", "--kind", "document", "--attach", f.to_str().unwrap()]));
    let v = jg(w, &["brief", "--subject", "bare", "--seats", "auto", "--lens", "critical", "--model", "m"]);
    assert_eq!(code(&v), "audience_unknown");
    // nobody serves it
    let f = w.join("legal.md");
    std::fs::write(&f, "# l\n").unwrap();
    ok(jg(w, &["subject", "add", "legal", "--kind", "document", "--attach", f.to_str().unwrap(), "--audience", "legal"]));
    let v = jg(w, &["brief", "--subject", "legal", "--seats", "auto", "--lens", "critical", "--model", "m"]);
    assert_eq!(code(&v), "no_matching_personas");
    // it picks its own seats
    assert_eq!(code(&auto(w, &["--query", "1 x=primary"])), "invalid_arguments");
    let v = jg(w, &["brief", "--subject", "s", "--seats", "some", "--lens", "critical", "--model", "m"]);
    assert_eq!(code(&v), "invalid_arguments");
}

#[test]
fn with_no_unrelated_persona_no_control_sits_and_it_says_so() {
    let ws = installed();
    persona(ws.path(), "d1", &["docs"], Value::Null);
    subject(ws.path(), Some("docs"));
    let d = ok(auto(ws.path(), &[]));
    assert_eq!(seats(&d).len(), 1);
    assert_eq!(d["warnings"][0]["code"], "no_control_persona");
}

#[test]
fn under_a_rule_it_is_no_override_by_itself() {
    let ws = world();
    let w = ws.path();
    ok(jg(w, &["panel", "add", "p", "--query", "2 x=primary"]));
    std::fs::write(w.join("personajury.toml"), "[[rule]]\nclass = \"docs\"\npaths = [\"page.md\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n").unwrap();
    let d = ok(jg(w, &["brief", "--subject", "s", "--rule", "docs", "--seats", "auto", "--model", "m"]));
    assert!(d.get("override").is_none_or(serde_json::Value::is_null), "{}", d["override"]);
}
