//! D30: quotas (the [filter] grammar), the cover on a panel, the coverage counts and the cover-aware draw.
#[path = "traits_support/mod.rs"]
mod support;
use serde_json::{json, Value};
use support::*;

const RF: &str = "risk.financial";
const MS: &str = "trust.marketing_skepticism";
const REF: &str = "cognition.reflection";
const EASE: &str = "adoption.ease_demanded";

fn resolve(ws: &std::path::Path, name: &str) -> Value {
    jg(ws, &["panel", "resolve", name])
}
fn ids(v: &Value) -> Vec<String> {
    v["data"]["personas"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect()
}
fn panel(ws: &std::path::Path, name: &str, query: &str, cover: &[&str]) -> Value {
    let mut a = vec!["panel", "add", name, "--query", query];
    let joined = cover.join(",");
    if !cover.is_empty() {
        a.extend(["--cover", &joined]);
    }
    jg(ws, &a)
}

fn graded() -> tempfile::TempDir {
    let ws = installed();
    for n in 1..=5 {
        add_ok(ws.path(), &format!("p{n}"), &["x"], json!({"expertise.domain": n, "access.need": if n % 2 == 0 { json!(["captions"]) } else { json!(["screen_reader", "captions"]) }, "choice.decision_style": "rational"}));
    }
    ws
}

#[test]
fn every_operator_narrows_the_pool_before_the_draw() {
    let ws = graded();
    let w = ws.path();
    for (filter, want) in [
        ("expertise.domain>=4", vec!["p4", "p5"]),
        ("expertise.domain<=2", vec!["p1", "p2"]),
        ("expertise.domain=3", vec!["p3"]),
        ("expertise.domain!=3", vec!["p1", "p2", "p4", "p5"]),
        ("expertise.domain>=2, expertise.domain<=3", vec!["p2", "p3"]),
        ("access.need=screen_reader", vec!["p1", "p3", "p5"]),
        ("access.need!=screen_reader", vec!["p2", "p4"]),
        ("choice.decision_style=rational", vec!["p1", "p2", "p3", "p4", "p5"]),
    ] {
        ok(panel(w, "q", &format!("5 x[{filter}]"), &[]));
        assert_eq!(ids(&resolve(w, "q")), want, "{filter}");
    }
}

#[test]
fn a_filter_that_excludes_nobody_gives_the_v05_seats() {
    let ws = installed();
    for n in 0..12 {
        add_ok(ws.path(), &format!("p{n:02}"), &["x"], json!({"expertise.domain": 1 + n % 5}));
    }
    add_ok(ws.path(), "other", &["y"], Value::Null);
    ok(jg(ws.path(), &["subject", "add", "s", "--kind", "document", "--attach", {
        let f = ws.path().join("d.md");
        std::fs::write(&f, "# d\n").unwrap();
        Box::leak(f.to_string_lossy().into_owned().into_boxed_str())
    }]));
    ok(panel(ws.path(), "plain", "4 x=primary", &[]));
    let before = jg(ws.path(), &["panel", "resolve", "plain", "--subject", "s"]);
    assert_eq!(before["data"]["terms"][0]["got"], 4);
    // the seed names the panel, so the same name with a filter that keeps everyone must seat the same people
    ok(panel(ws.path(), "plain", "4 x=primary[expertise.domain>=1]", &[]));
    let after = jg(ws.path(), &["panel", "resolve", "plain", "--subject", "s"]);
    assert_eq!(ids(&before), ids(&after), "a filter that excludes nobody moves no seat");
}

#[test]
fn the_filter_refusals_by_code() {
    let ws = graded();
    let w = ws.path();
    assert_eq!(code(&panel(w, "q", "2 x[access.need>=screen_reader]", &[])), "invalid_filter");
    assert_eq!(code(&panel(w, "q", "2 x[choice.decision_style>=rational]", &[])), "invalid_filter");
    assert_eq!(code(&panel(w, "q", "2 x[expertise.domain]", &[])), "invalid_filter");
    assert_eq!(code(&panel(w, "q", "2 x[]", &[])), "invalid_filter");
    assert_eq!(code(&panel(w, "q", "2 x[culture.power_distance>=3]", &[])), "unknown_trait");
    assert_eq!(code(&panel(w, "q", "2 x[expertise.domain>=9]", &[])), "trait_value_off_scale");
    assert_eq!(code(&panel(w, "q", "2 x[choice.decision_style=wild]", &[])), "trait_option_unknown");
    assert_eq!(code(&panel(w, "q", "2 x[situation.job=x]", &[])), "invalid_filter");
}

#[test]
fn a_filter_that_matches_nobody_leaves_the_term_empty() {
    let ws = graded();
    let w = ws.path();
    let f = w.join("d.md");
    std::fs::write(&f, "# d\n").unwrap();
    ok(jg(w, &["subject", "add", "s", "--kind", "document", "--attach", f.to_str().unwrap()]));
    let v = jg(w, &["brief", "--subject", "s", "--query", "2 x=primary[expertise.domain>=5, expertise.domain<=1]", "--lens", "critical", "--model", "m"]);
    assert_eq!(code(&v), "panel_term_empty");
    // a persona without the trait never matches, even for !=
    add_ok(w, "bare", &["x"], Value::Null);
    ok(panel(w, "q", "9 x[expertise.domain!=3]", &[]));
    assert!(!ids(&resolve(w, "q")).contains(&"bare".to_string()));
}

#[test]
fn a_cover_names_at_most_four_stored_traits() {
    let ws = installed();
    let w = ws.path();
    ok(panel(w, "four", "1 x", &[RF, MS, REF, EASE]));
    let v = panel(w, "five", "1 x", &[RF, MS, REF, EASE, "cognition.need_for_closure"]);
    assert_eq!(code(&v), "cover_too_large");
    assert_eq!(v["data"]["cover"], 5);
    assert_eq!(code(&panel(w, "ghost", "1 x", &["culture.power_distance"])), "unknown_trait");
}

fn three() -> tempfile::TempDir {
    let ws = installed();
    let w = ws.path();
    add_ok(w, "p1", &["x"], json!({RF: 1, MS: 1}));
    add_ok(w, "p2", &["x", "newcomer"], json!({RF: 3, MS: 5}));
    add_ok(w, "p3", &["x", "newcomer"], json!({RF: 5, MS: 5}));
    ws
}

#[test]
fn coverage_counts_one_way_pairs_balance_and_what_is_uncovered() {
    let ws = three();
    let w = ws.path();
    ok(panel(w, "c", "3 x=primary", &[RF, MS]));
    let c = resolve(w, "c")["data"]["coverage"].clone();
    assert_eq!(c["cover"], json!([RF, MS]));
    let one = c["one_way"].as_array().unwrap();
    assert_eq!(one.len(), 2);
    assert_eq!(one[0], json!({"trait": RF, "shape": "band", "stratum": "primary", "seated": {"low": 1, "mid": 1, "high": 1, "unset": 0}}));
    assert_eq!(one[1]["seated"], json!({"low": 1, "mid": 0, "high": 2, "unset": 0}));
    assert_eq!(c["balance"], json!([{"trait": MS, "stratum": "primary", "missing": ["mid"]}]));
    assert_eq!(c["pairs"]["scope"], "panel");
    assert_eq!((c["pairs"]["total"].as_i64(), c["pairs"]["covered"].as_i64()), (Some(9), Some(3)));
    let un = c["pairs"]["uncovered"].as_array().unwrap();
    assert_eq!(un.len(), 6);
    assert!(un.contains(&json!([RF, "low", MS, "high"])));
    assert!(!un.contains(&json!([RF, "low", MS, "low"])));
    assert_eq!(c["pairs"]["excluded_correlated"], json!([]));
    assert_eq!(c["collinear"], json!([]));
    // the same counts ride on the brief answer
    let f = w.join("d.md");
    std::fs::write(&f, "# d\n").unwrap();
    ok(jg(w, &["subject", "add", "s", "--kind", "document", "--attach", f.to_str().unwrap()]));
    let b = ok(jg(w, &["brief", "--subject", "s", "--panel", "c", "--lens", "critical", "--model", "m"]));
    assert_eq!(b["coverage"]["pairs"]["total"], 9);
    let rec = personajury::store::Store::open(w).unwrap().run_traits(b["run"].as_i64().unwrap()).unwrap().0.unwrap();
    assert_eq!(rec["draw"], "cover");
    // no cover, no coverage
    ok(panel(w, "plain", "3 x=primary", &[]));
    assert!(resolve(w, "plain")["data"].get("coverage").is_none());
}

#[test]
fn a_correlated_pair_is_left_out_of_the_pair_total() {
    let ws = three();
    let w = ws.path();
    for (n, v) in [("p1", 1), ("p2", 3), ("p3", 5)] {
        let mut body = persona_body(n, if n == "p1" { &["x"][..] } else { &["x", "newcomer"][..] }, json!({RF: v, MS: if n == "p1" { 1 } else { 5 }, "personality.neuroticism": v}));
        body["name"] = json!(n);
        ok(add_persona(w, &body));
    }
    ok(panel(w, "c", "3 x=primary", &[RF, "personality.neuroticism"]));
    let c = resolve(w, "c")["data"]["coverage"].clone();
    assert_eq!(c["pairs"]["total"], 0);
    assert_eq!(c["pairs"]["excluded_correlated"], json!([[RF, "personality.neuroticism"]]));
    // the same trio with an uncorrelated partner counts nine
    ok(panel(w, "d", "3 x=primary", &[RF, MS]));
    assert_eq!(resolve(w, "d")["data"]["coverage"]["pairs"]["total"], 9);
}

#[test]
fn collinear_and_confounded_are_reported() {
    let ws = three();
    let w = ws.path();
    ok(panel(w, "c", "3 x=primary", &[RF, MS]));
    let c = resolve(w, "c")["data"]["coverage"].clone();
    // newcomer holds ms high on both seats, and the seat without the tag is low
    assert_eq!(c["confounded"], json!([{"trait": MS, "tag": "newcomer", "band": "high"}]));
    // two traits that sort the seats the same way
    let ws = installed();
    add_ok(ws.path(), "q1", &["x"], json!({RF: 1, MS: 1}));
    add_ok(ws.path(), "q2", &["x"], json!({RF: 5, MS: 5}));
    ok(panel(ws.path(), "c", "2 x=primary", &[RF, MS]));
    assert_eq!(resolve(ws.path(), "c")["data"]["coverage"]["collinear"], json!([[RF, MS]]));
}

#[test]
fn an_unstaked_term_is_its_own_stratum_and_a_category_has_its_options_as_bands() {
    let ws = installed();
    let w = ws.path();
    add_ok(w, "a", &["x"], json!({"choice.decision_style": "rational", RF: 2}));
    add_ok(w, "b", &["x"], json!({"choice.decision_style": "dependent"}));
    ok(panel(w, "c", "2 x", &["choice.decision_style", RF]));
    let c = resolve(w, "c")["data"]["coverage"].clone();
    let row = &c["one_way"][0];
    assert_eq!((row["stratum"].as_str(), row["shape"].as_str()), (Some("unstaked"), Some("category")));
    assert_eq!(row["seated"]["rational"], 1);
    assert_eq!(row["seated"]["dependent"], 1);
    assert_eq!(row["seated"]["avoidant"], 0);
    assert_eq!(c["one_way"][1]["seated"]["unset"], 1);
}

/// OA(9, 4, 3, 2): any two columns show all nine band pairs.
const OA: [[i64; 4]; 9] = [[1, 1, 1, 1], [1, 3, 3, 3], [1, 5, 5, 5], [3, 1, 3, 5], [3, 3, 5, 1], [3, 5, 1, 3], [5, 1, 5, 3], [5, 3, 1, 5], [5, 5, 3, 1]];

fn oa_pool(w: &std::path::Path, extra: usize) {
    let cols = [RF, MS, REF, EASE];
    for (i, row) in OA.iter().enumerate() {
        let t: serde_json::Map<String, Value> = cols.iter().zip(row).map(|(c, v)| (c.to_string(), json!(v))).collect();
        add_ok(w, &format!("r{i}"), &["x"], Value::Object(t));
    }
    // decoys that repeat the first row: they add nothing a seat of the array has not
    for i in 0..extra {
        add_ok(w, &format!("a{i}"), &["x"], json!({RF: 1, MS: 1, REF: 1, EASE: 1}));
    }
}

#[test]
fn a_covering_array_pool_covers_every_pair_with_nine_seats() {
    let ws = installed();
    oa_pool(ws.path(), 0);
    ok(panel(ws.path(), "c", "9 x=primary", &[RF, MS, REF, EASE]));
    let c = resolve(ws.path(), "c")["data"]["coverage"].clone();
    assert_eq!(c["pairs"]["total"], 54);
    assert_eq!(c["pairs"]["covered"], 54);
    assert_eq!(c["pairs"]["uncovered"], json!([]));
    assert_eq!(c["balance"], json!([]));
}

#[test]
fn the_cover_draw_finds_the_array_among_decoys_and_repeats_itself() {
    let ws = installed();
    oa_pool(ws.path(), 5);
    let f = ws.path().join("d.md");
    std::fs::write(&f, "# d\n").unwrap();
    ok(jg(ws.path(), &["subject", "add", "s", "--kind", "document", "--attach", f.to_str().unwrap()]));
    ok(panel(ws.path(), "c", "9 x=primary", &[RF, MS, REF, EASE]));
    let a = jg(ws.path(), &["panel", "resolve", "c", "--subject", "s"]);
    let b = jg(ws.path(), &["panel", "resolve", "c", "--subject", "s"]);
    assert_eq!(ids(&a), ids(&b), "same inputs, same seats");
    assert_eq!(a["data"]["coverage"]["pairs"]["covered"], 54);
    assert_eq!(ids(&a).iter().filter(|i| i.starts_with('a')).count(), 0, "the array's nine rows are all it needs");
    // without a cover the same pool leaves pairs uncovered or the decoys in: the draw is the seeded one
    ok(panel(ws.path(), "plain", "9 x=primary", &[]));
    assert!(jg(ws.path(), &["panel", "resolve", "plain", "--subject", "s"])["data"].get("coverage").is_none());
}
