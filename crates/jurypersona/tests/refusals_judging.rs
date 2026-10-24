//! The integration refusals of the plan's table that Task 5 owns (R1-R9, R11-R13): each asserts `error.code`.
#[path = "judging_support/mod.rs"]
mod support;
use jurypersona::domain::Stake;
use serde_json::{json, Value};
use support::*;

fn verdict(ws: &Ws, run: i64, v: &Value) -> Value {
    ws.cli(&[
        "verdict",
        "--run",
        &run.to_string(),
        "--json",
        &v.to_string(),
    ])
}
fn setup(lens: &str, rival: bool) -> (Ws, i64) {
    let ws = Ws::new();
    ws.site();
    if rival {
        ws.rival();
    }
    let run = ws.run(lens, &TWO, rival);
    (ws, run)
}
fn code(r: &Value) -> &str {
    assert_eq!(r["ok"], false, "{r}");
    r["error"]["code"].as_str().unwrap()
}

#[test]
fn verdict_citation_into_a_text_file_checks_the_line() {
    let ws = Ws::new();
    ws.subject(
        "doc",
        jurypersona::domain::SubjectKind::Document,
        false,
        &[
            ("page.json", br#"{"nodes":[{"id":"n1"}]}"#),
            ("readme.md", b"one\ntwo\nthree\n"),
        ],
    );
    let run = ws.run_of("doc", "critical", &TWO, false);
    for c in ["readme.md#L9", "readme.md#L0"] {
        let r = verdict(&ws, run, &critical("ana", [3, 3, 3], &[], &[c]));
        assert_eq!(code(&r), "citation_unresolved", "{c}");
    }
    let ok = verdict(&ws, run, &critical("ana", [3, 3, 3], &[], &["readme.md#L3"]));
    assert_eq!(ok["ok"], true, "{ok}");
}

#[test]
fn i6_only_line_locators_of_a_text_file_are_checked_and_nul_bytes_are_not_text() {
    let ws = Ws::new();
    ws.subject(
        "doc",
        jurypersona::domain::SubjectKind::Document,
        false,
        &[
            ("page.json", br#"{"nodes":[{"id":"n1"}]}"#),
            ("notes.md", b"a\r\nb\r\nc\r\n"),
            ("mod.wasm", b"\0asm\x01\0\0\0\0\0"),
        ],
    );
    let run = ws.run_of("doc", "critical", &TWO, false);
    let try_cite = |c: &str| verdict(&ws, run, &critical("ana", [3, 3, 3], &[], &[c]));
    for c in ["notes.md#intro", "notes.md#Latest", "notes.md#L3", "mod.wasm#L99", "mod.wasm#intro"] {
        let r = try_cite(c);
        assert_eq!(r["ok"], true, "{c}: {r}");
    }
    for c in ["notes.md#L4", "notes.md#L0", "notes.md#L+1", "notes.md#L01", "notes.md#L 1", "notes.md#L-1"] {
        assert_eq!(code(&try_cite(c)), "citation_unresolved", "{c}");
    }
}

#[test]
fn verdict_seen_without_citation() {
    let (ws, run) = setup("critical", false);
    let mut v = critical("ana", [3, 3, 3], &[], &[]);
    v["lines"][0]["citations"] = json!([]);
    assert_eq!(code(&verdict(&ws, run, &v)), "seen_without_citation");
    // nothing was stored
    assert_eq!(
        ws.cli(&["matrix", &run.to_string()])["data"]["strata"],
        json!([])
    );
}

#[test]
fn verdict_citation_unresolved() {
    let (ws, run) = setup("critical", false);
    for c in [
        "nowhere.png",
        "page.json#ghost",
        "change.diff#L9",
        "cmp:rival/rival.json",
    ] {
        let r = verdict(&ws, run, &critical("ana", [3, 3, 3], &[], &[c]));
        assert_eq!(code(&r), "citation_unresolved", "{c}");
    }
    let ok = verdict(
        &ws,
        run,
        &critical(
            "ana",
            [3, 3, 3],
            &[],
            &["change.diff#L5", "page.json#n2", "shot.png"],
        ),
    );
    assert_eq!(
        ok["ok"], true,
        "children's ids and diff lines resolve: {ok}"
    );
}

#[test]
fn verdict_compare_without_snapshot() {
    let (ws, run) = setup("compare", false);
    let mut v = compare("ana", "page.json", "r");
    for l in v["lines"].as_array_mut().unwrap() {
        l["evidence"] = json!("told");
        l["citations"] = json!([]);
    }
    v["lines"][0]["about"] = json!("cmp:rival");
    assert_eq!(code(&verdict(&ws, run, &v)), "comparator_not_snapshotted");
    v["lines"][0]["evidence"] = json!("recalled");
    v["lines"][0]["citations"] = json!([]);
    assert_eq!(
        verdict(&ws, run, &v)["ok"],
        true,
        "a recalled line may name any comparator"
    );
}

#[test]
fn verdict_unknown_about() {
    let (ws, run) = setup("compare", true);
    for about in ["nowhere.png", "rival", "cmp:"] {
        let mut v = compare("ana", "page.json", "r");
        v["lines"][0]["about"] = json!(about);
        let r = verdict(&ws, run, &v);
        assert_eq!(code(&r), "unknown_about", "{about}");
        assert!(
            r["error"]["message"]
                .as_str()
                .unwrap()
                .starts_with("line 1: "),
            "{r}"
        );
    }
    let mut v = compare("ana", "page.json", "r");
    v["lines"][0]["about"] = json!("shot.png");
    assert_eq!(
        verdict(&ws, run, &v)["ok"],
        true,
        "an attachment label is an option"
    );
}

#[test]
fn verdict_one_line_per_dimension_and_about() {
    let (ws, run) = setup("compare", true);
    let mut v = compare("ana", "page.json", "r");
    let extra = json!({"dimension": "clarity", "score": 4, "anchor_met": 3, "evidence": "seen",
                       "about": "shot.png", "citations": ["shot.png", "cmp:rival/rival.json#r1"]});
    v["lines"].as_array_mut().unwrap().push(extra.clone());
    assert_eq!(verdict(&ws, run, &v)["ok"], true);
    v["lines"].as_array_mut().unwrap().push(extra);
    let r = verdict(&ws, run, &v);
    assert_eq!(code(&r), "duplicate_dimension");
    let msg = r["error"]["message"].as_str().unwrap();
    assert!(msg.contains("clarity") && msg.contains("shot.png"), "{msg}");
}

#[test]
fn verdict_seen_compare_without_comparator() {
    let (ws, run) = setup("compare", true);
    let mut v = compare("ana", "cmp:rival", "x");
    v["lines"][1]["citations"] = json!(["page.json#n1"]);
    let r = verdict(&ws, run, &v);
    assert_eq!(code(&r), "seen_compare_without_comparator");
    assert!(
        r["error"]["message"]
            .as_str()
            .unwrap()
            .starts_with("line 2: "),
        "{r}"
    );
    // about the held comparator instead of citing it: passes
    v["lines"][1]["about"] = json!("cmp:rival");
    assert_eq!(verdict(&ws, run, &v)["ok"], true);
}

#[test]
fn verdict_bottom_score_without_blocker() {
    let (ws, run) = setup("critical", false);
    let r = verdict(
        &ws,
        run,
        &critical("ana", [1, 3, 3], &[], &["page.json#n1"]),
    );
    assert_eq!(code(&r), "bottom_score_without_blocker");
    assert_eq!(
        r["error"]["message"],
        "line 1: the bottom score needs a blocker that says what failed"
    );
    let ok = verdict(
        &ws,
        run,
        &critical(
            "ana",
            [1, 3, 3],
            &["the button does nothing"],
            &["page.json#n1"],
        ),
    );
    assert_eq!(ok["ok"], true, "{ok}");
}

#[test]
fn verdict_score_out_of_range() {
    let (ws, run) = setup("critical", false);
    assert_eq!(
        code(&verdict(
            &ws,
            run,
            &critical("ana", [6, 3, 3], &[], &["page.json#n1"])
        )),
        "score_out_of_range"
    );
    assert_eq!(
        code(&verdict(
            &ws,
            run,
            &critical("ana", [0, 3, 3], &[], &["page.json#n1"])
        )),
        "score_out_of_range"
    );
}

#[test]
fn verdict_unknown_dimension() {
    let (ws, run) = setup("critical", false);
    let mut v = critical("ana", [3, 3, 3], &[], &["page.json#n1"]);
    v["lines"][1]["dimension"] = json!("vibes");
    assert_eq!(code(&verdict(&ws, run, &v)), "unknown_dimension");
}

#[test]
fn matrix_across_strata_refused() {
    let (ws, run) = setup("critical", false);
    ws.ingest(run, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    let r = ws.cli(&["matrix", &run.to_string(), "--across-strata"]);
    assert_eq!(code(&r), "strata_average_refused");
    // refused first, even for a run that does not exist
    assert_eq!(
        code(&ws.cli(&["matrix", "404", "--across-strata"])),
        "strata_average_refused"
    );
}

#[test]
fn verdict_after_subject_moved() {
    let (ws, run) = setup("critical", false);
    let paths = ws.store.subject("site").unwrap().unwrap().attachments;
    let src = paths
        .iter()
        .find(|a| a.label == "change.diff")
        .unwrap()
        .source_path
        .clone();
    std::fs::write(&src, "different\n").unwrap();
    let r = verdict(
        &ws,
        run,
        &critical("ana", [3, 3, 3], &[], &["page.json#n1"]),
    );
    assert_eq!(code(&r), "subject_moved");
    assert!(
        r["error"]["message"]
            .as_str()
            .unwrap()
            .contains("change.diff changed"),
        "{r}"
    );
}

#[test]
fn deleted_source_counts_as_moved() {
    let (ws, run) = setup("critical", false);
    let a = ws.store.subject("site").unwrap().unwrap().attachments;
    let src = a
        .iter()
        .find(|a| a.label == "shot.png")
        .unwrap()
        .source_path
        .clone();
    std::fs::remove_file(&src).unwrap();
    let r = verdict(
        &ws,
        run,
        &critical("ana", [3, 3, 3], &[], &["page.json#n1"]),
    );
    assert_eq!(code(&r), "subject_moved");
    assert!(
        r["error"]["message"]
            .as_str()
            .unwrap()
            .contains(&format!("source_missing: {src}")),
        "{r}"
    );
}

#[test]
fn verdict_from_persona_not_in_run() {
    let ws = Ws::new();
    ws.site();
    // zed has a stake in the declared stakes but was not briefed
    let run = ws.run("critical", &TWO, false);
    let mut row = ws.store.run(run).unwrap().unwrap();
    row.id = 0;
    row.stakes.insert("zed".into(), Stake::Bystander);
    let run = ws.store.insert_run(&row).unwrap();
    let r = verdict(
        &ws,
        run,
        &critical("zed", [3, 3, 3], &[], &["page.json#n1"]),
    );
    assert_eq!(code(&r), "persona_without_stake");
    let r = verdict(
        &ws,
        run,
        &critical("nobody", [3, 3, 3], &[], &["page.json#n1"]),
    );
    assert_eq!(code(&r), "persona_without_stake");
}

#[test]
fn verdict_pick_without_reason() {
    let (ws, run) = setup("compare", true);
    assert_eq!(
        code(&verdict(&ws, run, &compare("ana", "page.json", "  "))),
        "pick_without_reason"
    );
    let mut v = compare("ana", "page.json", "x");
    v.as_object_mut().unwrap().remove("pick");
    assert_eq!(code(&verdict(&ws, run, &v)), "pick_missing");
    assert_eq!(
        code(&verdict(&ws, run, &compare("ana", "elsewhere", "x"))),
        "pick_unknown_option"
    );
    assert_eq!(
        verdict(&ws, run, &compare("ana", "cmp:rival", "x"))["ok"],
        true
    );
}

#[test]
fn verdict_evidence_not_allowed() {
    let (ws, run) = setup("recall", false);
    let line =
        |d: &str, ev: &str| json!({"dimension": d, "score": 3, "anchor_met": 3, "evidence": ev});
    let v = json!({"persona": "ana", "lines": [line("first-read", "told"), line("would-return", "recalled")]});
    assert_eq!(code(&verdict(&ws, run, &v)), "evidence_not_allowed");
    let v = json!({"persona": "ana", "lines": [line("first-read", "recalled"), line("would-return", "recalled")]});
    assert_eq!(verdict(&ws, run, &v)["ok"], true);
}

#[test]
fn should_run_refuses_human_under_persona() {
    let d = tempfile::tempdir().unwrap();
    let bad = fixtures().join("jurypersona-bad.toml");
    let r = cli_in(
        d.path(),
        &[
            "should-run",
            "--config",
            bad.to_str().unwrap(),
            "--paths",
            "x",
        ],
    );
    assert_eq!(code(&r), "human_class_under_persona_quorum");
    assert!(!d.path().join(".jurypersona").exists());
}

#[test]
fn dismissed_point_cannot_return_without_new_evidence() {
    let (ws, run) = setup("critical", false);
    let first = ws.ingest(
        run,
        &critical("ana", [3, 3, 3], &["price hidden"], &["page.json#n1"]),
    );
    let f = first["findings"][0].as_str().unwrap();
    // not dismissed yet: raising it again is fine
    assert_eq!(
        ws.cli(&[
            "verdict",
            "--run",
            &run.to_string(),
            "--json",
            &critical("ana", [3, 3, 3], &["price hidden"], &["page.json#n1"]).to_string()
        ])["ok"],
        true
    );
    assert_eq!(
        ws.cli(&["decide", f, "dismissed", "--why", "by design", "--by", "n"])["ok"],
        true
    );
    // the same evidence again, even worded differently: refused
    let r = verdict(
        &ws,
        run,
        &critical("ana", [3, 3, 3], &["cost not visible"], &["page.json#n1"]),
    );
    assert_eq!(code(&r), "dismissed_point_repeated");
    // a later run of the same subject is held to it too
    let run2 = ws.run("critical", &TWO, false);
    assert_eq!(
        code(&verdict(
            &ws,
            run2,
            &critical("ana", [3, 3, 3], &["price hidden"], &["page.json#n1"])
        )),
        "dismissed_point_repeated"
    );
}

#[test]
fn dismissed_point_returns_with_new_evidence_or_from_another_persona() {
    let (ws, run) = setup("critical", false);
    let first = ws.ingest(
        run,
        &critical("ana", [3, 3, 3], &["price hidden"], &["page.json#n1"]),
    );
    ws.cli(&[
        "decide",
        first["findings"][0].as_str().unwrap(),
        "dismissed",
        "--why",
        "by design",
        "--by",
        "n",
    ]);
    // new evidence: another node
    let r = verdict(
        &ws,
        run,
        &critical("ana", [3, 3, 3], &["price hidden"], &["page.json#n2"]),
    );
    assert_eq!(r["ok"], true, "{r}");
    // another persona with the old evidence
    let r = verdict(
        &ws,
        run,
        &critical("bob", [3, 3, 3], &["price hidden"], &["page.json#n1"]),
    );
    assert_eq!(r["ok"], true, "{r}");
    // the same persona with the old evidence but no blocker
    let r = verdict(
        &ws,
        run,
        &critical("ana", [3, 3, 3], &[], &["page.json#n1"]),
    );
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn uncited_dismissed_point_returns_only_as_new_point() {
    let (ws, run) = setup("critical", false);
    let first = ws.ingest(run, &told("ana", &["feels cold"]));
    ws.cli(&[
        "decide",
        first["findings"][0].as_str().unwrap(),
        "dismissed",
        "--why",
        "taste",
        "--by",
        "n",
    ]);
    // the same words again: refused
    assert_eq!(
        code(&verdict(&ws, run, &told("ana", &["feels cold"]))),
        "dismissed_point_repeated"
    );
    // a different uncited point is not silenced by the dismissed one
    assert_eq!(verdict(&ws, run, &told("ana", &["no search"]))["ok"], true);
}

#[test]
fn check_fails_on_unresolved_findings() {
    let (ws, run) = setup("critical", false);
    ws.ingest(
        run,
        &critical(
            "ana",
            [3, 3, 3],
            &["price hidden", "no search"],
            &["page.json#n1"],
        ),
    );
    let r = ws.cli(&["check"]);
    assert_eq!(code(&r), "check_failed");
    assert!(
        r["error"]["message"]
            .as_str()
            .unwrap()
            .starts_with("2 finding(s) have no decision"),
        "{r}"
    );
    let u = r["data"]["unresolved"].as_array().unwrap();
    assert_eq!(u.len(), 2);
    for k in ["finding", "run", "persona", "dimension", "text"] {
        assert!(u[0].get(k).is_some(), "{k} in {}", u[0]);
    }
    assert_eq!(u[0]["persona"], "ana");
    assert_eq!(u[0]["dimension"], "works");
    // a later batch that drops a blocker does not clear it (C3: findings of every batch); it still needs a decision
    ws.ingest(
        run,
        &critical("ana", [3, 3, 3], &["no search"], &["page.json#n1"]),
    );
    let r = ws.cli(&["check"]);
    assert_eq!(r["data"]["unresolved"].as_array().unwrap().len(), 2);
}
