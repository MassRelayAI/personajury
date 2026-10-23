//! Task 5: verdict, matrix, pick, delta, check, decide and should-run through the CLI. Runs are inserted straight
//! into the store; the `brief` verb is not a dependency.
#[path = "judging_support/mod.rs"]
mod support;
use jurygraph::domain::{Stake, SubjectKind};
use serde_json::{json, Value};
use support::*;

#[test]
fn verdict_ingest_reports_run_persona_batch_and_findings() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    let d = ws.ingest(
        run,
        &critical(
            "ana",
            [3, 2, 4],
            &["cannot find the price"],
            &["page.json#n2"],
        ),
    );
    assert_eq!(d["run"], run);
    assert_eq!(d["persona"], "ana");
    assert_eq!(d["stake"], "primary");
    assert_eq!(d["batch"], 1);
    assert_eq!(d["lines"], 3);
    assert_eq!(d["findings"].as_array().unwrap().len(), 1);
    assert_eq!(d["findings"][0].as_str().unwrap().len(), 16);
    let d2 = ws.ingest(run, &critical("ana", [3, 2, 4], &[], &["page.json#n2"]));
    assert_eq!(d2["batch"], 2);
    assert_eq!(d2["findings"], json!([]));
}

#[test]
fn verdict_malformed_json_is_invalid_arguments() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    let r = ws.cli(&["verdict", "--run", &run.to_string(), "--json", "{not json"]);
    assert_eq!(r["ok"], false);
    assert_eq!(r["error"]["code"], "invalid_arguments");
    assert!(
        r["error"]["message"]
            .as_str()
            .unwrap()
            .starts_with("verdict JSON: "),
        "{r}"
    );
    // an unknown field is malformed too
    let r = ws.cli(&[
        "verdict",
        "--run",
        &run.to_string(),
        "--json",
        r#"{"persona":"ana","lines":[],"extra":1}"#,
    ]);
    assert_eq!(r["error"]["code"], "invalid_arguments");
}

#[test]
fn verdict_on_missing_run_is_not_found() {
    let ws = Ws::new();
    let r = ws.cli(&["verdict", "--run", "99", "--json", "{}"]);
    assert_eq!(r["error"]["code"], "not_found");
}

#[test]
fn latest_ingest_wins_history_shows_all() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [2, 2, 2], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    let r = ws.cli(&["matrix", &run.to_string(), "--detail", "full"]);
    assert_eq!(r["ok"], true, "{r}");
    let cell = &r["data"]["strata"][0]["cells"][0];
    assert_eq!(cell["dimension"], "works");
    assert_eq!(cell["scores"]["ana"], 4);
    assert_eq!(cell["n"], 1);
    assert!(r["data"].get("history").is_none());
    let h = ws.cli(&["matrix", &run.to_string(), "--history"]);
    let rows = h["data"]["history"].as_array().unwrap();
    assert_eq!(rows.len(), 6, "{h}");
    let scores: Vec<i64> = rows
        .iter()
        .filter(|r| r["dimension"] == "works")
        .map(|r| r["score"].as_i64().unwrap())
        .collect();
    assert_eq!(scores, vec![2, 4]);
}

#[test]
fn matrix_per_stratum_table() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(
        run,
        &critical("ana", [4, 3, 2], &["slow"], &["page.json#n1"]),
    );
    ws.ingest(run, &critical("bob", [2, 2, 2], &[], &["page.json#n1"]));
    let r = ws.cli(&["matrix", &run.to_string(), "--table", "--detail", "full"]);
    assert_eq!(r["ok"], true, "{r}");
    let strata = r["data"]["strata"].as_array().unwrap();
    assert_eq!(strata.len(), 2);
    assert_eq!(strata[0]["stake"], "primary");
    assert_eq!(strata[1]["stake"], "secondary");
    assert_eq!(strata[0]["cells"][0]["median"], 4);
    assert!(
        strata[0]["cells"][0]["median"].is_i64(),
        "an integer, never a float"
    );
    assert_eq!(strata[0]["cells"][0]["blockers"], 1);
    assert_eq!(strata[1]["cells"][0]["median"], 2);
    let t = r["data"]["table"].as_str().unwrap();
    assert!(t.contains("| primary | ana |"), "{t}");
    assert!(t.contains("| secondary | bob |"), "{t}");
    assert!(t.contains("| primary | median |"), "{t}");
    assert!(t.contains("| secondary | median |"), "{t}");
    // never one number across strata
    let text = r["data"].to_string();
    assert!(
        !text.contains("mean") && !text.contains("average"),
        "{text}"
    );
    assert_eq!(r["data"]["runs"], json!([run]));
}

#[test]
fn matrix_compact_omits_persona_rows() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [4, 3, 2], &[], &["page.json#n1"]));
    let r = ws.cli(&["matrix", &run.to_string()]);
    assert_eq!(r["ok"], true, "{r}");
    for c in r["data"]["strata"][0]["cells"].as_array().unwrap() {
        assert_eq!(c["scores"], json!({}), "{c}");
        assert!(
            c["median"].is_number()
                && c["min"].is_number()
                && c["max"].is_number()
                && c["n"].is_number()
        );
    }
    assert!(r["data"].get("table").is_none());
    // the table is built from the full matrix even when the answer is compact
    let t = ws.cli(&["matrix", &run.to_string(), "--table"]);
    assert!(t["data"]["table"]
        .as_str()
        .unwrap()
        .contains("| primary | ana |"));
    assert_eq!(t["data"]["strata"][0]["cells"][0]["scores"], json!({}));
}

#[test]
fn matrix_over_two_runs_merges_by_stratum() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &[("ana", Stake::Primary)], false);
    let r2 = ws.run("critical", &[("bob", Stake::Secondary)], false);
    ws.ingest(r1, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(r2, &critical("bob", [2, 2, 2], &[], &["page.json#n1"]));
    let r = ws.cli(&[
        "matrix",
        &r1.to_string(),
        &r2.to_string(),
        "--detail",
        "full",
    ]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"]["strata"].as_array().unwrap().len(), 2);
    assert_eq!(r["data"]["runs"], json!([r1, r2]));
}

#[test]
fn matrix_one_stake_per_persona_across_runs() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &[("ana", Stake::Primary)], false);
    let r2 = ws.run("critical", &[("ana", Stake::Bystander)], false);
    let r = ws.cli(&["matrix", &r1.to_string(), &r2.to_string()]);
    assert_eq!(r["error"]["code"], "invalid_arguments");
    assert!(r["error"]["message"]
        .as_str()
        .unwrap()
        .contains("one stake per persona in a matrix"));
}

#[test]
fn pick_tallies_per_stratum() {
    let ws = Ws::new();
    ws.site();
    ws.rival();
    let p = [
        ("ana", Stake::Primary),
        ("bob", Stake::Primary),
        ("dan", Stake::Primary),
        ("cat", Stake::Secondary),
    ];
    let run = ws.run("compare", &p, true);
    ws.ingest(run, &compare("ana", "page.json", "clearer"));
    ws.ingest(run, &compare("bob", "cmp:rival", "cheaper"));
    ws.ingest(run, &compare("dan", "page.json", "faster"));
    ws.ingest(run, &compare("cat", "cmp:rival", "known brand"));
    let r = ws.cli(&["pick", &run.to_string()]);
    assert_eq!(r["ok"], true, "{r}");
    let s = r["data"]["strata"].as_array().unwrap();
    assert_eq!(s.len(), 2);
    assert_eq!(s[0]["stake"], "primary");
    assert_eq!(s[0]["choices"][0]["choice"], "page.json");
    assert_eq!(s[0]["choices"][0]["count"], 2);
    assert_eq!(s[0]["choices"][0]["personas"], json!(["ana", "dan"]));
    assert_eq!(
        s[0]["choices"][0]["reasons"][0],
        json!({"persona": "ana", "reason": "clearer"})
    );
    assert_eq!(s[0]["choices"][1]["choice"], "cmp:rival");
    assert_eq!(s[0]["choices"][1]["count"], 1);
    assert_eq!(s[1]["stake"], "secondary");
    assert_eq!(s[1]["choices"][0]["choice"], "cmp:rival");
    assert_eq!(r["data"]["run"], run);
}

#[test]
fn pick_accepts_an_attachment_label_and_refuses_unknown() {
    let ws = Ws::new();
    ws.site();
    ws.rival();
    let run = ws.run("compare", &TWO, true);
    let r = ws.cli(&[
        "verdict",
        "--run",
        &run.to_string(),
        "--json",
        &compare("ana", "nowhere", "x").to_string(),
    ]);
    assert_eq!(r["error"]["code"], "pick_unknown_option");
    assert!(
        r["error"]["message"]
            .as_str()
            .unwrap()
            .contains("change.diff, page.json, shot.png, cmp:rival"),
        "{r}"
    );
}

#[test]
fn delta_reports_not_landing() {
    let ws = Ws::new();
    ws.site();
    let a = ws.run("critical", &[("ana", Stake::Primary)], false);
    let b = ws.run("critical", &[("ana", Stake::Primary)], false);
    ws.ingest(a, &critical("ana", [4, 4, 4], &["slow"], &["page.json#n1"]));
    // blockers not down and works down by 3 (> one anchor step of 2)
    ws.ingest(b, &critical("ana", [1, 4, 4], &["slow"], &["page.json#n1"]));
    let r = ws.cli(&["delta", &a.to_string(), &b.to_string()]);
    assert_eq!(r["ok"], true, "{r}");
    let d = &r["data"];
    assert_eq!(d["landed"], false);
    assert_eq!(d["blockers_before"], 1);
    assert_eq!(d["blockers_after"], 1);
    assert_eq!(d["regressions"].as_array().unwrap().len(), 1);
    assert_eq!(d["regressions"][0]["dimension"], "works");
    assert_eq!(d["step"], 2);
    assert_eq!((d["a"].as_i64(), d["b"].as_i64()), (Some(a), Some(b)));
    assert_eq!(d["subject"], "site");
    assert_eq!(d["lens"], "critical");
}

#[test]
fn delta_lands_when_blockers_fall_and_nothing_regresses() {
    let ws = Ws::new();
    ws.site();
    let a = ws.run("critical", &[("ana", Stake::Primary)], false);
    let b = ws.run("critical", &[("ana", Stake::Primary)], false);
    ws.ingest(a, &critical("ana", [3, 3, 3], &["slow"], &["page.json#n1"]));
    ws.ingest(b, &critical("ana", [4, 3, 3], &[], &["page.json#n1"]));
    let r = ws.cli(&["delta", &a.to_string(), &b.to_string()]);
    assert_eq!(r["data"]["landed"], true, "{r}");
}

#[test]
fn matrix_scores_each_option() {
    let ws = Ws::new();
    ws.site();
    ws.rival();
    let run = ws.run("compare", &TWO, true);
    let mut v = compare("ana", "page.json", "r");
    for (about, score) in [("shot.png", 4), ("page.json", 2)] {
        v["lines"].as_array_mut().unwrap().push(json!({
            "dimension": "clarity", "score": score, "anchor_met": 1, "evidence": "seen", "about": about,
            "citations": [about, "cmp:rival/rival.json#r1"],
        }));
    }
    ws.ingest(run, &v);
    let r = ws.cli(&["matrix", &run.to_string(), "--table", "--detail", "full"]);
    assert_eq!(r["ok"], true, "{r}");
    let cells = r["data"]["strata"][0]["cells"].as_array().unwrap();
    let keys: Vec<(String, Value, Value)> = cells
        .iter()
        .map(|c| {
            (
                c["dimension"].as_str().unwrap().to_string(),
                c["about"].clone(),
                c["median"].clone(),
            )
        })
        .collect();
    assert_eq!(
        keys[..3],
        [
            ("clarity".to_string(), Value::Null, json!(3)),
            ("clarity".to_string(), json!("page.json"), json!(2)),
            ("clarity".to_string(), json!("shot.png"), json!(4)),
        ]
    );
    let t = r["data"]["table"].as_str().unwrap();
    assert!(
        t.starts_with("| stratum | persona | compare:clarity | compare:clarity@page.json | compare:clarity@shot.png | compare:price |"),
        "{t}"
    );
    // the delta keeps each option apart
    let run2 = ws.run("compare", &TWO, true);
    v["lines"][4]["score"] = json!(4);
    ws.ingest(run2, &v);
    let d = ws.cli(&["delta", &run.to_string(), &run2.to_string()]);
    let moved: Vec<(Value, Value)> = d["data"]["moves"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["dimension"] == "clarity")
        .map(|m| (m["about"].clone(), m["moved"].clone()))
        .collect();
    assert_eq!(
        moved,
        vec![
            (Value::Null, json!(0)),
            (json!("page.json"), json!(2)),
            (json!("shot.png"), json!(0))
        ]
    );
}

#[test]
fn delta_refuses_lens_version_change() {
    let ws = Ws::new();
    ws.site();
    let a = ws.run("critical", &TWO, false);
    let mut l = ws.store.lens("critical").unwrap().unwrap();
    l.dimensions[0]
        .anchors
        .insert("5".into(), "rewritten between the runs".into());
    assert_eq!(ws.store.insert_lens(&l).unwrap(), 2);
    let b = ws.run("critical", &TWO, false);
    let r = ws.cli(&["delta", &a.to_string(), &b.to_string()]);
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["code"], "invalid_arguments");
    let msg = r["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("critical v1") && msg.contains("critical v2"),
        "{msg}"
    );
}

#[test]
fn delta_mismatch_refused() {
    let ws = Ws::new();
    ws.site();
    ws.subject(
        "other",
        SubjectKind::Page,
        false,
        &[("page.json", br#"{"nodes":[]}"#)],
    );
    let a = ws.run("critical", &TWO, false);
    let b = ws.run_of("other", "critical", &TWO, false);
    let c = ws.run("taste", &TWO, false);
    for other in [b, c] {
        let r = ws.cli(&["delta", &a.to_string(), &other.to_string()]);
        assert_eq!(r["ok"], false);
        assert_eq!(r["error"]["code"], "invalid_arguments");
        assert!(r["error"]["message"]
            .as_str()
            .unwrap()
            .starts_with("delta needs two runs on the same subject and lens"));
    }
    let r = ws.cli(&["delta", &a.to_string(), "77"]);
    assert_eq!(r["error"]["code"], "not_found");
}

fn first_finding(ws: &Ws, run: i64, persona: &str, blocker: &str, cite: &[&str]) -> String {
    let d = ws.ingest(run, &critical(persona, [3, 3, 3], &[blocker], cite));
    d["findings"][0].as_str().unwrap().to_string()
}

#[test]
fn decide_dismissed_needs_why() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    let f = first_finding(&ws, run, "ana", "slow", &["page.json#n1"]);
    for why in [None, Some("  ")] {
        let mut args = vec!["decide", &f, "dismissed", "--by", "nick"];
        if let Some(w) = why {
            args.extend(["--why", w]);
        }
        let r = ws.cli(&args);
        assert_eq!(r["error"]["code"], "invalid_arguments", "{r}");
    }
    let r = ws.cli(&[
        "decide",
        &f,
        "dismissed",
        "--why",
        "intended",
        "--by",
        "nick",
    ]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"]["finding"], f.as_str());
    assert_eq!(r["data"]["outcome"], "dismissed");
    assert!(r["data"]["decision_id"].as_i64().unwrap() >= 1);
}

#[test]
fn decide_row_needs_row_and_by_and_known_outcome_and_finding() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    let f = first_finding(&ws, run, "ana", "slow", &["page.json#n1"]);
    assert_eq!(
        ws.cli(&["decide", &f, "row", "--by", "n"])["error"]["code"],
        "invalid_arguments"
    );
    assert_eq!(
        ws.cli(&["decide", &f, "row", "--row", "G12", "--by", " "])["error"]["code"],
        "invalid_arguments"
    );
    assert_eq!(
        ws.cli(&["decide", &f, "maybe", "--by", "n"])["error"]["code"],
        "invalid_arguments"
    );
    assert_eq!(
        ws.cli(&["decide", "ffffffffffffffff", "fixed", "--by", "n"])["error"]["code"],
        "not_found"
    );
    assert_eq!(
        ws.cli(&["decide", &f, "row", "--row", "G12", "--by", "n"])["ok"],
        true
    );
    assert_eq!(ws.cli(&["decide", &f, "fixed", "--by", "n"])["ok"], true);
}

#[test]
fn check_passes_once_every_finding_has_a_decision() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    let f = first_finding(&ws, run, "ana", "slow", &["page.json#n1"]);
    ws.ingest(run, &critical("bob", [3, 3, 3], &[], &["page.json#n1"]));
    assert_eq!(ws.cli(&["check"])["ok"], false);
    ws.cli(&["decide", &f, "fixed", "--by", "n"]);
    let r = ws.cli(&["check"]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"]["unresolved"], json!([]));
    assert_eq!(r["data"]["runs"], 1);
    assert!(r["data"].get("decisions").is_none());
    let h = ws.cli(&["check", "--history"]);
    assert_eq!(h["data"]["decisions"].as_array().unwrap().len(), 1);
}

#[test]
fn check_lists_fixed_unverified() {
    let ws = Ws::new();
    ws.site();
    ws.subject(
        "other",
        SubjectKind::Page,
        false,
        &[("page.json", br#"{"nodes":[{"id":"n1"}]}"#)],
    );
    let run = ws.run("critical", &TWO, false);
    let f = first_finding(&ws, run, "ana", "slow", &["page.json#n1"]);
    let g = first_finding(&ws, run, "bob", "no search", &["page.json#n1"]);
    ws.cli(&["decide", &f, "fixed", "--by", "n"]);
    ws.cli(&["decide", &g, "row", "--row", "G1", "--by", "n"]);
    let r = ws.cli(&["check"]);
    assert_eq!(r["ok"], true, "listed, never failed: {r}");
    let fu = r["data"]["fixed_unverified"].as_array().unwrap();
    assert_eq!(fu.len(), 1, "{r}");
    assert_eq!(fu[0]["finding"], f.as_str());
    assert_eq!(fu[0]["run"], run);
    assert_eq!(fu[0]["subject"], "site");
    assert_eq!(fu[0]["persona"], "ana");
    // a later run of another subject does not verify it
    ws.run_of("other", "critical", &TWO, false);
    assert_eq!(
        ws.cli(&["check"])["data"]["fixed_unverified"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    // a later run of the same subject and lens does, once the persona that raised it has judged it (I5)
    let later = ws.run("critical", &TWO, false);
    assert_eq!(
        ws.cli(&["check"])["data"]["fixed_unverified"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    ws.ingest(later, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    assert_eq!(ws.cli(&["check"])["data"]["fixed_unverified"], json!([]));
}

#[test]
fn check_reports_stale_runs_without_failing() {
    let ws = Ws::new();
    let paths = ws.site();
    let run = ws.run("critical", &TWO, false);
    std::fs::write(&paths["change.diff"], "edited\n").unwrap();
    let r = ws.cli(&["check"]);
    assert_eq!(r["ok"], true, "{r}");
    let s = &r["data"]["stale"][0];
    assert_eq!(
        (s["run"].as_i64(), s["subject"].as_str()),
        (Some(run), Some("site"))
    );
    assert_eq!(s["why"], "change.diff changed");
    std::fs::remove_file(&paths["shot.png"]).unwrap();
    let r = ws.cli(&["check"]);
    let whys: Vec<String> = r["data"]["stale"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["why"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(whys.len(), 1, "one entry per run: {whys:?}");
}

fn should_run(root: &std::path::Path, extra: &[&str]) -> Value {
    let cfg = fixtures().join("jurygraph.toml");
    let mut args = vec!["should-run", "--config", cfg.to_str().unwrap()];
    args.extend(extra);
    cli_in(root, &args)
}

#[test]
fn should_run_internal_is_none() {
    let d = tempfile::tempdir().unwrap();
    let r = should_run(
        d.path(),
        &["--paths", "crates/jurygraph/src/lib.rs,tests/x.rs"],
    );
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"]["quorum"], "none");
    assert_eq!(r["data"]["spend"], json!([]));
    assert_eq!(r["data"]["hits"][0]["class"], "internals");
    assert_eq!(r["data"]["unmatched"], json!([]));
    assert!(
        !d.path().join(".jurygraph").exists(),
        "should-run needs no store"
    );
}

#[test]
fn should_run_public_is_high_with_spend() {
    let d = tempfile::tempdir().unwrap();
    let r = should_run(
        d.path(),
        &["--paths", "site/index.html,crates/a.rs,README.md"],
    );
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"]["quorum"], "high");
    assert_eq!(r["data"]["unmatched"], json!(["README.md"]));
    let s = &r["data"]["spend"];
    assert_eq!(s.as_array().unwrap().len(), 1);
    assert_eq!(s[0]["class"], "public-surface");
    assert_eq!(s[0]["panel"], "fixture");
    assert_eq!(s[0]["personas"], "full panel fixture");
    assert_eq!(s[0]["lenses"], 4);
    assert_eq!(
        s[0]["detail"], "compact",
        "a rule without detail is compact"
    );
    assert!(s[0].get("briefs").is_none());
    assert!(
        s[0].get("tokens").is_none(),
        "tokens need a numeric persona count: {s}"
    );
}

#[test]
fn should_run_counts_the_stored_panel_and_tokens() {
    let ws = Ws::new();
    let ids: Vec<String> = (1..=7).map(|i| format!("p{i}")).collect();
    ws.store.upsert_panel("fixture", "7 people").unwrap();
    ws.store.set_panel_resolution("fixture", &ids, "h").unwrap();
    let r = should_run(ws.root(), &["--paths", "site/index.html"]);
    let s = &r["data"]["spend"][0];
    assert_eq!(s["personas"], 7, "{r}");
    assert_eq!(s["lenses"], 4);
    assert_eq!(s["briefs"], 28);
    assert_eq!(s["tokens"], 28 * 15000);
}

#[test]
fn should_run_spend_from_rule_and_diff_file() {
    let d = tempfile::tempdir().unwrap();
    let diff = d.path().join("c.diff");
    std::fs::write(&diff, "diff --git a/copy/hero.md b/copy/hero.md\n--- a/copy/hero.md\n+++ b/copy/hero.md\n@@ -1 +1 @@\n-a\n+b\n").unwrap();
    let r = should_run(d.path(), &["--diff", diff.to_str().unwrap()]);
    assert_eq!(r["data"]["quorum"], "low", "{r}");
    let s = &r["data"]["spend"][0];
    assert_eq!(
        (
            s["class"].as_str(),
            s["personas"].as_i64(),
            s["lenses"].as_i64()
        ),
        (Some("copy"), Some(1), Some(1))
    );
    assert_eq!(s["briefs"], 1);
    assert_eq!(s["tokens"], 15000);
}

#[test]
fn should_run_spend_carries_the_rule_detail() {
    let d = tempfile::tempdir().unwrap();
    let cfg = d.path().join("jurygraph.toml");
    std::fs::write(
        &cfg,
        "[[rule]]\nclass = \"public\"\npaths = [\"site/**\"]\nquorum = \"high\"\nlenses = [\"critical\"]\nspend = 3\ndetail = \"full\"\n",
    )
    .unwrap();
    let r = cli_in(d.path(), &["should-run", "--paths", "site/a.html"]);
    assert_eq!(r["data"]["spend"][0]["detail"], "full", "{r}");
}

#[test]
fn should_run_human_quorum_has_no_persona_spend() {
    let d = tempfile::tempdir().unwrap();
    let r = should_run(d.path(), &["--paths", "deploy/prod.yml"]);
    assert_eq!(r["data"]["quorum"], "human");
    assert_eq!(r["data"]["spend"], json!([]));
}

#[test]
fn should_run_needs_paths_and_a_config() {
    let d = tempfile::tempdir().unwrap();
    let cfg = fixtures().join("jurygraph.toml");
    let r = cli_in(d.path(), &["should-run", "--config", cfg.to_str().unwrap()]);
    assert_eq!(r["error"]["code"], "invalid_arguments");
    let r = cli_in(d.path(), &["should-run", "--paths", "a"]);
    assert_eq!(
        r["error"]["code"], "not_found",
        "no <root>/jurygraph.toml: {r}"
    );
    std::fs::copy(&cfg, d.path().join("jurygraph.toml")).unwrap();
    let r = cli_in(d.path(), &["should-run", "--paths", "site/a"]);
    assert_eq!(
        r["data"]["quorum"], "high",
        "default config is <root>/jurygraph.toml: {r}"
    );
}
