//! D31: `matrix --slice TRAIT` groups each stratum's scores by a trait's bands, as counts, and `summary --slice` says it in
//! one fixed sentence per stratum. A new workspace with the catalog installed (not the golden run).
#[path = "judging_support/mod.rs"]
mod fx;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

fn run_ws() -> (tempfile::TempDir, i64, i64) {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fx::fixture_run(ws.path());
    (ws, a, b)
}
fn matrix(ws: &Path, a: i64, b: i64, slice: &str, extra: &[&str]) -> Value {
    let (a, b) = (a.to_string(), b.to_string());
    let mut args = vec!["matrix", a.as_str(), b.as_str(), "--slice", slice];
    args.extend(extra);
    fx::cli_in(ws, &args)
}
fn slice_of<'a>(m: &'a Value, stratum: &str) -> &'a Value {
    m["data"]["slices"].as_array().unwrap().iter().find(|s| s["stratum"] == stratum).unwrap()
}
fn group<'a>(s: &'a Value, band: &str) -> &'a Value {
    s["groups"].as_array().unwrap().iter().find(|g| g["band"] == band).unwrap()
}

#[test]
fn groups_per_stratum_with_thin_and_unset() {
    let (ws, a, b) = run_ws();
    let m = matrix(ws.path(), a, b, "expertise.domain", &[]);
    assert_eq!(m["ok"], true, "{m}");
    let slices = m["data"]["slices"].as_array().unwrap();
    assert_eq!(slices.len(), 3, "one entry per stratum");
    let primary = slice_of(&m, "primary");
    assert_eq!(primary["trait"], "expertise.domain");
    let low = group(primary, "low");
    assert_eq!((low["personas"].clone(), low["n"].as_i64(), low["thin"].as_bool()), (serde_json::json!(["newcomer-sofia"]), Some(1), Some(true)));
    let high = group(primary, "high");
    assert_eq!((high["n"].as_i64(), high["thin"].as_bool()), (Some(3), Some(false)));
    // nobody in primary holds the middle band: omitted, not an empty group
    assert!(primary["groups"].as_array().unwrap().iter().all(|g| g["band"] != "mid"));
    let secondary = slice_of(&m, "secondary");
    assert_eq!(group(secondary, "low")["n"], 2);
    assert_eq!(slice_of(&m, "bystander")["groups"][0]["thin"], true);
    // a group's cells carry the scores of its own personas only
    let cell = &low["cells"][0];
    assert_eq!(cell["scores"].as_object().unwrap().keys().collect::<Vec<_>>(), vec!["newcomer-sofia"]);
    assert_eq!(cell["n"], 1);
    // the trait that two primary personas lack
    let m = matrix(ws.path(), a, b, "trust.marketing_skepticism", &[]);
    assert_eq!(slice_of(&m, "primary")["unset"], serde_json::json!(["builder-mei", "newcomer-sofia"]));
}

#[test]
fn separates_counts_the_cells_on_which_two_groups_do_not_overlap() {
    let (ws, a, b) = run_ws();
    let m = matrix(ws.path(), a, b, "expertise.domain", &[]);
    let primary = slice_of(&m, "primary");
    // recompute from the recorded verdicts: anchor_met ranges of the low group (sofia) and the high group (ana, mei, ravi)
    type Key = (String, String, Option<String>);
    let mut cells: BTreeMap<Key, BTreeMap<&str, (i64, i64)>> = BTreeMap::new();
    for f in std::fs::read_dir(fx::repo().join("fixtures/run/verdicts")).unwrap() {
        let p = f.unwrap().path();
        let name = p.file_stem().unwrap().to_str().unwrap().to_string();
        let (persona, lens) = name.split_once("--").unwrap();
        let g = match persona {
            "newcomer-sofia" => "low",
            "builder-ana" | "builder-mei" | "builder-ravi" => "high",
            _ => continue,
        };
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        for l in v["lines"].as_array().unwrap() {
            let k = (lens.to_string(), l["dimension"].as_str().unwrap().to_string(), l["about"].as_str().map(String::from));
            let am = l["anchor_met"].as_i64().unwrap();
            let e = cells.entry(k).or_default().entry(g).or_insert((am, am));
            e.0 = e.0.min(am);
            e.1 = e.1.max(am);
        }
    }
    let separated = cells.values().filter(|c| c.len() == 2 && { let (x, y) = (c["low"], c["high"]); x.1 < y.0 || y.1 < x.0 }).count();
    assert_eq!(primary["separates"]["cells"].as_u64().unwrap() as usize, separated, "{primary}");
    let total = m["data"]["strata"].as_array().unwrap().iter().find(|s| s["stake"] == "primary").unwrap()["cells"].as_array().unwrap().len();
    assert_eq!(primary["separates"]["total"].as_u64().unwrap() as usize, total);
}

/// A key that names a pooled figure: any key containing one of these stems, at any depth (`mean_score` and
/// `share_low` count), or exactly `rate` (as a substring it would hit `separates`).
const POOLED: [&str; 8] = ["mean", "avg", "average", "share", "pct", "percent", "prob", "probability"];
fn is_pooled_key(k: &str) -> bool {
    let k = k.to_lowercase();
    k == "rate" || POOLED.iter().any(|s| k.contains(s))
}
#[test]
fn the_key_walk_catches_a_pooled_key_at_any_depth() {
    for k in ["mean", "mean_score", "avg", "group_average", "share_low", "pct", "percent_high", "prob", "probability", "rate"] {
        let mut bad = Vec::new();
        walk(&serde_json::json!({"a": [{"b": {k: 1}}]}), "x", &mut bad);
        assert_eq!(bad, vec![format!("x.a[0].b.{k}")], "{k}");
    }
    let mut bad = Vec::new();
    walk(&serde_json::json!({"separates": {"cells": 1}, "median": 2, "thin": true}), "x", &mut bad);
    assert!(bad.is_empty(), "{bad:?}");
}
fn walk(v: &Value, path: &str, bad: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                if is_pooled_key(k) {
                    bad.push(format!("{path}.{k}"));
                }
                walk(x, &format!("{path}.{k}"), bad);
            }
        }
        Value::Array(a) => a.iter().enumerate().for_each(|(i, x)| walk(x, &format!("{path}[{i}]"), bad)),
        _ => {}
    }
}
#[test]
fn nothing_in_the_slice_is_a_mean_a_share_a_rate_or_a_probability() {
    let (ws, a, b) = run_ws();
    for t in ["expertise.domain", "choice.decision_style", "buying.role"] {
        let m = matrix(ws.path(), a, b, t, &["--detail", "full"]);
        let mut bad = Vec::new();
        walk(&m["data"]["slices"], "slices", &mut bad);
        assert!(bad.is_empty(), "{t}: {bad:?}");
        assert!(!m["data"]["slices"].as_array().unwrap().is_empty());
    }
}

#[test]
fn the_table_repeats_the_stratum_in_every_group_block() {
    let (ws, a, b) = run_ws();
    let m = matrix(ws.path(), a, b, "expertise.domain", &["--table"]);
    let t = m["data"]["table"].as_str().unwrap();
    for header in [
        "slice expertise.domain: primary, low (n 1: newcomer-sofia)",
        "slice expertise.domain: primary, high (n 3: builder-ana, builder-mei, builder-ravi)",
        "slice expertise.domain: primary, mid: \u{2014}",
        "slice expertise.domain: secondary, low (n 2: company-george, company-hanna)",
        "slice expertise.domain: bystander, high (n 1: investor-claire)",
    ] {
        assert!(t.contains(header), "{header}\n{t}");
    }
    // every group's rows name their stratum
    let block = t.split("slice expertise.domain: primary, low").nth(1).unwrap();
    assert!(block.contains("| primary | newcomer-sofia |"), "{block}");
}

#[test]
fn a_category_groups_by_option_and_a_missing_trait_is_refused() {
    let (ws, a, b) = run_ws();
    let m = matrix(ws.path(), a, b, "choice.decision_style", &[]);
    let primary = slice_of(&m, "primary");
    let bands: Vec<&str> = primary["groups"].as_array().unwrap().iter().map(|g| g["band"].as_str().unwrap()).collect();
    assert_eq!(bands, vec!["rational", "dependent"], "option order of the trait");
    let v = matrix(ws.path(), a, b, "culture.power_distance", &[]);
    assert_eq!(v["error"]["code"], "unknown_trait");
    let v = matrix(ws.path(), a, b, "situation.job", &[]);
    assert_eq!(v["error"]["code"], "invalid_arguments");
}

#[test]
fn summary_slice_adds_one_sentence_per_stratum_and_nothing_else() {
    let (ws, a, b) = run_ws();
    let (a, b) = (a.to_string(), b.to_string());
    let plain = fx::j(ws.path(), &["summary", &a, &b])["data"]["text"].as_str().unwrap().to_string();
    let sliced = fx::j(ws.path(), &["summary", &a, &b, "--slice", "expertise.domain"])["data"]["text"].as_str().unwrap().to_string();
    let flat = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
    let (flat_sliced, tail) = (flat(&sliced), "In this jury, not a claim about people.");
    assert_eq!(flat_sliced.matches(tail).count(), 3, "one sentence per stratum: {flat_sliced}");
    assert!(flat_sliced.contains("Among primary jurors, those high on \"How well the reader knows this subject's field\" (3: builder-ana, builder-mei, builder-ravi) scored "), "{flat_sliced}");
    assert!(flat_sliced.contains("those low (1) scored it "), "{flat_sliced}");
    assert!(flat_sliced.contains("Among secondary jurors, those low on"), "{flat_sliced}");
    assert!(flat_sliced.contains("Among bystander jurors, those high on"), "{flat_sliced}");
    // nothing else changes: every line of the plain text is still there, in order
    let mut rest = sliced.lines();
    for l in plain.lines() {
        assert!(rest.any(|x| x == l), "the line is gone: {l:?}");
    }
    let cat = fx::j(ws.path(), &["summary", &a, &b, "--slice", "choice.decision_style"])["data"]["text"].as_str().unwrap().to_string();
    assert!(flat(&cat).contains("those who searches and weighs (1: builder-ana) scored "), "{cat}");
}
