//! v0.3 (D11): `check --history` lists each finding's life across runs, and `graph --svg` draws the jury. Every test
//! here was seen red first.
#[path = "judging_support/mod.rs"]
mod support;
use personajury::domain::Stake;
use serde_json::{json, Value};
use support::*;

const AC: [(&str, Stake); 2] = [("ana", Stake::Primary), ("cy", Stake::Bystander)];

fn life(c: &Value, finding: &str) -> Vec<(i64, String)> {
    let l = c["data"]["lives"]
        .as_array()
        .unwrap_or_else(|| panic!("no lives: {c}"))
        .iter()
        .find(|l| l["finding"] == finding)
        .unwrap_or_else(|| panic!("no life for {finding}: {c}"));
    l["life"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (s["run"].as_i64().unwrap(), s["state"].as_str().unwrap().to_string()))
        .collect()
}

#[test]
fn history_shows_raised_then_fixed_then_absent() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    let bad = critical("ana", [3, 3, 3], &["the Go button does nothing"], &["page.json#n1"]);
    let f = ws.ingest(r1, &bad)["findings"][0].as_str().unwrap().to_string();
    ws.cli(&["decide", &f, "fixed", "--by", "agent"]);
    // the next round: ana judged again and raised nothing
    let r2 = ws.run("critical", &AC, false);
    ws.ingest(r2, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    let c = ws.cli(&["check", "--history"]);
    assert_eq!(c["ok"], true, "{c}");
    assert_eq!(life(&c, &f), vec![(r1, "raised".to_string()), (r2, "fixed".to_string())]);
    let l = &c["data"]["lives"][0];
    assert_eq!(l["status"], "fixed", "{l}");
    assert_eq!(l["decision"]["outcome"], "fixed");
}

#[test]
fn history_shows_a_fix_that_did_not_land_as_raised_again() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    let bad = critical("ana", [3, 3, 3], &["the Go button does nothing"], &["page.json#n1"]);
    let f = ws.ingest(r1, &bad)["findings"][0].as_str().unwrap().to_string();
    ws.cli(&["decide", &f, "fixed", "--by", "agent"]);
    let r2 = ws.run("critical", &AC, false);
    ws.ingest(r2, &bad);
    // check fails (fix_not_landed) and still carries the lives
    let c = ws.cli(&["check", "--history"]);
    assert_eq!(c["error"]["code"], "check_failed", "{c}");
    assert_eq!(life(&c, &f), vec![(r1, "raised".to_string()), (r2, "raised_again".to_string())]);
    assert_eq!(c["data"]["lives"][0]["status"], "open", "a fix that did not land is open again: {c}");
}

#[test]
fn history_without_the_flag_has_no_lives() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    ws.ingest(r1, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    let c = ws.cli(&["check"]);
    assert!(c["data"]["lives"].is_null(), "{c}");
    let c = ws.cli(&["check", "--history"]);
    assert_eq!(c["data"]["lives"], json!([]));
}

// ---- the graph ----

const RED: &str = "#cf222e";
const GREEN: &str = "#1a7f37";
const GREY: &str = "#8c959f";
const BLUE: &str = "#0969da";

fn svg_of(ws: &Ws, runs: &[i64]) -> String {
    let mut a: Vec<String> = vec!["graph".into()];
    a.extend(runs.iter().map(|r| r.to_string()));
    a.push("--svg".into());
    let a: Vec<&str> = a.iter().map(String::as_str).collect();
    let r = ws.cli(&a);
    assert_eq!(r["ok"], true, "{r}");
    r["data"]["svg"].as_str().unwrap().to_string()
}
/// ana (primary) and cy (bystander) judge the critical lens; ana's first line scores `works` and raises two blockers,
/// cy raises two more. Returns the run and the four finding ids (ana's two, then cy's two).
fn world(works: i64) -> (Ws, i64, Vec<String>) {
    let ws = Ws::new();
    ws.site();
    let r = ws.run("critical", &AC, false);
    let mut ids = Vec::new();
    let a = ws.ingest(r, &critical("ana", [works, 3, 4], &["A: Go does nothing", "B: price hidden"], &["page.json#n1"]));
    let c = ws.ingest(r, &critical("cy", [2, 3, 5], &["C: headline is jargon", "D: no way to try"], &["page.json#n1"]));
    for d in [a, c] {
        ids.extend(d["findings"].as_array().unwrap().iter().map(|f| f.as_str().unwrap().to_string()));
    }
    (ws, r, ids)
}
/// Every `<tag ... class="cls" ...>` opening in the svg, as its text.
fn elements<'a>(svg: &'a str, cls: &str) -> Vec<&'a str> {
    let needle = format!("class=\"{cls}\"");
    svg.lines().filter(|l| l.contains(&needle)).collect()
}
fn attr<'a>(el: &'a str, name: &str) -> &'a str {
    let k = format!(" {name}=\"");
    let i = el.find(&k).unwrap_or_else(|| panic!("no {name} in {el}")) + k.len();
    &el[i..i + el[i..].find('"').unwrap()]
}

#[test]
fn graph_needs_the_svg_flag() {
    let (ws, r, _) = world(3);
    let v = ws.cli(&["graph", &r.to_string()]);
    assert_eq!(v["error"]["code"], "invalid_arguments", "{v}");
}

#[test]
fn the_same_store_gives_the_same_bytes_and_out_writes_them() {
    let (ws, r, _) = world(3);
    let (a, b) = (svg_of(&ws, &[r]), svg_of(&ws, &[r]));
    assert_eq!(a, b);
    assert!(a.starts_with("<svg ") && a.trim_end().ends_with("</svg>"));
    let out = ws.root().join("g.svg");
    let v = ws.cli(&["graph", &r.to_string(), "--svg", "--out", out.to_str().unwrap()]);
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(std::fs::read_to_string(&out).unwrap(), a);
    assert!(v["data"]["svg"].is_null() && v["data"]["bytes"] == a.len(), "{v}");
}

#[test]
fn one_moved_score_changes_the_svg() {
    let (w3, r3, _) = world(3);
    let (w4, r4, _) = world(4);
    assert_ne!(svg_of(&w3, &[r3]), svg_of(&w4, &[r4]));
}

#[test]
fn the_card_is_the_readme_card_with_title_desc_and_role() {
    let (ws, r, _) = world(3);
    let svg = svg_of(&ws, &[r]);
    assert!(svg.contains("width=\"1200\""), "{}", &svg[..300]);
    assert!(svg.contains("role=\"img\"") && svg.contains("<title id=\"t\">") && svg.contains("<desc id=\"d\">"));
    assert!(svg.contains("stroke=\"#d0d7de\"") && svg.contains("rx=\"10\""));
    assert!(svg.contains("system-ui"));
}

#[test]
fn desc_counts_equal_the_stores() {
    let (ws, r, ids) = world(3);
    ws.cli(&["decide", &ids[0], "fixed", "--by", "a"]);
    ws.cli(&["decide", &ids[1], "dismissed", "--why", "w", "--by", "a"]);
    ws.cli(&["decide", &ids[2], "row", "--row", "docs#1", "--by", "a"]);
    let svg = svg_of(&ws, &[r]);
    let d = &svg[svg.find("<desc").unwrap()..svg.find("</desc>").unwrap()];
    let m = ws.cli(&["matrix", &r.to_string()]);
    let n = |stake: &str| {
        m["data"]["strata"].as_array().unwrap().iter().find(|s| s["stake"] == stake).map_or(0, |s| s["personas"].as_array().unwrap().len())
    };
    assert!(d.contains(&format!("{} personas (primary {}, secondary {}, bystander {})", 2, n("primary"), n("secondary"), n("bystander"))), "{d}");
    assert!(d.contains("3 dimensions"), "{d}");
    assert!(d.contains("4 findings: 1 open, 1 fixed, 1 dismissed, 1 row"), "{d}");
}

#[test]
fn finding_edges_and_nodes_are_coloured_by_decision() {
    let (ws, r, ids) = world(3);
    ws.cli(&["decide", &ids[1], "fixed", "--by", "a"]);
    ws.cli(&["decide", &ids[2], "dismissed", "--why", "w", "--by", "a"]);
    ws.cli(&["decide", &ids[3], "row", "--row", "docs#1", "--by", "a"]);
    let svg = svg_of(&ws, &[r]);
    let edges = elements(&svg, "edge-finding");
    assert_eq!(edges.len(), 4, "{svg}");
    for (id, status, colour) in [(&ids[0], "open", RED), (&ids[1], "fixed", GREEN), (&ids[2], "dismissed", GREY), (&ids[3], "row", BLUE)] {
        let e = edges.iter().find(|e| attr(e, "data-finding") == id).unwrap();
        assert_eq!(attr(e, "data-status"), status);
        assert_eq!(attr(e, "stroke"), colour, "{e}");
        assert_eq!(e.contains("stroke-dasharray"), status == "dismissed", "{e}");
        let node = elements(&svg, "finding").into_iter().find(|n| attr(n, "data-finding") == id).unwrap();
        assert_eq!(attr(node, "data-status"), status);
    }
    // the full blocker is the tooltip; the label is short
    assert!(svg.contains("A: Go does nothing"));
}

#[test]
fn score_edges_carry_the_number_and_one_badge_per_stratum_never_a_combined_one() {
    let (ws, r, _) = world(3);
    let svg = svg_of(&ws, &[r]);
    let edges = elements(&svg, "edge-score");
    assert_eq!(edges.len(), 6, "2 personas x 3 dimensions: {svg}");
    assert!(svg.contains("<title>ana scored critical:works 3</title>"), "{svg}");
    let badges = elements(&svg, "badge");
    assert_eq!(badges.len(), 3 * 3, "three strata x three dimensions");
    let works_p = badges.iter().find(|b| attr(b, "data-node") == "critical:works" && attr(b, "data-stratum") == "primary").unwrap();
    assert_eq!(attr(works_p, "data-median"), "3");
    let works_s = badges.iter().find(|b| attr(b, "data-node") == "critical:works" && attr(b, "data-stratum") == "secondary").unwrap();
    assert_eq!(attr(works_s, "data-median"), "none");
    assert!(!svg.to_lowercase().contains("average") || svg.contains("never"), "no combined number");
}

#[test]
fn names_are_escaped() {
    let ws = Ws::new();
    ws.site();
    ws.store
        .insert_persona(&personajury::domain::Persona {
            id: "ana".into(),
            version: 0,
            name: "A<b>&\"x".into(),
            tags: vec![],
            stake_rules: "r".into(),
            profile_json: "{}".into(),
            origin: personajury::domain::Origin::Hand,
            hash: "h".into(),
            traits: None,
            serves: vec![],
        })
        .unwrap();
    let r = ws.run("critical", &AC, false);
    ws.ingest(r, &critical("ana", [3, 3, 3], &["x <script> & y"], &["page.json#n1"]));
    let svg = svg_of(&ws, &[r]);
    assert!(svg.contains("A&lt;b&gt;&amp;&quot;x"), "{svg}");
    assert!(!svg.contains("<script>") && !svg.contains("A<b>"));
}

#[test]
fn thirty_personas_lay_out_without_overlap() {
    let ws = Ws::new();
    ws.site();
    let names: Vec<String> = (0..30).map(|i| format!("p{i:02}")).collect();
    let personas: Vec<(&str, Stake)> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), if i < 18 { Stake::Primary } else if i < 26 { Stake::Secondary } else { Stake::Bystander }))
        .collect();
    let r = ws.run("critical", &personas, false);
    for n in &names {
        ws.ingest(r, &critical(n, [3, 3, 3], &["a very long blocker sentence that has to be cut short somewhere sensible"], &["page.json#n1"]));
    }
    let svg = svg_of(&ws, &[r]);
    let height: i64 = attr(svg.lines().next().unwrap(), "height").parse().unwrap();
    let mut boxes: Vec<(i64, i64, i64, i64)> = elements(&svg, "persona")
        .iter()
        .map(|e| (attr(e, "x").parse().unwrap(), attr(e, "y").parse().unwrap(), attr(e, "width").parse().unwrap(), attr(e, "height").parse().unwrap()))
        .collect();
    assert_eq!(boxes.len(), 30);
    boxes.sort();
    for w in boxes.windows(2) {
        let (a, b) = (w[0], w[1]);
        if a.0 == b.0 {
            assert!(b.1 >= a.1 + a.3 + 4, "persona boxes overlap or touch: {a:?} {b:?}");
        }
    }
    for (x, y, w, h) in &boxes {
        assert!(*x >= 0 && x + w <= 1200 && *y >= 0 && y + h <= height, "outside the card");
    }
    // finding rows too
    let mut f: Vec<(i64, i64)> = elements(&svg, "finding").iter().map(|e| (attr(e, "y").parse().unwrap(), attr(e, "height").parse().unwrap())).collect();
    f.sort();
    assert_eq!(f.len(), 30);
    for w in f.windows(2) {
        assert!(w[1].0 >= w[0].0 + w[0].1 + 2, "finding rows overlap: {w:?}");
    }
}

#[test]
fn several_runs_draw_each_findings_life() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    let bad = critical("ana", [3, 3, 3], &["the Go button does nothing"], &["page.json#n1"]);
    let f = ws.ingest(r1, &bad)["findings"][0].as_str().unwrap().to_string();
    ws.cli(&["decide", &f, "fixed", "--by", "a"]);
    let r2 = ws.run("critical", &AC, false);
    ws.ingest(r2, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    let svg = svg_of(&ws, &[r1, r2]);
    let cells: Vec<(String, String)> = elements(&svg, "life")
        .iter()
        .filter(|e| attr(e, "data-finding") == f)
        .map(|e| (attr(e, "data-run").to_string(), attr(e, "data-state").to_string()))
        .collect();
    assert_eq!(cells, vec![(r1.to_string(), "raised".into()), (r2.to_string(), "fixed".into())]);
    // a single run draws no strip
    assert!(elements(&svg_of(&ws, &[r1]), "life").is_empty());
}

#[test]
fn a_fix_that_did_not_land_is_drawn_raised_again_and_red() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    let bad = critical("ana", [3, 3, 3], &["the Go button does nothing"], &["page.json#n1"]);
    let f = ws.ingest(r1, &bad)["findings"][0].as_str().unwrap().to_string();
    ws.cli(&["decide", &f, "fixed", "--by", "a"]);
    let r2 = ws.run("critical", &AC, false);
    ws.ingest(r2, &bad);
    let svg = svg_of(&ws, &[r1, r2]);
    let states: Vec<String> = elements(&svg, "life").iter().map(|e| attr(e, "data-state").to_string()).collect();
    assert_eq!(states, ["raised", "raised_again"]);
    let n = elements(&svg, "finding")[0];
    assert_eq!(attr(n, "data-status"), "open");
}

#[test]
fn runs_of_two_subjects_are_refused() {
    let (ws, r, _) = world(3);
    ws.subject("other", personajury::domain::SubjectKind::Page, false, &[("o.json", br#"{"nodes":[{"id":"n1"}]}"#)]);
    let r2 = ws.run_of("other", "critical", &AC, false);
    let v = ws.cli(&["graph", &r.to_string(), &r2.to_string(), "--svg"]);
    assert_eq!(v["error"]["code"], "invalid_arguments", "{v}");
}

#[test]
fn the_mcp_tool_returns_the_same_svg_and_checks_its_arguments() {
    use std::io::Write;
    let (ws, r, _) = world(3);
    let want = svg_of(&ws, &[r]);
    let lines = [
        json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"personajury_graph","arguments":{"runs":[r]}}}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"personajury_graph","arguments":{"runs":[]}}}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"personajury_graph","arguments":{"runs":[r],"out":"x.svg"}}}),
    ];
    let mut ch = std::process::Command::new(assert_cmd::cargo::cargo_bin("personajury"))
        .arg("--root")
        .arg(ws.root())
        .arg("mcp")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = ch.stdin.take().unwrap();
    for l in &lines {
        writeln!(stdin, "{l}").unwrap();
    }
    drop(stdin);
    let out = ch.wait_with_output().unwrap();
    let rows: Vec<Value> = String::from_utf8_lossy(&out.stdout).lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    let env = |i: usize| -> Value { serde_json::from_str(rows[i]["result"]["content"][0]["text"].as_str().unwrap()).unwrap() };
    assert_eq!(env(0)["data"]["svg"], want.as_str());
    assert_eq!(env(0)["verb"], "graph");
    assert_eq!(env(1)["error"]["code"], "invalid_arguments");
    assert_eq!(env(2)["error"]["code"], "invalid_arguments", "the tool writes no file");
    assert!(!ws.root().join("x.svg").exists());
}

#[test]
fn history_keeps_every_step_after_a_second_fixed_decision() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    let bad = critical("ana", [3, 3, 3], &["the Go button does nothing"], &["page.json#n1"]);
    let f = ws.ingest(r1, &bad)["findings"][0].as_str().unwrap().to_string();
    ws.cli(&["decide", &f, "fixed", "--by", "a"]);
    let r2 = ws.run("critical", &AC, false);
    ws.ingest(r2, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    let r3 = ws.run("critical", &AC, false);
    ws.ingest(r3, &bad);
    ws.cli(&["decide", &f, "fixed", "--by", "a"]);
    let c = ws.cli(&["check", "--history"]);
    assert_eq!(c["ok"], true, "{c}");
    assert_eq!(
        life(&c, &f),
        vec![(r1, "raised".to_string()), (r2, "fixed".to_string()), (r3, "raised_again".to_string())]
    );
    let svg = svg_of(&ws, &[r1, r2, r3]);
    let states: Vec<String> = elements(&svg, "life").iter().map(|e| attr(e, "data-state").to_string()).collect();
    assert_eq!(states, ["raised", "fixed", "raised_again"]);
}

#[test]
fn xml_non_characters_are_dropped_from_svg_text() {
    let ws = Ws::new();
    ws.site();
    let r = ws.run("critical", &AC, false);
    ws.ingest(r, &critical("ana", [3, 3, 3], &["bad\u{FFFE}char\u{FFFF} \u{1}x"], &["page.json#n1"]));
    let svg = svg_of(&ws, &[r]);
    assert!(!svg.contains('\u{FFFE}') && !svg.contains('\u{FFFF}') && !svg.contains('\u{1}'));
    assert!(svg.contains("badchar x"), "{svg}");
}

#[test]
fn the_life_strip_keeps_the_last_eight_rounds_with_a_mark_for_the_rest() {
    let ws = Ws::new();
    ws.site();
    let bad = critical("ana", [3, 3, 3], &["the Go button does nothing"], &["page.json#n1"]);
    let mut runs = Vec::new();
    for _ in 0..20 {
        let r = ws.run("critical", &AC, false);
        ws.ingest(r, &bad);
        runs.push(r);
    }
    let svg = svg_of(&ws, &runs);
    let dots = elements(&svg, "life");
    assert_eq!(dots.len(), 8, "{}", dots.len());
    assert_eq!(attr(dots[7], "data-run"), runs[19].to_string());
    assert_eq!(attr(dots[0], "data-run"), runs[12].to_string());
    assert!(svg.contains(">+12</text>") && svg.contains("12 earlier"), "mark");
    let x0: i64 = attr(dots[0], "cx").parse().unwrap();
    assert!(x0 >= 966 + 40, "dots stay inside the finding box: {x0}");
}

#[test]
fn above_thirty_personas_the_strata_sit_side_by_side() {
    let ws = Ws::new();
    ws.site();
    let names: Vec<String> = (0..31).map(|i| format!("p{i:02}")).collect();
    let personas: Vec<(&str, Stake)> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), if i < 12 { Stake::Primary } else if i < 22 { Stake::Secondary } else { Stake::Bystander }))
        .collect();
    let r = ws.run("critical", &personas, false);
    for n in &names {
        ws.ingest(r, &critical(n, [3, 3, 3], &[], &["page.json#n1"]));
    }
    let svg = svg_of(&ws, &[r]);
    let boxes: Vec<(i64, i64)> = elements(&svg, "persona").iter().map(|e| (attr(e, "x").parse().unwrap(), attr(e, "y").parse().unwrap())).collect();
    let mut tops: std::collections::BTreeMap<i64, i64> = Default::default();
    for (x, y) in &boxes {
        let t = tops.entry(*x).or_insert(*y);
        *t = (*t).min(*y);
    }
    assert_eq!(tops.len(), 3);
    let v: Vec<i64> = tops.values().copied().collect();
    assert!(v[0] == v[1] && v[1] == v[2], "all columns start at the same height: {tops:?}");
    let height: i64 = attr(svg.lines().next().unwrap(), "height").parse().unwrap();
    assert!(height < 1000, "height follows the tallest column: {height}");
}
