//! D29: every refusal of the trait files (§1.4) and of the persona key (§3), by code.
#[path = "traits_support/mod.rs"]
mod support;
use serde_json::json;
use support::*;

const GOOD: &str = r#"id = "trust.sample"
version = 1
family = "trust"
title = "A sample"
shape = "band"
scale = [1, 5]
moves = ["buy"]
evidence = "weak"
source = "own words"
items_copied = false
correlates_with = []
caveats = "none"

[anchors]
"1" = "take every claim at its word, and a logo wall reassures you at once."
"3" = "read a claim and look once for something that backs it, then move on."
"5" = "assume every claim is written to sell and credit only what you can check."
"#;

fn add_file(ws: &std::path::Path, name: &str, body: &str) -> serde_json::Value {
    let f = ws.join(name);
    std::fs::write(&f, body).unwrap();
    jg(ws, &["trait", "add", "--file", f.to_str().unwrap()])
}
fn refused(body: &str) -> String {
    let ws = tempfile::tempdir().unwrap();
    code(&add_file(ws.path(), "t.toml", body))
}

#[test]
fn a_good_file_is_added_and_marked_project_outside_the_fourteen() {
    let ws = tempfile::tempdir().unwrap();
    let d = ok(add_file(ws.path(), "t.toml", GOOD));
    assert_eq!((d["id"].as_str(), d["version"].as_i64(), d["origin"].as_str()), (Some("trust.sample"), Some(1), Some("catalog")));
    let own = GOOD.replace("trust.sample", "acme.sample").replace("family = \"trust\"", "family = \"project.acme\"");
    assert_eq!(ok(add_file(ws.path(), "u.toml", &own))["origin"], "project");
}

#[test]
fn invalid_trait_file_for_every_malformed_shape() {
    let cases: Vec<(&str, String)> = vec![
        ("not toml", "this is = = not toml".into()),
        ("bad id", GOOD.replace("trust.sample", "Trust-Sample")),
        ("family mismatch", GOOD.replace("family = \"trust\"", "family = \"risk\"")),
        ("bad shape", GOOD.replace("band", "scale5")),
        ("missing anchor", GOOD.replace("\"3\" = \"read a claim and look once for something that backs it, then move on.\"\n", "")),
        ("wrong scale", GOOD.replace("[1, 5]", "[1, 7]")),
        ("short sentence", GOOD.replace("take every claim at its word, and a logo wall reassures you at once.", "take it.")),
        ("upper-case start", GOOD.replace("take every claim", "Take every claim")),
        ("agree word", GOOD.replace("take every claim at its word", "agree with every claim at its word")),
        ("items copied true", GOOD.replace("items_copied = false", "items_copied = true")),
        ("items copied missing", GOOD.replace("items_copied = false\n", "")),
        ("bad evidence", GOOD.replace("\"weak\"", "\"proven\"")),
        ("bad move", GOOD.replace("[\"buy\"]", "[\"win\"]")),
        ("self correlation", GOOD.replace("correlates_with = []", "correlates_with = [\"trust.sample\"]")),
        ("version zero", GOOD.replace("version = 1", "version = 0")),
        ("category with one option", GOOD.replace("shape = \"band\"\nscale = [1, 5]", "shape = \"category\"").split("[anchors]").next().unwrap().to_string()
            + "[[option]]\nid = \"a\"\ntitle = \"A\"\nsentence = \"search for the alternatives and weigh them against each other.\"\n"),
    ];
    for (what, body) in cases {
        assert_eq!(refused(&body), "invalid_trait_file", "{what}");
    }
}

#[test]
fn a_category_with_a_duplicate_option_is_refused() {
    let opt = "[[option]]\nid = \"a\"\ntitle = \"A\"\nsentence = \"search for the alternatives and weigh them against each other.\"\n\n";
    let body = format!(
        "id = \"trust.kind\"\nversion = 1\nfamily = \"trust\"\ntitle = \"K\"\nshape = \"category\"\nevidence = \"weak\"\nsource = \"s\"\nitems_copied = false\n\n{opt}{opt}"
    );
    assert_eq!(refused(&body), "invalid_trait_file");
}

#[test]
fn refused_trait_ids_name_their_reason() {
    for (id, why) in [
        ("mbti", "typology without predictive validity"),
        ("gender", "protected attribute, not a mechanism"),
        ("beauty", "an attribute of the persona, not a sensitivity of the juror"),
    ] {
        let ws = tempfile::tempdir().unwrap();
        let body = GOOD.replace("trust.sample", &format!("culture.{id}"));
        let v = add_file(ws.path(), "t.toml", &body);
        assert_eq!(code(&v), "refused_trait_id", "{id}");
        assert_eq!(v["data"]["why"], why);
    }
}

#[test]
fn trait_sets_are_bounded_and_checked_against_the_store() {
    let ws = installed();
    let set = |name: &str, traits: &[&str], cover: &[&str]| {
        let body = format!(
            "name = \"{name}\"\nversion = 1\ntraits = {}\ncover = {}\nnote = \"n\"\n",
            json!(traits),
            json!(cover)
        );
        let f = ws.path().join(format!("{name}.toml"));
        std::fs::write(&f, body).unwrap();
        jg(ws.path(), &["trait", "set", "add", "--file", f.to_str().unwrap()])
    };
    let nine = ["trust.propensity", "trust.institutions", "trust.automation", "risk.social", "risk.financial", "risk.ethical", "cognition.reflection", "cognition.need_for_closure", "choice.maximizing"];
    assert_eq!(code(&set("big", &nine, &[])), "trait_set_too_large");
    let ok8 = ok(set("eight", &nine[..8], &[]));
    assert_eq!(ok8["version"], 1);
    assert_eq!(code(&set("wide", &nine[..8], &nine[..5])), "cover_too_large");
    assert_eq!(code(&set("stray", &nine[..4], &["risk.ethical"])), "unknown_trait");
    assert_eq!(code(&set("ghost", &["trust.propensity", "culture.power_distance"], &[])), "unknown_trait");
    // a correlated pair is a warning, not a refusal
    let w = ok(set("pair", &["personality.agreeableness", "trust.propensity"], &[]));
    assert_eq!(w["warnings"][0]["code"], "correlated_pair");
}

#[test]
fn the_persona_key_is_checked_against_the_store() {
    let ws = installed();
    let w = ws.path();
    let try_add = |traits: serde_json::Value| add_persona(w, &persona_body("p", &["x"], traits));
    // a culture trait is refused by its family before the store is asked (review I2); a plain unknown id is unknown
    assert_eq!(code(&try_add(json!({"culture.power_distance": 3}))), "refused_trait_id");
    assert_eq!(code(&try_add(json!({"trust.never_added": 3}))), "unknown_trait");
    assert_eq!(code(&try_add(json!({"gender": "x"}))), "refused_trait_id");
    let v = try_add(json!({"beauty": 5}));
    assert_eq!(v["data"]["why"], "an attribute of the persona, not a sensitivity of the juror");
    assert_eq!(code(&try_add(json!({"trust.propensity": 6}))), "trait_value_off_scale");
    assert_eq!(code(&try_add(json!({"trust.propensity": "high"}))), "trait_value_off_scale");
    assert_eq!(code(&try_add(json!({"trust.propensity": 2.5}))), "invalid_arguments");
    assert_eq!(code(&try_add(json!({"situation.job": "x".repeat(201)}))), "trait_text_refused");
    assert_eq!(code(&try_add(json!({"choice.decision_style": "wild"}))), "trait_option_unknown");
    assert_eq!(code(&try_add(json!({"choice.decision_style": ["rational", "intuitive"]}))), "trait_option_unknown");
    assert_eq!(code(&try_add(json!({"access.need": ["telepathy"]}))), "trait_option_unknown");
    ok(try_add(json!({"access.need": ["screen_reader", "captions"], "situation.job": "compare two plans"})));
    let list = ok(jg(w, &["persona", "list"]));
    assert_eq!(list["items"][0]["traits"], 2);
}

#[test]
fn eight_traits_are_accepted_and_nine_are_not() {
    let ws = installed();
    let ids = ["trust.propensity", "trust.institutions", "trust.automation", "risk.social", "risk.financial", "risk.ethical", "cognition.reflection", "cognition.need_for_closure", "choice.maximizing"];
    let m = |n: usize| serde_json::Value::Object(ids[..n].iter().map(|i| (i.to_string(), json!(3))).collect());
    ok(add_persona(ws.path(), &persona_body("eight", &["x"], m(8))));
    let v = add_persona(ws.path(), &persona_body("nine", &["x"], m(9)));
    assert_eq!(code(&v), "traits_too_many");
    assert_eq!(v["data"]["count"], 9);
}

#[test]
fn a_persona_without_traits_keeps_its_hash() {
    let ws = tempfile::tempdir().unwrap();
    let d = ok(add_persona(ws.path(), &persona_body("plain", &["x"], serde_json::Value::Null)));
    // the canonical JSON of a file with no `traits` key is what v0.5 hashed
    let body = persona_body("plain", &["x"], serde_json::Value::Null);
    let canon = personajury::domain::hash::canonical_json(&body);
    assert_eq!(d["hash"], personajury::domain::hash::sha256_hex(canon.as_bytes()));
}
