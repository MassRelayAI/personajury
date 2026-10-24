//! D36: `graph --svg --slice TRAIT` fills the persona nodes by group, adds a legend line with the counts per stratum, and
//! leaves the layout and the medians alone. Without `--slice` the output is the v0.5 one (tests/graph_asset.rs holds the bytes).
#[path = "judging_support/mod.rs"]
mod fx;
#[path = "traits_support/mod.rs"]
mod support;
use personajury::domain::Stake;
use serde_json::{json, Value};

const T: &str = "consumer.price_consciousness";
const LOW: &str = "#b4d3ee";
const MID: &str = "#4f9fe3";
const HIGH: &str = "#0550ae";
const GREY: &str = "#8c959f";

fn crit(persona: &str, s: i64) -> Value {
    let lines: Vec<Value> = ["works", "blocks", "missing"]
        .iter()
        .map(|d| json!({"dimension": d, "score": s, "anchor_met": 3, "evidence": "seen", "citations": ["page.json#n1"]}))
        .collect();
    json!({"persona": persona, "lines": lines})
}
/// j1 low, j2 high, j3 high, j4 no trait (all primary); s1 mid (secondary).
fn world() -> (fx::Ws, i64) {
    let ws = fx::Ws::new();
    ws.site();
    support::ok(support::jg(ws.root(), &["trait", "install"]));
    for (p, v) in [("j1", Some(1)), ("j2", Some(5)), ("j3", Some(4)), ("j4", None), ("s1", Some(3))] {
        support::add_ok(ws.root(), p, &["t"], v.map_or(Value::Null, |b| json!({ T: b })));
    }
    let run = ws.run(
        "critical",
        &[("j1", Stake::Primary), ("j2", Stake::Primary), ("j3", Stake::Primary), ("j4", Stake::Primary), ("s1", Stake::Secondary)],
        false,
    );
    for p in ["j1", "j2", "j3", "j4", "s1"] {
        ws.ingest(run, &crit(p, 3));
    }
    (ws, run)
}
fn svg(ws: &fx::Ws, run: i64, extra: &[&str]) -> String {
    let r = run.to_string();
    let mut a = vec!["graph", r.as_str(), "--svg"];
    a.extend(extra);
    let v = ws.cli(&a);
    assert_eq!(v["ok"], true, "{v}");
    v["data"]["svg"].as_str().unwrap().to_string()
}
fn fill_of(svg: &str, persona: &str) -> String {
    let at = svg.find(&format!("class=\"persona\" data-persona=\"{persona}\"")).unwrap();
    let rest = &svg[at..];
    let f = rest.find("fill=\"").unwrap() + 6;
    rest[f..f + 7].to_string()
}
/// Every geometry attribute of every persona rect and of the document.
fn geometry(svg: &str) -> Vec<String> {
    let mut out = vec![svg.lines().next().unwrap().to_string()];
    for l in svg.lines().filter(|l| l.contains("class=\"persona\"")) {
        let a = |k: &str| {
            let at = l.find(&format!(" {k}=\"")).unwrap() + k.len() + 3;
            l[at..].split('"').next().unwrap().to_string()
        };
        out.push(format!("{} {} {} {} {}", a("data-persona"), a("x"), a("y"), a("width"), a("height")));
    }
    out
}

#[test]
fn personas_are_filled_by_group_and_unset_is_grey() {
    let (ws, run) = world();
    let s = svg(&ws, run, &["--slice", T]);
    assert_eq!(fill_of(&s, "j1"), LOW);
    assert_eq!(fill_of(&s, "j2"), HIGH);
    assert_eq!(fill_of(&s, "j3"), HIGH);
    assert_eq!(fill_of(&s, "j4"), GREY);
    assert_eq!(fill_of(&s, "s1"), MID);
}
#[test]
fn the_legend_line_names_the_trait_and_the_counts_per_stratum_never_one_total() {
    let (ws, run) = world();
    let s = svg(&ws, run, &["--slice", T]);
    let title = support::ok(support::jg(ws.root(), &["trait", "show", T]))["title"].as_str().unwrap().to_string();
    assert!(s.contains(&title.chars().take(30).collect::<String>()), "the trait title is named: {title} in {s}");
    assert!(s.contains("P 1\u{b7}0\u{b7}2"), "primary: one low, no mid, two high: {s}");
    assert!(s.contains("S 0\u{b7}1\u{b7}0"), "secondary: the middle band only");
    assert!(!s.contains("B 0"), "no bystander stratum, no bystander count");
    assert!(s.contains("1 unset"), "the primary persona without the trait is counted as unset");
}
#[test]
fn layout_and_medians_do_not_move() {
    let (ws, run) = world();
    let (plain, sliced) = (svg(&ws, run, &[]), svg(&ws, run, &["--slice", T]));
    assert_eq!(geometry(&plain), geometry(&sliced));
    let badges = |s: &str| s.lines().filter(|l| l.contains("class=\"badge\"")).map(String::from).collect::<Vec<_>>();
    assert_eq!(badges(&plain), badges(&sliced));
    assert_eq!(fill_of(&plain, "j1"), "#ffffff", "without --slice the node is as it was");
    assert!(!plain.contains("unset"));
}
#[test]
fn a_category_cycles_the_three_steps_in_option_order() {
    let ws = fx::Ws::new();
    ws.site();
    support::ok(support::jg(ws.root(), &["trait", "install"]));
    let opts = ["rational", "intuitive", "dependent", "avoidant", "spontaneous"];
    let def = support::ok(support::jg(ws.root(), &["trait", "show", "choice.decision_style"]));
    let ids: Vec<String> = def["option"].as_array().or(def["options"].as_array()).unwrap().iter().map(|o| o["id"].as_str().unwrap().to_string()).collect();
    assert!(ids.len() >= 4, "{ids:?} {opts:?}");
    let mut stakes = Vec::new();
    for (i, id) in ids.iter().take(4).enumerate() {
        let p = format!("c{i}");
        support::add_ok(ws.root(), &p, &["t"], json!({"choice.decision_style": id}));
        stakes.push((p, Stake::Primary));
    }
    let st: Vec<(&str, Stake)> = stakes.iter().map(|(p, s)| (p.as_str(), *s)).collect();
    let run = ws.run("critical", &st, false);
    for (p, _) in &stakes {
        ws.ingest(run, &crit(p, 3));
    }
    let s = svg(&ws, run, &["--slice", "choice.decision_style"]);
    assert_eq!([fill_of(&s, "c0"), fill_of(&s, "c1"), fill_of(&s, "c2"), fill_of(&s, "c3")], [LOW, MID, HIGH, LOW]);
    assert!(s.contains(&ids[0]) && s.contains("P 1\u{b7}1\u{b7}1\u{b7}1"), "{s}");
}
#[test]
fn slice_refusals() {
    let (ws, run) = world();
    let r = run.to_string();
    assert_eq!(support::code(&ws.cli(&["graph", &r, "--svg", "--slice", "nope.nothing"])), "unknown_trait");
    assert_eq!(support::code(&ws.cli(&["graph", &r, "--svg", "--slice", "situation.job"])), "invalid_arguments");
}

// ---- the (P) badge of a project trait in a slice header ----
#[test]
fn a_project_trait_is_badged_p_in_the_matrix_slice_header() {
    let (ws, run) = world();
    let m = ws.root().join("buyers.md");
    std::fs::write(&m, "# Buyers\nOur buyers are procurement or engineering.\n").unwrap();
    let g = support::ok(support::jg(ws.root(), &["trait", "generate", "--from", m.to_str().unwrap()]));
    let hash = g["material_hash"].as_str().unwrap().to_string();
    let brief = std::fs::read_to_string(ws.root().join(g["brief"].as_str().unwrap())).unwrap();
    let body = brief.split("```toml\n").nth(1).unwrap().split("```").next().unwrap().to_string();
    let f = ws.root().join("t.toml");
    std::fs::write(&f, body).unwrap();
    support::ok(support::jg(ws.root(), &["trait", "add", "--file", f.to_str().unwrap(), "--generated-from", &hash]));
    let id = "buyers.reads_the_fine_print";
    support::add_ok(ws.root(), "p1", &["t"], json!({ id: 5 }));
    support::add_ok(ws.root(), "p2", &["t"], json!({ id: 1 }));
    let run2 = ws.run("critical", &[("p1", Stake::Primary), ("p2", Stake::Primary)], false);
    for p in ["p1", "p2"] {
        ws.ingest(run2, &crit(p, 3));
    }
    let m = ws.cli(&["matrix", &run2.to_string(), "--slice", id, "--table"]);
    assert_eq!(m["ok"], true, "{m}");
    assert!(m["data"]["table"].as_str().unwrap().contains(&format!("slice {id} (P): primary, low")), "{}", m["data"]["table"]);
    // a catalog trait has no badge
    let m = ws.cli(&["matrix", &run.to_string(), "--slice", T, "--table"]);
    assert!(m["data"]["table"].as_str().unwrap().contains(&format!("slice {T}: primary, low")));
    assert!(!m["data"]["table"].as_str().unwrap().contains("(P)"));
}
