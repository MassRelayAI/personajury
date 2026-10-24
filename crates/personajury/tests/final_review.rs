//! The final review of v0.1 (docs/reports/2026-10-23-v0-1-final-review.md): one test per Critical and
//! Important finding, built from the reviewer's reproductions, and one per testable Minor. Each was seen red first.
#[path = "judging_support/mod.rs"]
mod support;
use personajury::domain::{Stake, SubjectKind};
use serde_json::{json, Value};
use std::path::Path;
use support::*;

const AC: [(&str, Stake); 2] = [("ana", Stake::Primary), ("cy", Stake::Bystander)];

fn unresolved(r: &Value) -> Vec<Value> {
    r["data"]["unresolved"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}
fn code(r: &Value) -> &str {
    r["error"]["code"].as_str().unwrap_or("")
}

// ---- Critical ----

#[test]
fn c1_a_fixed_point_raised_again_by_a_later_run_is_unresolved() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    let v = critical(
        "ana",
        [3, 3, 3],
        &["the Go button does nothing"],
        &["page.json#n1"],
    );
    let f = ws.ingest(r1, &v)["findings"][0]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        ws.cli(&["decide", &f, "fixed", "--by", "agent"])["ok"],
        true
    );
    assert_eq!(ws.cli(&["check"])["ok"], true);
    // nothing changed; a run after the decision raises the same point with the same citation
    let r2 = ws.run("critical", &AC, false);
    ws.ingest(r2, &v);
    let c = ws.cli(&["check"]);
    assert_eq!(c["ok"], false, "{c}");
    assert_eq!(code(&c), "check_failed");
    let u = unresolved(&c);
    assert_eq!(u.len(), 1, "{c}");
    assert_eq!(u[0]["finding"], f.as_str());
    assert_eq!(u[0]["reason"], "fix_not_landed");
    assert_eq!(u[0]["run"], r2);
    // a fresh `fixed` after that run resolves it again, and it waits for a later judged run
    assert_eq!(
        ws.cli(&["decide", &f, "fixed", "--by", "agent"])["ok"],
        true
    );
    let c = ws.cli(&["check"]);
    assert_eq!(c["ok"], true, "{c}");
    assert_eq!(c["data"]["fixed_unverified"].as_array().unwrap().len(), 1);
}

#[test]
fn c1_an_uncited_fixed_point_raised_on_a_new_version_is_unresolved() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    let f = ws.ingest(r1, &told("ana", &["no price anywhere"]))["findings"][0]
        .as_str()
        .unwrap()
        .to_string();
    ws.cli(&["decide", &f, "fixed", "--by", "agent"]);
    ws.subject(
        "site",
        SubjectKind::Page,
        false,
        &[("page.json", br#"{"nodes":[{"id":"n1"},{"id":"n9"}]}"#)],
    );
    let r2 = ws.run("critical", &AC, false);
    ws.ingest(r2, &told("ana", &["no price anywhere"]));
    let u = unresolved(&ws.cli(&["check"]));
    assert_eq!(u.len(), 1);
    assert_eq!(u[0]["reason"], "fix_not_landed");
}

#[test]
fn c2_delta_with_missing_cells_is_not_landed() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    ws.ingest(
        r1,
        &critical(
            "ana",
            [3, 3, 3],
            &["the Go button does nothing"],
            &["page.json#n1"],
        ),
    );
    ws.ingest(r1, &critical("cy", [3, 3, 3], &[], &["page.json#n1"]));
    // a run with no verdict at all
    let r2 = ws.run("critical", &AC, false);
    let d = ws.cli(&["delta", &r1.to_string(), &r2.to_string()]);
    assert_eq!(d["ok"], true, "{d}");
    assert_eq!(d["data"]["landed"], false, "{d}");
    assert_eq!(d["data"]["incomplete"].as_array().unwrap().len(), 6);
    // one of two personas answered
    let r3 = ws.run("critical", &AC, false);
    ws.ingest(r3, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    let d = ws.cli(&["delta", &r1.to_string(), &r3.to_string()]);
    assert_eq!(d["data"]["landed"], false, "{d}");
    let inc = d["data"]["incomplete"].as_array().unwrap();
    assert_eq!(inc.len(), 3);
    assert!(inc.iter().all(|k| k["persona"] == "cy"), "{d}");
    // both answered: it lands and nothing is incomplete
    ws.ingest(r3, &critical("cy", [3, 3, 3], &[], &["page.json#n1"]));
    let d = ws.cli(&["delta", &r1.to_string(), &r3.to_string()]);
    assert_eq!(d["data"]["landed"], true, "{d}");
    assert_eq!(d["data"]["incomplete"], json!([]));
}

#[test]
fn c3_reingesting_without_the_blocker_keeps_the_finding_unresolved() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    ws.ingest(
        r1,
        &critical(
            "ana",
            [2, 3, 3],
            &["the Go button does nothing"],
            &["page.json#n1"],
        ),
    );
    assert_eq!(unresolved(&ws.cli(&["check"])).len(), 1);
    let d = ws.ingest(r1, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    assert_eq!(d["batch"], 2);
    let c = ws.cli(&["check"]);
    assert_eq!(
        c["ok"], false,
        "a retracted point still needs a decision: {c}"
    );
    assert_eq!(unresolved(&c).len(), 1);
}

// ---- Important ----

#[test]
fn i1_delta_counts_blockers_per_stratum() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    ws.ingest(r1, &critical("ana", [3, 3, 3], &["a1"], &["page.json#n1"]));
    ws.ingest(
        r1,
        &critical("cy", [3, 3, 3], &["c1", "c2", "c3"], &["page.json#n1"]),
    );
    let r2 = ws.run("critical", &AC, false);
    ws.ingest(
        r2,
        &critical("ana", [3, 3, 3], &["a1", "a2", "a3"], &["page.json#n1"]),
    );
    ws.ingest(r2, &critical("cy", [3, 3, 3], &[], &["page.json#n1"]));
    let d = ws.cli(&["delta", &r1.to_string(), &r2.to_string()]);
    assert_eq!(d["ok"], true, "{d}");
    assert_eq!(d["data"]["landed"], false, "primary blockers rose: {d}");
    let by = &d["data"]["blockers_by_stratum"];
    assert_eq!(by["primary"], json!({"before": 1, "after": 3}), "{d}");
    assert_eq!(by["bystander"], json!({"before": 3, "after": 0}), "{d}");
}

#[test]
fn i2_a_new_subject_version_makes_old_runs_stale() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    // v2: the same files plus a new one
    ws.subject(
        "site",
        SubjectKind::Page,
        false,
        &[
            (
                "page.json",
                br#"{"nodes":[{"id":"n1","children":[{"id":"n2"}]},{"id":"n3"}]}"#,
            ),
            ("shot.png", b"\x89PNG not really"),
            ("change.diff", b"a\nb\nc\nd\ne\n"),
            ("pricing.json", br#"{"nodes":[{"id":"p1"}]}"#),
        ],
    );
    let r = ws.cli(&[
        "verdict",
        "--run",
        &r1.to_string(),
        "--json",
        &critical("ana", [3, 3, 3], &[], &["page.json#n1"]).to_string(),
    ]);
    assert_eq!(code(&r), "subject_moved", "{r}");
    assert!(
        r["error"]["message"]
            .as_str()
            .unwrap()
            .contains("new subject version 2"),
        "{r}"
    );
    let c = ws.cli(&["check"]);
    let stale = c["data"]["stale"].as_array().unwrap();
    assert_eq!(stale.len(), 1, "{c}");
    assert_eq!(stale[0]["run"], r1);
    assert_eq!(stale[0]["why"], "new subject version 2");
}

#[test]
fn i3_matrix_refuses_mixed_subjects_and_lens_versions() {
    let ws = Ws::new();
    ws.site();
    ws.subject(
        "other",
        SubjectKind::Page,
        false,
        &[("page.json", br#"{"nodes":[{"id":"n1"}]}"#)],
    );
    let r1 = ws.run("critical", &AC, false);
    ws.ingest(r1, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    let r2 = ws.run_of("other", "critical", &AC, false);
    ws.ingest(r2, &critical("ana", [5, 5, 5], &[], &["page.json#n1"]));
    let m = ws.cli(&["matrix", &r1.to_string(), &r2.to_string(), "--table"]);
    assert_eq!(code(&m), "invalid_arguments", "{m}");
    // two versions of one lens name
    let toml = std::fs::read_to_string(fixtures().join("../lenses/critical.toml"))
        .unwrap()
        .replace("standard = \"\"", "standard = \"house style\"");
    let p = ws.root().join("critical2.toml");
    std::fs::write(&p, toml).unwrap();
    assert_eq!(
        ws.cli(&["lens", "add", "--file", p.to_str().unwrap()])["data"]["version"],
        2
    );
    let r3 = ws.run("critical", &AC, false);
    ws.ingest(r3, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    let m = ws.cli(&["matrix", &r1.to_string(), &r3.to_string()]);
    assert_eq!(code(&m), "invalid_arguments", "{m}");
}

#[test]
fn i4_matrix_cells_carry_the_evidence_class() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    ws.ingest(r1, &told("ana", &[]));
    let lines: Vec<Value> = ["works", "blocks", "missing"]
        .iter()
        .map(|d| json!({"dimension": d, "score": 5, "anchor_met": 5, "evidence": "recalled"}))
        .collect();
    let recalled = json!({"persona": "cy", "lines": lines});
    ws.ingest(r1, &recalled);
    let m = ws.cli(&["matrix", &r1.to_string(), "--table", "--detail", "full"]);
    assert_eq!(m["ok"], true, "{m}");
    let primary = &m["data"]["strata"][0]["cells"][0];
    assert_eq!(
        primary["evidence"],
        json!({"seen": 0, "told": 1, "recalled": 0}),
        "{m}"
    );
    let bystander = &m["data"]["strata"][1]["cells"][0];
    assert_eq!(
        bystander["evidence"],
        json!({"seen": 0, "told": 0, "recalled": 1}),
        "{m}"
    );
    let t = m["data"]["table"].as_str().unwrap();
    assert!(t.contains("| primary | ana | 2t | 2t | 2t |\n"), "{t}");
    assert!(t.contains("| bystander | cy | 5r | 5r | 5r |\n"), "{t}");
    assert!(
        t.contains("| bystander | median | 5 | 5 | 5 |\n"),
        "a median has no suffix: {t}"
    );
    assert!(
        t.ends_with("\nscores: r = recalled, t = told, unmarked = seen\n"),
        "{t}"
    );
}

#[test]
fn i5_fixed_unverified_needs_a_judged_run_with_the_same_lens() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    let f = ws.ingest(
        r1,
        &critical("ana", [3, 3, 3], &["slow"], &["page.json#n1"]),
    )["findings"][0]
        .as_str()
        .unwrap()
        .to_string();
    ws.cli(&["decide", &f, "fixed", "--by", "agent"]);
    let fu = |ws: &Ws| {
        ws.cli(&["check"])["data"]["fixed_unverified"]
            .as_array()
            .unwrap()
            .len()
    };
    assert_eq!(fu(&ws), 1);
    // a recall brief on the same subject, never judged
    ws.run("recall", &AC, false);
    assert_eq!(
        fu(&ws),
        1,
        "an unjudged run of another lens verifies nothing"
    );
    // a critical run where only cy answered
    let r3 = ws.run("critical", &AC, false);
    ws.ingest(r3, &critical("cy", [3, 3, 3], &[], &["page.json#n1"]));
    assert_eq!(
        fu(&ws),
        1,
        "the persona that raised it has not looked again"
    );
    // ana judges a later critical run and does not raise it
    ws.ingest(r3, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    assert_eq!(fu(&ws), 0);
}

fn persona_json(ws: &Ws, id: &str) -> String {
    let p = ws.root().join(format!("p-{}.json", id.replace('/', "_")));
    std::fs::write(
        &p,
        json!({"id": id, "name": id, "tags": ["t"], "role": "r", "context": "c", "daily_tasks": "d", "fears": "f",
               "never_reads": "n", "decides_by": "b", "stake_rules": "s"})
        .to_string(),
    )
    .unwrap();
    p.to_string_lossy().into_owned()
}

#[test]
fn i6_ids_and_names_must_be_slugs() {
    let ws = Ws::new();
    for bad in ["../../../../ESCAPED", "Ana", "a/b", "a..b", "-a", ""] {
        let p = persona_json(&ws, bad);
        let r = ws.cli(&["persona", "add", "--file", &p]);
        assert_eq!(code(&r), "invalid_arguments", "persona id {bad:?}: {r}");
    }
    assert_eq!(
        ws.cli(&[
            "persona",
            "add",
            "--file",
            &persona_json(&ws, "builder-ana.2")
        ])["ok"],
        true
    );
    let r = ws.cli(&["panel", "add", "a/b", "--query", "1 t"]);
    assert_eq!(code(&r), "invalid_arguments", "{r}");
    let f = ws.root().join("w.json");
    std::fs::write(&f, r#"{"nodes":[]}"#).unwrap();
    for verb in ["subject", "comparator"] {
        let r = ws.cli(&[
            verb,
            "add",
            "../x",
            "--kind",
            "page",
            "--attach",
            f.to_str().unwrap(),
        ]);
        assert_eq!(code(&r), "invalid_arguments", "{verb}: {r}");
    }
    let toml = std::fs::read_to_string(fixtures().join("../lenses/critical.toml"))
        .unwrap()
        .replace("name = \"critical\"", "name = \"../crit\"");
    let p = ws.root().join("bad-lens.toml");
    std::fs::write(&p, toml).unwrap();
    assert_eq!(
        code(&ws.cli(&["lens", "add", "--file", p.to_str().unwrap()])),
        "invalid_arguments"
    );
}

#[test]
fn i6_brief_checks_the_id_before_the_path_and_leaves_no_run() {
    let ws = Ws::new();
    // a persona stored with an id that escapes, as an older binary or a hand edit could leave it
    let evil = personajury::domain::Persona {
        id: "../../ESCAPED".into(),
        version: 0,
        name: "e".into(),
        tags: vec!["t".into()],
        stake_rules: "s".into(),
        profile_json: "{}".into(),
        origin: personajury::domain::Origin::Hand,
        hash: "h".into(),
        traits: None,
        serves: vec![],
    };
    ws.store.insert_persona(&evil).unwrap();
    ws.site();
    assert_eq!(ws.cli(&["panel", "add", "p", "--query", "1 t"])["ok"], true);
    let r = ws.cli(&[
        "brief",
        "--subject",
        "site",
        "--panel",
        "p",
        "--lens",
        "critical",
        "--stake",
        "../../ESCAPED=primary",
        "--model",
        "m",
    ]);
    assert_eq!(code(&r), "invalid_arguments", "{r}");
    assert!(!ws.root().join("../ESCAPED--critical.md").exists());
    assert_eq!(ws.cli(&["check"])["data"]["runs"], 0, "no orphan run");
}

fn should_run(ws: &Ws, cfg: &str, args: &[&str]) -> Value {
    let p = ws.root().join("personajury.toml");
    std::fs::write(&p, cfg).unwrap();
    let mut a = vec!["should-run", "--config", p.to_str().unwrap()];
    a.extend(args);
    ws.cli(&a)
}

#[test]
fn i7_human_classes_are_matched_after_normalising_the_spelling() {
    let ws = Ws::new();
    for class in [
        "license",
        "Money",
        " public_claim ",
        "Public Claim",
        "LICENCE",
    ] {
        let cfg = format!("[[rule]]\nclass = \"{class}\"\npaths = [\"x/**\"]\nquorum = \"low\"\n");
        let r = should_run(&ws, &cfg, &["--paths", "x/a"]);
        assert_eq!(
            code(&r),
            "human_class_under_persona_quorum",
            "{class:?}: {r}"
        );
    }
}

#[test]
fn i8_quoted_paths_in_a_diff_are_read() {
    let ws = Ws::new();
    let cfg = "[[rule]]\nclass = \"web\"\npaths = [\"apps/web/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"recall\"]\n\
[[rule]]\nclass = \"brand\"\npaths = [\"brand/**\"]\nquorum = \"human\"\n";
    let diff = "diff --git \"a/brand/logo-\\303\\251.svg\" \"b/brand/logo-\\303\\251.svg\"\nindex 1..2 100644\n\
--- \"a/brand/logo-\\303\\251.svg\"\n+++ \"b/brand/logo-\\303\\251.svg\"\n@@ -1 +1 @@\n-a\n+b\n\
diff --git a/apps/web/a.ts b/apps/web/a.ts\n--- a/apps/web/a.ts\n+++ b/apps/web/a.ts\n@@ -1 +1 @@\n-a\n+b\n";
    let d = ws.root().join("q.diff");
    std::fs::write(&d, diff).unwrap();
    let r = should_run(&ws, cfg, &["--diff", d.to_str().unwrap()]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"]["quorum"], "human", "{r}");
    assert_eq!(
        r["data"]["hits"][1]["paths"],
        json!(["brand/logo-é.svg"]),
        "{r}"
    );
    // a header the parser cannot read is refused, never dropped
    std::fs::write(&d, "diff --git what is this\n").unwrap();
    let r = should_run(&ws, cfg, &["--diff", d.to_str().unwrap()]);
    assert_eq!(code(&r), "invalid_diff", "{r}");
}

#[test]
fn i9_about_separates_findings_and_dismissals() {
    let ws = Ws::new();
    ws.site();
    ws.rival();
    let r1 = ws.run("compare", &AC, true);
    let cite = ["cmp:rival/rival.json#r1"];
    let line = |dim: &str, about: Option<&str>, blockers: &[&str]| {
        let mut l = json!({"dimension": dim, "score": 2, "anchor_met": 1, "evidence": "seen", "citations": cite,
                           "blockers": blockers});
        if let Some(a) = about {
            l["about"] = json!(a);
        }
        l
    };
    let v = json!({"persona": "ana", "lines": [
        line("clarity", Some("shot.png"), &["the price is hidden"]),
        line("clarity", Some("cmp:rival"), &["the price is hidden"]),
        line("price", None, &[]), line("trust", None, &[])],
        "pick": {"choice": "shot.png", "reason": "r"}});
    let d = ws.ingest(r1, &v);
    let ids = d["findings"].as_array().unwrap();
    assert_eq!(ids.len(), 2, "one finding per option: {d}");
    let c = ws.cli(&["check"]);
    let u = unresolved(&c);
    assert_eq!(u.len(), 2);
    let on_shot = u
        .iter()
        .find(|x| x["about"] == "shot.png")
        .expect("check lists about");
    let id = on_shot["finding"].as_str().unwrap();
    assert_eq!(
        ws.cli(&["decide", id, "dismissed", "--why", "intended", "--by", "n"])["ok"],
        true
    );
    // the same point on the other option is not the dismissed one
    let r2 = ws.run("compare", &AC, true);
    let v2 = json!({"persona": "ana", "lines": [
        line("clarity", Some("cmp:rival"), &["the price is hidden"]),
        line("price", None, &[]), line("trust", None, &[])],
        "pick": {"choice": "shot.png", "reason": "r"}});
    let r = ws.cli(&[
        "verdict",
        "--run",
        &r2.to_string(),
        "--json",
        &v2.to_string(),
    ]);
    assert_eq!(r["ok"], true, "{r}");
    // the dismissed option itself is still refused
    let v3 = json!({"persona": "ana", "lines": [
        line("clarity", Some("shot.png"), &["the price is hidden"]),
        line("price", None, &[]), line("trust", None, &[])],
        "pick": {"choice": "shot.png", "reason": "r"}});
    let r = ws.cli(&[
        "verdict",
        "--run",
        &r2.to_string(),
        "--json",
        &v3.to_string(),
    ]);
    assert_eq!(code(&r), "dismissed_point_repeated", "{r}");
}

// ---- release-blocking deferrals ----

/// A workspace built through the CLI only: personas ana and cy tagged t, panel p = "2 t", subject site (walk.json).
fn cli_ws() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let w = dir.path().to_path_buf();
    for id in ["ana", "cy"] {
        let p = w.join(format!("{id}.json"));
        std::fs::write(
            &p,
            json!({"id": id, "name": id, "tags": ["t"], "role": "r", "context": "c", "daily_tasks": "d",
                   "fears": "f", "never_reads": "n", "decides_by": "b", "stake_rules": "s"})
            .to_string(),
        )
        .unwrap();
        assert_eq!(
            cli_in(&w, &["persona", "add", "--file", p.to_str().unwrap()])["ok"],
            true
        );
    }
    assert_eq!(
        cli_in(&w, &["panel", "add", "p", "--query", "2 t"])["ok"],
        true
    );
    std::fs::create_dir_all(w.join("src")).unwrap();
    let walk = w.join("src/walk.json");
    std::fs::write(
        &walk,
        r#"{"nodes":[{"id":"n1","role":"button","name":"Go"}]}"#,
    )
    .unwrap();
    let r = cli_in(
        &w,
        &[
            "subject",
            "add",
            "site",
            "--kind",
            "page",
            "--attach",
            walk.to_str().unwrap(),
        ],
    );
    assert_eq!(r["ok"], true, "{r}");
    (dir, w)
}
fn brief(w: &Path, extra: &[&str]) -> Value {
    let mut a = vec![
        "brief",
        "--subject",
        "site",
        "--panel",
        "p",
        "--lens",
        "critical",
        "--stake",
        "ana=primary,cy=bystander",
        "--model",
        "m",
    ];
    a.extend(extra);
    cli_in(w, &a)
}

#[test]
fn brief_write_failure_leaves_no_orphan_run() {
    let (_d, w) = cli_ws();
    std::fs::write(
        w.join(".personajury/runs"),
        "a file where the runs folder goes",
    )
    .unwrap();
    let r = brief(&w, &[]);
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(cli_in(&w, &["check"])["data"]["runs"], 0, "no orphan run");
    std::fs::remove_file(w.join(".personajury/runs")).unwrap();
    let r = brief(&w, &[]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"]["run"], 1);
}

#[test]
fn lens_list_shows_compares() {
    let (_d, w) = cli_ws();
    let l = cli_in(&w, &["lens", "list"]);
    let items = l["data"]["items"].as_array().unwrap();
    let cmp = items.iter().find(|i| i["name"] == "compare").unwrap();
    assert_eq!(cmp["compares"], true, "{l}");
    let crit = items.iter().find(|i| i["name"] == "critical").unwrap();
    assert_eq!(crit["compares"], false, "{l}");
}

// ---- Minor ----

#[test]
fn minor_blank_blocker_counts_as_none() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    let v = critical("ana", [1, 3, 3], &["  "], &["page.json#n1"]);
    let r = ws.cli(&[
        "verdict",
        "--run",
        &r1.to_string(),
        "--json",
        &v.to_string(),
    ]);
    assert_eq!(code(&r), "bottom_score_without_blocker", "{r}");
    let d = ws.ingest(
        r1,
        &critical("ana", [2, 3, 3], &["", " \t"], &["page.json#n1"]),
    );
    assert_eq!(d["findings"], json!([]));
    assert_eq!(ws.cli(&["check"])["ok"], true);
}

#[test]
fn minor_pick_under_a_lens_that_asks_none_is_refused() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &AC, false);
    let mut v = critical("ana", [3, 3, 3], &[], &["page.json#n1"]);
    v["pick"] = json!({"choice": "not-an-option", "reason": "r"});
    let r = ws.cli(&[
        "verdict",
        "--run",
        &r1.to_string(),
        "--json",
        &v.to_string(),
    ]);
    assert_eq!(code(&r), "pick_not_asked", "{r}");
}

const LENS: &str = "name = \"mini\"\nversion = 1\nscale = [1, 5]\nevidence = [\"told\"]\n\n\
[[dimension]]\nid = \"a\"\ntitle = \"A\"\nanchors = { \"1\" = \"bad\", \"3\" = \"ok\", \"5\" = \"good\" }\n\n\
[[dimension]]\nid = \"b\"\ntitle = \"B\"\nanchors = { \"1\" = \"bad\", \"2\" = \"meh\", \"5\" = \"good\" }\n";

#[test]
fn minor_lens_add_validates_the_lens() {
    let ws = Ws::new();
    let add = |text: &str| {
        let p = ws.root().join("l.toml");
        std::fs::write(&p, text).unwrap();
        ws.cli(&["lens", "add", "--file", p.to_str().unwrap()])
    };
    assert_eq!(add(LENS)["ok"], true);
    let no_dims =
        "name = \"x\"\nversion = 1\nscale = [1, 5]\nevidence = [\"told\"]\ndimension = []\n";
    for (why, text) in [
        ("no dimension", no_dims.to_string()),
        ("duplicate id", LENS.replace("id = \"b\"", "id = \"a\"")),
        (
            "non-integer anchor",
            LENS.replace("\"2\" = \"meh\"", "\"two\" = \"meh\""),
        ),
        (
            "anchor outside scale",
            LENS.replace("\"2\" = \"meh\"", "\"9\" = \"meh\""),
        ),
        (
            "scale reversed",
            LENS.replace("scale = [1, 5]", "scale = [5, 1]"),
        ),
        (
            "scale flat",
            LENS.replace("scale = [1, 5]", "scale = [3, 3]"),
        ),
        (
            "no evidence",
            LENS.replace("evidence = [\"told\"]", "evidence = []"),
        ),
    ] {
        assert_eq!(code(&add(&text)), "invalid_arguments", "{why}");
    }
}

#[test]
fn minor_anchor_met_is_checked_against_its_own_dimension() {
    let ws = Ws::new();
    ws.site();
    let p = ws.root().join("mini.toml");
    std::fs::write(&p, LENS).unwrap();
    assert_eq!(
        ws.cli(&["lens", "add", "--file", p.to_str().unwrap()])["ok"],
        true
    );
    let r1 = ws.run("mini", &AC, false);
    // 3 is an anchor of a, not of b
    let v = json!({"persona": "ana", "lines": [
        {"dimension": "a", "score": 3, "anchor_met": 3, "evidence": "told"},
        {"dimension": "b", "score": 3, "anchor_met": 3, "evidence": "told"}]});
    let r = ws.cli(&[
        "verdict",
        "--run",
        &r1.to_string(),
        "--json",
        &v.to_string(),
    ]);
    assert_eq!(code(&r), "bad_anchor", "{r}");
}

#[test]
fn minor_missing_input_files_are_not_found() {
    let (_d, w) = cli_ws();
    let gone = w.join("gone.json");
    let g = gone.to_str().unwrap();
    for args in [
        vec!["persona", "add", "--file", g],
        vec!["lens", "add", "--file", g],
        vec!["subject", "add", "s2", "--kind", "page", "--attach", g],
        vec!["verdict", "--run", "1", "--file", g],
    ] {
        let r = cli_in(&w, &args);
        assert_eq!(code(&r), "not_found", "{args:?}: {r}");
    }
}

#[test]
fn minor_attachment_labels_that_cannot_be_cited_are_refused() {
    let (_d, w) = cli_ws();
    for name in ["a#b.json", "cmp:x.png"] {
        let p = w.join(name);
        std::fs::write(&p, "x").unwrap();
        let r = cli_in(
            &w,
            &[
                "subject",
                "add",
                "s2",
                "--kind",
                "page",
                "--attach",
                p.to_str().unwrap(),
            ],
        );
        assert_eq!(code(&r), "invalid_arguments", "{name}: {r}");
    }
}

#[test]
fn minor_model_and_stake_are_checked() {
    let (_d, w) = cli_ws();
    let r = cli_in(
        &w,
        &[
            "brief",
            "--subject",
            "site",
            "--panel",
            "p",
            "--lens",
            "critical",
            "--stake",
            "ana=primary,cy=bystander",
            "--model",
            " ",
        ],
    );
    assert_eq!(code(&r), "invalid_arguments", "{r}");
    let r = cli_in(
        &w,
        &[
            "brief",
            "--subject",
            "site",
            "--panel",
            "p",
            "--lens",
            "critical",
            "--stake",
            "ana=primary,cy=bystander,ana=bystander",
            "--model",
            "m",
        ],
    );
    assert_eq!(code(&r), "invalid_arguments", "{r}");
}

#[test]
fn minor_brief_shows_what_the_query_wanted_and_got() {
    let (_d, w) = cli_ws();
    assert_eq!(
        cli_in(&w, &["panel", "add", "p", "--query", "5 t"])["ok"],
        true
    );
    let r = brief(&w, &[]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(
        r["data"]["panel"]["terms"],
        json!([{"tag": "t", "wanted": 5, "got": 2}]),
        "{r}"
    );
}

#[test]
fn minor_brief_refuses_a_subject_whose_sources_moved() {
    let (_d, w) = cli_ws();
    std::fs::write(w.join("src/walk.json"), r#"{"nodes":[]}"#).unwrap();
    let r = brief(&w, &[]);
    assert_eq!(code(&r), "subject_moved", "{r}");
    assert_eq!(cli_in(&w, &["check"])["data"]["runs"], 0);
}

#[test]
fn minor_verdict_column_is_named_about() {
    let ws = Ws::new();
    let cols: Vec<String> = ws
        .store
        .conn()
        .prepare("SELECT name FROM pragma_table_info('verdict')")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert!(cols.contains(&"about".to_string()), "{cols:?}");
    assert!(!cols.contains(&"comparator".to_string()), "{cols:?}");
}
