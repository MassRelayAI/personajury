//! D35: `trait suggest --persona` writes a brief from the persona's prose; `--check` verifies the answer and writes nothing.
#[path = "traits_support/mod.rs"]
mod support;
use serde_json::{json, Value};
use support::*;

fn prose_persona(ws: &std::path::Path, traits: Value) {
    let mut b = persona_body("builder-ravi", &["builder"], traits);
    b["role"] = json!("A backend engineer who evaluates tools for the team");
    b["context"] = json!("Arrives from a search\nfor a   docs generator, with ten minutes.");
    b["daily_tasks"] = json!("Reviews pull requests and reads changelogs.");
    b["fears"] = json!("Adopting a tool that is dropped next year.");
    b["never_reads"] = json!("Testimonials and logo walls.");
    b["decides_by"] = json!("Reads the pricing page first and checks the licence.");
    b["stake_rules"] = json!("Primary when the product is a developer tool.");
    ok(add_persona(ws, &b));
}
fn brief(ws: &std::path::Path, d: &Value) -> String {
    std::fs::read_to_string(ws.join(d["path"].as_str().unwrap())).unwrap()
}
fn write(ws: &std::path::Path, name: &str, v: &Value) -> String {
    let f = ws.join(name);
    std::fs::write(&f, v.to_string()).unwrap();
    f.to_string_lossy().into_owned()
}
fn check(ws: &std::path::Path, v: &Value) -> Value {
    let f = write(ws, "sug.json", v);
    jg(ws, &["trait", "suggest", "--check", &f])
}

#[test]
fn the_brief_lays_the_prose_beside_the_candidates_and_is_deterministic() {
    let ws = installed();
    prose_persona(ws.path(), Value::Null);
    let d = ok(jg(ws.path(), &["trait", "suggest", "--persona", "builder-ravi", "--set", "docs"]));
    let p = d["path"].as_str().unwrap();
    assert!(p.starts_with(".personajury/generate/trait-suggest-builder-ravi-") && p.ends_with(".md"), "{p}");
    let h8 = p.trim_end_matches(".md").rsplit('-').next().unwrap();
    assert_eq!(h8.len(), 8);
    assert!(d["hash"].as_str().unwrap().len() == 64 && d["bytes"].as_u64().unwrap() > 500);
    let t = brief(ws.path(), &d);
    assert!(t.starts_with("personajury-trait-suggest 1\n"));
    assert!(t.contains("# Traits for Person builder-ravi (builder-ravi), from their own words"));
    assert!(t.contains("## The persona's prose"));
    assert!(t.contains("- Role: A backend engineer who evaluates tools for the team"));
    assert!(t.contains("- Decides by: Reads the pricing page first and checks the licence."));
    assert!(t.contains("- Stake rules: Primary when the product is a developer tool."));
    assert!(t.contains("## Candidate traits"));
    assert!(t.contains("### expertise.domain \u{2014} "), "{t}");
    assert!(t.contains("(band 1\u{2013}5)") && t.contains("(category"));
    assert!(t.contains("- 1: ") && t.contains("- 5: "));
    assert!(!t.contains("trust.marketing_skepticism"), "only the set's traits are candidates");
    assert!(t.contains("## Your task") && t.contains("At most 8") && t.contains("\"evidence\""));
    // the same store gives the same bytes; another set gives another file
    let again = ok(jg(ws.path(), &["trait", "suggest", "--persona", "builder-ravi", "--set", "docs"]));
    assert_eq!(again["hash"], d["hash"]);
    assert_eq!(again["path"], d["path"]);
    let all = ok(jg(ws.path(), &["trait", "suggest", "--persona", "builder-ravi"]));
    assert_ne!(all["path"], d["path"]);
    assert!(brief(ws.path(), &all).contains("trust.marketing_skepticism"), "without --set every trait in the store");
}

#[test]
fn a_changed_persona_is_a_new_hash8() {
    let ws = installed();
    prose_persona(ws.path(), Value::Null);
    let a = ok(jg(ws.path(), &["trait", "suggest", "--persona", "builder-ravi"]));
    prose_persona(ws.path(), json!({"trust.marketing_skepticism": 5}));
    let b = ok(jg(ws.path(), &["trait", "suggest", "--persona", "builder-ravi"]));
    assert_ne!(a["path"], b["path"]);
}

#[test]
fn brief_refusals() {
    let ws = installed();
    prose_persona(ws.path(), Value::Null);
    assert_eq!(code(&jg(ws.path(), &["trait", "suggest", "--persona", "builder-ravi", "--set", "nope"])), "trait_set_unknown");
    assert_eq!(code(&jg(ws.path(), &["trait", "suggest", "--persona", "ghost"])), "not_found");
    assert_eq!(code(&jg(ws.path(), &["trait", "suggest"])), "invalid_arguments");
}

fn good() -> Value {
    json!({"persona": "builder-ravi",
           "traits": {"consumer.price_consciousness": 5, "trust.marketing_skepticism": 4},
           "evidence": {"consumer.price_consciousness": "Reads the pricing page first",
                        "trust.marketing_skepticism": "Testimonials  and\nlogo walls."}})
}

#[test]
fn check_answers_the_merged_persona_file_and_writes_nothing() {
    let ws = installed();
    prose_persona(ws.path(), json!({"situation.time_pressure": 4, "trust.marketing_skepticism": 1}));
    let before = ok(jg(ws.path(), &["persona", "list"]));
    let d = ok(check(ws.path(), &good()));
    let pf = &d["persona_file"];
    assert_eq!(pf["id"], "builder-ravi");
    assert_eq!(pf["traits"]["consumer.price_consciousness"], 5);
    assert_eq!(pf["traits"]["trust.marketing_skepticism"], 4, "a suggested trait replaces the stored value");
    assert_eq!(pf["traits"]["situation.time_pressure"], 4, "a trait not suggested is kept");
    assert_eq!(pf["decides_by"], "Reads the pricing page first and checks the licence.");
    assert_eq!(d["evidence"]["consumer.price_consciousness"], "Reads the pricing page first");
    assert_eq!(ok(jg(ws.path(), &["persona", "list"])), before, "nothing was written");
    // the caller saves the file and adds it: a new version
    let f = write(ws.path(), "ravi.json", pf);
    let a = ok(jg(ws.path(), &["persona", "add", "--file", &f]));
    assert_eq!(a["version"], 2);
    // a persona the file says nothing new about gives the same hash as a second check of the same answer
    let again = ok(check(ws.path(), &good()));
    assert_eq!(again["persona_file"], *pf);
}

#[test]
fn a_persona_without_traits_gets_the_suggested_map() {
    let ws = installed();
    prose_persona(ws.path(), Value::Null);
    let d = ok(check(ws.path(), &good()));
    assert_eq!(d["persona_file"]["traits"].as_object().unwrap().len(), 2);
}

#[test]
fn check_refusals_each_have_a_code() {
    let ws = installed();
    prose_persona(ws.path(), Value::Null);
    let with = |f: &dyn Fn(&mut Value)| {
        let mut v = good();
        f(&mut v);
        code(&check(ws.path(), &v))
    };
    assert_eq!(with(&|v| v["persona"] = json!("someone-else")), "not_found");
    assert_eq!(with(&|v| v["traits"]["nope.nothing"] = json!(3)), "unknown_trait");
    assert_eq!(with(&|v| v["traits"]["consumer.price_consciousness"] = json!(9)), "trait_value_off_scale");
    assert_eq!(with(&|v| v["traits"]["choice.decision_style"] = json!("moody")), "trait_option_unknown");
    assert_eq!(with(&|v| v["traits"]["x.gender"] = json!(3)), "unknown_trait", "labelled, not refused (D39), when held");
    assert_eq!(with(&|v| v["traits"]["culture.power_distance"] = json!(3)), "refused_trait_id");
    assert_eq!(
        with(&|v| {
            for i in 0..8 {
                v["traits"][format!("t.t{i}")] = json!(3);
            }
        }),
        "traits_too_many"
    );
    assert_eq!(with(&|v| { v["evidence"].as_object_mut().unwrap().remove("trust.marketing_skepticism"); }), "suggestion_without_quote");
    assert_eq!(with(&|v| v["evidence"]["trust.marketing_skepticism"] = json!("   ")), "suggestion_without_quote");
    let c = with(&|v| v["evidence"]["trust.marketing_skepticism"] = json!("Distrusts every claim by default"));
    assert_eq!(c, "quote_not_found");
    let e = check(ws.path(), &json!({"persona": "builder-ravi", "traits": {"trust.marketing_skepticism": 4},
        "evidence": {"trust.marketing_skepticism": "Distrusts every claim"}}));
    assert_eq!(e["data"]["id"], "trust.marketing_skepticism");
    assert_eq!(e["data"]["quote"], "Distrusts every claim");
    // a quote is case-sensitive: verbatim
    assert_eq!(with(&|v| v["evidence"]["trust.marketing_skepticism"] = json!("testimonials and logo walls")), "quote_not_found");
}

#[test]
fn check_persona_flag_must_match_the_file() {
    let ws = installed();
    prose_persona(ws.path(), Value::Null);
    let f = write(ws.path(), "sug.json", &good());
    let v = jg(ws.path(), &["trait", "suggest", "--check", &f, "--persona", "other"]);
    assert_eq!(code(&v), "invalid_arguments");
    ok(jg(ws.path(), &["trait", "suggest", "--check", &f, "--persona", "builder-ravi"]));
}
