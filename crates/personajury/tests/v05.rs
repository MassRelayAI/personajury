//! v0.5 (D20..D23): calibration, CSV cell locators, the GitHub Action, the Agent SDK recipe. Every test was seen red first.
mod judging_support;
use personajury::domain::Stake;
use judging_support::*;
use serde_json::{json, Value};

fn data(v: Value) -> Value {
    assert_eq!(v["ok"], true, "{v}");
    v["data"].clone()
}
fn code(v: &Value) -> String {
    assert_eq!(v["ok"], false, "{v}");
    v["error"]["code"].as_str().unwrap().to_string()
}
fn rs(n: i64) -> String {
    n.to_string()
}
fn by_human(ws: &Ws, run: i64, v: &Value) -> Value {
    ws.cli(&["verdict", "--run", &rs(run), "--json", &v.to_string(), "--by-human", "Dr Rao, ward nurse"])
}
const LOW_RULE: &str = "[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/site/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n";

// ---- item 1: calibration (D20) ----

const PB: [(&str, Stake); 2] = [("ana", Stake::Primary), ("bob", Stake::Bystander)];

/// A jury run scored `jury` by ana (primary) and `jury_b` by bob (bystander), and a human run scored `human` by a person
/// speaking from ana's seat and `human_b` from bob's. Returns (jury run, human run).
fn pair(ws: &Ws, jury: [i64; 3], jury_b: [i64; 3], human: [i64; 3], human_b: [i64; 3]) -> (i64, i64) {
    ws.site();
    let j = ws.run("critical", &PB, false);
    ws.ingest(j, &critical("ana", jury, &[], &["page.json#n1"]));
    ws.ingest(j, &critical("bob", jury_b, &[], &["page.json#n1"]));
    let h = ws.run("critical", &PB, false);
    data(by_human(ws, h, &critical("ana", human, &[], &["page.json#n1"])));
    data(by_human(ws, h, &critical("bob", human_b, &[], &["page.json#n1"])));
    (j, h)
}
fn cell(c: &Value, stake: &str, dim: &str) -> Value {
    let s = c["strata"].as_array().unwrap().iter().find(|s| s["stake"] == stake).unwrap();
    s["cells"].as_array().unwrap().iter().find(|x| x["dimension"] == dim).unwrap().clone()
}
fn calibrate(ws: &Ws, j: i64, h: i64) -> Value {
    data(ws.cli(&["calibrate", &rs(j), &rs(h)]))
}

#[test]
fn identical_rank_orders_are_calibrated() {
    let ws = Ws::new();
    let (j, h) = pair(&ws, [5, 4, 3], [5, 4, 3], [5, 4, 3], [5, 4, 3]);
    let c = calibrate(&ws, j, h);
    assert_eq!(c["calibrated"], true, "{c}");
    let cells = c["strata"][0]["cells"].as_array().unwrap();
    assert_eq!(c["strata"][0]["stake"], "primary", "{c}");
    assert_eq!(cells.len(), 3, "{c}");
    assert!(cells.iter().all(|x| x["within_one_rank"] == true), "{c}");
}

#[test]
fn one_adjacent_swap_is_within_one_rank() {
    let ws = Ws::new();
    let (j, h) = pair(&ws, [5, 4, 3], [5, 4, 3], [4, 5, 3], [5, 4, 3]);
    let c = calibrate(&ws, j, h);
    assert_eq!(c["calibrated"], true, "{c}");
    let w = cell(&c, "primary", "works");
    assert_eq!((w["jury_rank"].as_i64(), w["human_rank"].as_i64()), (Some(1), Some(2)), "{c}");
}

#[test]
fn a_far_swap_on_the_primary_stratum_is_not_calibrated() {
    let ws = Ws::new();
    let (j, h) = pair(&ws, [5, 4, 3], [5, 4, 3], [3, 4, 5], [5, 4, 3]);
    let c = calibrate(&ws, j, h);
    assert_eq!(c["calibrated"], false, "{c}");
    assert_eq!(cell(&c, "primary", "works")["within_one_rank"], false, "{c}");
}

#[test]
fn a_far_swap_only_on_a_bystander_is_calibrated_but_reported() {
    let ws = Ws::new();
    let (j, h) = pair(&ws, [5, 4, 3], [5, 4, 3], [5, 4, 3], [3, 4, 5]);
    let c = calibrate(&ws, j, h);
    assert_eq!(c["calibrated"], true, "{c}");
    let off = c["outside_primary"].as_array().unwrap();
    assert!(!off.is_empty() && off.iter().all(|x| x["stake"] == "bystander"), "{c}");
    // no number spans the strata
    assert!(c.get("agreement").is_none() && c.get("rho").is_none(), "{c}");
}

#[test]
fn ties_share_a_rank() {
    let ws = Ws::new();
    let (j, h) = pair(&ws, [4, 4, 3], [5, 4, 3], [4, 4, 3], [5, 4, 3]);
    let c = calibrate(&ws, j, h);
    let ranks: Vec<i64> = ["works", "blocks", "missing"].iter().map(|d| cell(&c, "primary", d)["jury_rank"].as_i64().unwrap()).collect();
    assert_eq!(ranks, [1, 1, 3], "{c}");
}

#[test]
fn a_run_without_a_human_batch_is_not_a_human_run() {
    let ws = Ws::new();
    ws.site();
    let j = ws.run("critical", &PB, false);
    ws.ingest(j, &critical("ana", [5, 4, 3], &[], &["page.json#n1"]));
    let k = ws.run("critical", &PB, false);
    ws.ingest(k, &critical("ana", [5, 4, 3], &[], &["page.json#n1"]));
    assert_eq!(code(&ws.cli(&["calibrate", &rs(j), &rs(k)])), "not_human_run");
}

#[test]
fn a_primary_cell_the_humans_did_not_judge_is_not_proven() {
    let ws = Ws::new();
    ws.site();
    let j = ws.run("critical", &PB, false);
    ws.ingest(j, &critical("ana", [5, 4, 3], &[], &["page.json#n1"]));
    let h = ws.run("critical", &PB, false);
    // the human judges bob's seat only: nothing to compare in the primary stratum
    data(by_human(&ws, h, &critical("bob", [5, 4, 3], &[], &["page.json#n1"])));
    let c = calibrate(&ws, j, h);
    assert_eq!(c["calibrated"], false, "{c}");
}

#[test]
fn out_writes_the_answer_and_apply_check_reads_it() {
    let ws = Ws::new();
    let (j, h) = pair(&ws, [5, 4, 3], [5, 4, 3], [3, 4, 5], [5, 4, 3]);
    let f = ws.root().join("cal.json");
    data(ws.cli(&["calibrate", &rs(j), &rs(h), "--out", f.to_str().unwrap()]));
    let saved: Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
    assert_eq!(saved["calibrated"], false);
    // a clean low-rule run (a workspace of its own: the human run above would supersede it) applies on its own ...
    let ws = Ws::new();
    ws.site();
    let j = ws.run("critical", &PB, false);
    ws.ingest(j, &critical("ana", [5, 4, 3], &[], &["page.json#n1"]));
    ws.ingest(j, &critical("bob", [5, 4, 3], &[], &["page.json#n1"]));
    std::fs::write(ws.root().join("personajury.toml"), LOW_RULE).unwrap();
    ws.store.conn().execute("UPDATE run SET rule_class = 'public-surface', quorum = 'low' WHERE id = ?1", [j]).unwrap();
    let a = data(ws.cli(&["apply-check", &rs(j)]));
    assert_eq!(a["answer"], "apply", "{a}");
    // ... and advises once the calibration says the jury does not rank like the humans
    let a = data(ws.cli(&["apply-check", &rs(j), "--calibration", f.to_str().unwrap()]));
    assert_eq!(a["answer"], "advise", "{a}");
    let r = a["reasons"].as_array().unwrap();
    assert!(r.iter().any(|x| x["code"] == "uncalibrated"), "{a}");
    // a calibrated file leaves apply alone (the file names this subject, lens, lens version and run: M1)
    let lv = ws.store.lens("critical").unwrap().unwrap().version;
    let cal = |ok: bool| json!({"calibrated": ok, "subject": "site", "lens": "critical", "lens_version": lv, "jury_run": j}).to_string();
    std::fs::write(&f, cal(false)).unwrap();
    let a = data(ws.cli(&["apply-check", &rs(j), "--calibration", f.to_str().unwrap()]));
    assert_eq!(a["answer"], "advise", "{a}");
    std::fs::write(&f, cal(true)).unwrap();
    let a = data(ws.cli(&["apply-check", &rs(j), "--calibration", f.to_str().unwrap()]));
    assert_eq!(a["answer"], "apply", "{a}");
    // a file that is not a calibration is refused
    std::fs::write(&f, "{}").unwrap();
    assert_eq!(code(&ws.cli(&["apply-check", &rs(j), "--calibration", f.to_str().unwrap()])), "invalid_arguments");
}

// ---- item 2: CSV cell locators (D21) ----

#[test]
fn a_verdict_cites_a_csv_cell_and_an_out_of_range_cell_is_refused() {
    let ws = Ws::new();
    ws.subject(
        "sheet",
        personajury::domain::SubjectKind::Prices,
        false,
        &[("prices.csv", b"plan,price\n\"Team, annual\",12\n\"two\nlines\",7\n")],
    );
    let run = ws.run_of("sheet", "critical", &[("ana", Stake::Primary)], false);
    let sheet = |cites: &[&str]| {
        let lines: Vec<Value> = ["works", "blocks", "missing"]
            .iter()
            .map(|d| json!({"dimension": d, "score": 4, "anchor_met": 3, "evidence": "seen", "citations": cites, "blockers": []}))
            .collect();
        json!({"persona": "ana", "lines": lines})
    };
    let ok = sheet(&["prices.csv#R2C2", "prices.csv#R3C1", "prices.csv#L4"]);
    let r = ws.cli(&["verdict", "--run", &rs(run), "--json", &ok.to_string()]);
    assert_eq!(r["ok"], true, "{r}");
    for bad in ["prices.csv#R4C1", "prices.csv#R2C3", "prices.csv#R01C1"] {
        let r = ws.cli(&["verdict", "--run", &rs(run), "--json", &sheet(&[bad]).to_string()]);
        assert_eq!(code(&r), "citation_unresolved", "{bad}: {r}");
    }
}

// ---- item 3: apply-check --latest (what the Action reads from a committed store) ----

#[test]
fn apply_check_latest_judges_the_subject_of_the_newest_run() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &TWO_STAKES, false);
    ws.ingest(r1, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(r1, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let a = data(ws.cli(&["apply-check", "--latest"]));
    assert_eq!(a["runs"], json!([r1]), "{a}");
    // naming runs and --latest together is refused, and so is naming neither
    assert_eq!(code(&ws.cli(&["apply-check", "1", "--latest"])), "invalid_arguments");
    let e = std::process::Command::new(assert_cmd::cargo::cargo_bin("personajury")).arg("--root").arg(ws.root()).arg("apply-check").output().unwrap();
    assert!(!e.status.success());
    // an empty store has nothing to judge
    let empty = Ws::new();
    assert_eq!(code(&empty.cli(&["apply-check", "--latest"])), "not_found");
}
const TWO_STAKES: [(&str, Stake); 2] = [("ana", Stake::Primary), ("bob", Stake::Secondary)];

// ---- item 4: the Claude Agent SDK recipe (D23): docs and one example, never run here (it needs a key) ----

fn root() -> std::path::PathBuf {
    judging_support::repo().canonicalize().unwrap()
}
#[test]
fn the_agent_sdk_recipe_exists_and_the_example_parses() {
    let doc = std::fs::read_to_string(root().join("docs/agent-sdk.md")).expect("docs/agent-sdk.md");
    for w in ["personajury mcp", "mcp_servers", "one context", "private", "apply-check", "check the SDK version"] {
        assert!(doc.contains(w), "docs/agent-sdk.md lacks `{w}`");
    }
    let py = root().join("examples/agent-sdk/run_jury.py");
    let src = std::fs::read_to_string(&py).expect("examples/agent-sdk/run_jury.py");
    // no key, and the script itself reads no environment: the SDK reads its own
    for bad in ["sk-ant", "api_key", "API_KEY", "os.environ", "getenv"] {
        assert!(!src.contains(bad), "the example mentions {bad}");
    }
    assert!(src.contains("claude_agent_sdk") && src.contains("mcp_servers") && src.contains("personajury_apply_check"));
    // it compiles; skipped with a note when there is no python3
    match std::process::Command::new("python3").arg("-I").arg("-m").arg("py_compile").arg(&py).output() {
        Ok(o) => assert!(o.status.success(), "py_compile: {}", String::from_utf8_lossy(&o.stderr)),
        Err(_) => eprintln!("python3 not found: the example's syntax check is skipped"),
    }
    let _ = std::fs::remove_dir_all(root().join("examples/agent-sdk/__pycache__"));
}

// ---- the docs blocks, the tradeoffs lens and its playbook ----

#[test]
fn the_tradeoffs_lens_is_accepted_by_lens_add_and_asks_for_a_pick() {
    let ws = Ws::new();
    let f = root().join("fixtures/lenses/tradeoffs.toml");
    let r = data(ws.cli(&["lens", "add", "--file", f.to_str().unwrap()]));
    assert_eq!(r["name"], "tradeoffs", "{r}");
    let text = std::fs::read_to_string(&f).unwrap();
    assert!(text.contains("asks_pick = true") && text.to_lowercase().contains("correctness is for tests"), "the comment says correctness is for tests");
    for d in ["fit", "risk", "cost-to-change", "operability", "learning-curve"] {
        assert!(text.contains(&format!("id = \"{d}\"")), "dimension {d}");
    }
    assert_eq!(text.matches("[[dimension]]").count(), 5);
    let lens = ws.cli(&["lens", "list"]);
    assert!(lens.to_string().contains("tradeoffs"), "{lens}");
}

#[test]
fn the_docs_say_that_display_is_free_and_the_verdict_is_not_and_what_goes_to_a_jury() {
    let read = |p: &str| -> String {
        let t = std::fs::read_to_string(root().join(p)).unwrap_or_else(|e| panic!("{p}: {e}"));
        // wrapped lines do not hide a phrase
        t.split_whitespace().collect::<Vec<_>>().join(" ")
    };
    for p in ["skills/running-a-jury/SKILL.md", "docs/usage.md"] {
        let t = read(p);
        for w in ["Display is free, the verdict is not", "never pool strata", "never write a number the data lacks", "pre-merge", "What goes to a jury", "never correctness", "unmatched"] {
            assert!(t.contains(w), "{p} lacks `{w}`");
        }
    }
    let u = read("docs/usage.md");
    assert!(u.contains("Where results live") && u.contains("report_dir"), "usage.md: where results live");
    let s = read("skills/running-a-jury/SKILL.md");
    for w in ["Do people with different stakes", "judgement", "costly to undo", "summary", "calibrate", "--save", "action.yml"] {
        assert!(s.contains(w), "the skill lacks `{w}`");
    }
    // the three-question rule stays short
    let raw = std::fs::read_to_string(root().join("skills/running-a-jury/SKILL.md")).unwrap();
    let a = raw.find("1. Do people with different stakes").unwrap();
    let z = raw[a..].find("it is reversible.").unwrap() + "it is reversible.".len();
    assert!(raw[a..a + z].lines().count() <= 8, "the rule is under eight lines");
    let p = read("docs/playbooks/a-technical-decision.md");
    for w in ["tradeoffs", "maintainer", "SRE", "finance approver", "#L<n>", "decide", "tests"] {
        assert!(p.contains(w), "playbook lacks `{w}`");
    }
    assert!(read("docs/playbooks/README.md").contains("a-technical-decision.md"));
    for (p, w) in [("docs/commands.md", "calibrate"), ("docs/commands.md", "summary"), ("docs/mcp-tools.md", "personajury_summary"), ("CHANGELOG.md", "[0.5.0]"), ("docs/decisions.md", "## D24")] {
        assert!(read(p).contains(w), "{p} lacks {w}");
    }
}
