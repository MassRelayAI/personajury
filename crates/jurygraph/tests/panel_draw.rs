//! v0.2 panel-draw (D9, D10): a stake per panel term, the seeded draw, `brief --rule`, the seat ceilings and the agent's
//! overrides with a reason. Every refusal here was seen red first.
use jurygraph::store::Store;
use assert_cmd::Command;
use serde_json::{json, Value};
use std::path::Path;

fn jg(ws: &Path, args: &[&str]) -> Value {
    let out = Command::cargo_bin("jurygraph")
        .unwrap()
        .arg("--root")
        .arg(ws)
        .args(args)
        .output()
        .unwrap();
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{args:?}: not json ({e}): {} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}
fn ok(v: Value) -> Value {
    assert_eq!(v["ok"], true, "{v}");
    v["data"].clone()
}
fn code(v: &Value) -> &str {
    assert_eq!(v["ok"], false, "{v}");
    v["error"]["code"].as_str().unwrap()
}

/// `n` personas `<prefix>-00 …` all tagged `tag`.
fn pool(ws: &Path, prefix: &str, tag: &str, n: usize) {
    let dir = ws.join("pool");
    std::fs::create_dir_all(&dir).unwrap();
    for i in 0..n {
        let id = format!("{prefix}-{i:03}");
        let f = dir.join(format!("{id}.json"));
        let body = json!({
            "id": id, "name": format!("Person {id}"), "tags": [tag],
            "role": "r", "context": "c", "daily_tasks": "d", "fears": "f",
            "never_reads": "n", "decides_by": "b", "stake_rules": "s"
        });
        std::fs::write(&f, body.to_string()).unwrap();
        ok(jg(ws, &["persona", "add", "--file", f.to_str().unwrap()]));
    }
}
/// A one-file document subject.
fn doc(ws: &Path, name: &str) {
    let dir = ws.join("src");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(format!("{name}.md"));
    std::fs::write(&f, format!("# {name}\nline two\nline three\n")).unwrap();
    ok(jg(
        ws,
        &["subject", "add", name, "--kind", "document", "--attach", f.to_str().unwrap()],
    ));
}
fn ids(v: &Value) -> Vec<String> {
    v["personas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

// ---- panel add: the seat ceilings ----

#[test]
fn panel_above_the_ceiling_is_refused_unless_allowed() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    for q in ["13 others", "7 builder, 6 others"] {
        let r = jg(w, &["panel", "add", "big", "--query", q]);
        assert_eq!(code(&r), "panel_too_large", "{q}");
        assert_eq!(r["data"]["ceiling"], 12, "{r}");
    }
    let r = jg(w, &["panel", "add", "twelve", "--query", "12 others"]);
    assert_eq!(r["ok"], true, "{r}");
    assert!(r["data"]["large"].is_null(), "12 is not large: {r}");
    let r = jg(w, &["panel", "add", "big", "--query", "13 others", "--allow-large"]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"]["large"], true, "{r}");
    let r = jg(w, &["panel", "add", "max", "--query", "1000 others", "--allow-large"]);
    assert_eq!(r["ok"], true, "{r}");
    let r = jg(w, &["panel", "add", "over", "--query", "1001 others", "--allow-large"]);
    assert_eq!(code(&r), "panel_too_large");
    assert_eq!(r["data"]["ceiling"], 1000, "{r}");
}

// ---- panel resolve: the draw ----

#[test]
fn panel_resolve_says_when_there_was_no_draw_and_names_the_seed_with_a_subject() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, "rdr", "reader", 6);
    doc(w, "pricing");
    ok(jg(w, &["panel", "add", "by", "--query", "3 reader=bystander"]));
    let none = ok(jg(w, &["panel", "resolve", "by"]));
    assert_eq!(none["draw"], "none (no subject)");
    assert_eq!(ids(&none), vec!["rdr-000", "rdr-001", "rdr-002"], "id order");
    let drawn = ok(jg(w, &["panel", "resolve", "by", "--subject", "pricing"]));
    let seed = drawn["draw"]["seed"].as_str().unwrap();
    assert_eq!(seed.len(), 16, "{drawn}");
    assert_eq!(drawn["terms"][0]["stake"], "bystander", "{drawn}");
}

#[test]
fn the_draw_differs_between_subjects_and_repeats_between_rounds() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, "rdr", "reader", 6);
    doc(w, "pricing");
    doc(w, "changelog");
    ok(jg(w, &["panel", "add", "by", "--query", "3 reader=bystander"]));
    let a1 = ids(&ok(jg(w, &["panel", "resolve", "by", "--subject", "pricing"])));
    let a2 = ids(&ok(jg(w, &["panel", "resolve", "by", "--subject", "pricing"])));
    let b = ids(&ok(jg(w, &["panel", "resolve", "by", "--subject", "changelog"])));
    assert_eq!(a1, a2, "round 2 of the same subject draws the same seats");
    assert_ne!(a1, b, "another subject draws other seats");
    assert_eq!(a1.len(), 3);
}

// ---- brief: stakes from the panel, the draw, a rule, overrides, ceilings, the estimate ----

const TOML: &str = r#"
tokens_per_brief = 20000

[[rule]]
class = "public-surface"
paths = ["docs/**"]
quorum = "high"
panel = "docs-public"
lenses = ["critical", "recall"]
spend = 0
detail = "full"

[[rule]]
class = "copy"
paths = ["copy/**"]
quorum = "low"
panel = "docs-public"
lenses = ["recall"]
spend = 1

[[rule]]
class = "money"
paths = ["**/prices/**"]
quorum = "human"

[[rule]]
class = "internals"
paths = ["crates/**"]
quorum = "none"

[[rule]]
class = "ghost-panel"
paths = ["g/**"]
quorum = "low"
panel = "nope"
lenses = ["recall"]

[[rule]]
class = "ghost-lens"
paths = ["g/**"]
quorum = "low"
panel = "docs-public"
lenses = ["nolens"]
"#;

/// 3 builders, 1 newcomer, 2 approvers, 6 readers; the panel `docs-public` seats 8 of them; the subject `pricing`.
fn world(w: &Path) {
    pool(w, "bld", "builder", 3);
    pool(w, "new", "newcomer", 1);
    pool(w, "app", "approver", 2);
    pool(w, "rdr", "reader", 6);
    doc(w, "pricing");
    ok(jg(
        w,
        &[
            "panel", "add", "docs-public", "--query",
            "3 builder=primary, 1 newcomer=primary, 2 approver=secondary, 2 others=bystander",
        ],
    ));
    std::fs::write(w.join("jurygraph.toml"), TOML).unwrap();
}
fn brief(w: &Path, extra: &[&str]) -> Value {
    let mut a = vec!["brief", "--subject", "pricing", "--model", "m"];
    a.extend_from_slice(extra);
    jg(w, &a)
}
fn run_row(w: &Path, id: i64) -> jurygraph::domain::RunRow {
    Store::open_existing(w).unwrap().run(id).unwrap().unwrap()
}
/// Who sat and what they were handed, without the run folder the path names.
fn sig(briefs: &Value) -> Vec<(String, String)> {
    briefs
        .as_array()
        .unwrap()
        .iter()
        .map(|b| (b["persona"].to_string(), b["hash"].to_string()))
        .collect()
}
fn stake_counts(briefs: &Value) -> (usize, usize, usize) {
    let n = |s: &str| {
        briefs
            .as_array()
            .unwrap()
            .iter()
            .filter(|b| b["stake"] == s)
            .count()
    };
    (n("primary"), n("secondary"), n("bystander"))
}

#[test]
fn brief_takes_each_stake_from_the_panel_term_and_records_the_seed() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let d = ok(brief(w, &["--panel", "docs-public", "--lens", "critical"]));
    assert_eq!(stake_counts(&d["briefs"]), (4, 2, 2), "{d}");
    let seed = d["draw"]["seed"].as_str().unwrap().to_string();
    assert_eq!(
        seed,
        hex::encode(jurygraph::domain::hash::draw_seed("pricing", "docs-public"))
    );
    let row = run_row(w, d["run"].as_i64().unwrap());
    assert_eq!(row.draw_seed.as_deref(), Some(seed.as_str()));
    assert_eq!(row.override_json, None, "no rule, no override bookkeeping");
    // the same brief again draws the same readers
    let again = ok(brief(w, &["--panel", "docs-public", "--lens", "critical"]));
    assert_eq!(sig(&d["briefs"]), sig(&again["briefs"]));
}

#[test]
fn stake_flag_overrides_a_term_stake_and_the_answer_says_so() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let d = ok(brief(
        w,
        &["--panel", "docs-public", "--lens", "critical", "--stake", "bld-000=bystander"],
    ));
    assert_eq!(stake_counts(&d["briefs"]), (3, 2, 3), "{d}");
    assert_eq!(
        d["stake_overrides"],
        json!([{"persona": "bld-000", "panel": "primary", "given": "bystander"}])
    );
    let row = run_row(w, d["run"].as_i64().unwrap());
    assert_eq!(row.stakes["bld-000"], jurygraph::domain::Stake::Bystander);
    // a stake for someone outside the panel is still refused
    let r = brief(w, &["--panel", "docs-public", "--lens", "critical", "--stake", "rdr-005=primary"]);
    assert_eq!(code(&r), "invalid_arguments", "{r}");
}

#[test]
fn draw_flag_sets_and_records_the_seed() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let args = ["--panel", "docs-public", "--lens", "critical", "--draw"];
    let mut a = args.to_vec();
    a.push("00000000000000ff");
    let d = ok(brief(w, &a));
    assert_eq!(d["draw"]["seed"], "00000000000000ff");
    assert_eq!(
        run_row(w, d["run"].as_i64().unwrap()).draw_seed.as_deref(),
        Some("00000000000000ff")
    );
    let mut a = args.to_vec();
    a.push("xyz");
    assert_eq!(code(&brief(w, &a)), "invalid_arguments");
}

#[test]
fn a_term_that_resolves_to_nobody_is_refused() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    ok(jg(w, &["panel", "add", "ghosts", "--query", "2 approver=secondary, 1 ghost=primary"]));
    let r = brief(w, &["--panel", "ghosts", "--lens", "critical"]);
    assert_eq!(code(&r), "panel_term_empty", "{r}");
    assert!(r["error"]["message"].as_str().unwrap().contains("ghost"));
    assert!(!w.join(".jurygraph/runs").exists(), "nothing was written");
}

#[test]
fn a_rule_gives_the_panel_lenses_spend_and_detail_one_run_per_lens() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let d = ok(brief(w, &["--rule", "public-surface"]));
    assert_eq!(d["rule"], "public-surface");
    let runs = d["runs"].as_array().unwrap();
    let lenses: Vec<&str> = runs.iter().map(|r| r["lens"]["name"].as_str().unwrap()).collect();
    assert_eq!(lenses, vec!["critical", "recall"]);
    assert!(runs.iter().all(|r| r["briefs"].as_array().unwrap().len() == 8));
    assert!(runs.iter().all(|r| r["detail"] == "full"), "{d}");
    assert_eq!(d["estimate"], json!({"briefs": 16, "tokens": 320000}));
    assert!(d["override"].is_null() && d["large"].is_null(), "{d}");
    let row = run_row(w, runs[0]["run"].as_i64().unwrap());
    assert_eq!(row.override_json, None, "following the rule is no override");
    // a cheap rule: one persona, primary first
    let c = ok(brief(w, &["--rule", "copy"]));
    let runs = c["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["briefs"].as_array().unwrap().len(), 1);
    assert_eq!(runs[0]["briefs"][0]["stake"], "primary");
    assert_eq!(c["estimate"], json!({"briefs": 1, "tokens": 20000}));
    // saying the rule's own spend is no departure
    let same = brief(w, &["--rule", "copy", "--spend", "1"]);
    assert_eq!(same["ok"], true, "{same}");
}

#[test]
fn a_rule_that_cannot_run_is_refused_by_name() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    assert_eq!(code(&brief(w, &["--rule", "ghost-panel"])), "rule_panel_unknown");
    assert_eq!(code(&brief(w, &["--rule", "ghost-lens"])), "rule_lens_unknown");
    // no override can run a jury on a human class, or where no jury is wanted
    for extra in [vec![], vec!["--spend", "2", "--reason", "x"], vec!["--panel", "docs-public", "--reason", "x"]] {
        let mut a = vec!["--rule", "money"];
        a.extend(extra);
        assert_eq!(code(&brief(w, &a)), "human_class_under_persona_quorum", "{a:?}");
    }
    assert_eq!(code(&brief(w, &["--rule", "internals"])), "invalid_arguments");
    assert_eq!(code(&brief(w, &["--rule", "nosuch"])), "not_found");
    std::fs::remove_file(w.join("jurygraph.toml")).unwrap();
    assert_eq!(code(&brief(w, &["--rule", "copy"])), "not_found");
    assert!(!w.join(".jurygraph/runs").exists());
}

#[test]
fn a_departure_from_the_rule_needs_a_reason_and_is_recorded_on_the_run() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let r = brief(w, &["--rule", "public-surface", "--spend", "3"]);
    assert_eq!(code(&r), "override_without_reason", "{r}");
    assert!(r["error"]["message"].as_str().unwrap().contains("spend"));
    assert!(!w.join(".jurygraph/runs").exists(), "nothing was written");
    let blank = brief(w, &["--rule", "public-surface", "--spend", "3", "--reason", "  "]);
    assert_eq!(code(&blank), "override_without_reason");
    let d = ok(brief(
        w,
        &["--rule", "public-surface", "--spend", "3", "--reason", "cheap first pass"],
    ));
    let runs = d["runs"].as_array().unwrap();
    assert!(runs.iter().all(|r| r["briefs"].as_array().unwrap().len() == 3));
    assert_eq!(d["estimate"]["briefs"], 6);
    assert_eq!(d["override"]["fields"]["spend"], json!({"rule": 0, "given": 3}));
    let want = json!({
        "rule": "public-surface",
        "fields": {"spend": {"rule": 0, "given": 3}},
        "reason": "cheap first pass",
        "by": "agent"
    });
    for r in runs {
        assert_eq!(run_row(w, r["run"].as_i64().unwrap()).override_json, Some(want.clone()));
    }
    // --by names who
    let d = ok(brief(
        w,
        &["--rule", "copy", "--lens", "taste,critical", "--reason", "tone matters", "--by", "ana"],
    ));
    assert_eq!(d["override"]["by"], "ana");
    assert_eq!(
        d["override"]["fields"]["lenses"],
        json!({"rule": ["recall"], "given": ["taste", "critical"]})
    );
    assert_eq!(d["runs"].as_array().unwrap().len(), 2);
}

#[test]
fn every_field_that_differs_from_the_rule_is_a_departure() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    pool(w, "zzz", "zebra", 2);
    ok(jg(w, &["panel", "add", "zebras", "--query", "2 zebra=primary"]));
    for (extra, field) in [
        (vec!["--lens", "taste"], "lenses"),
        (vec!["--detail", "compact"], "detail"),
        (vec!["--panel", "zebras"], "panel"),
        (vec!["--query", "1 builder=primary"], "panel"),
    ] {
        let mut a = vec!["--rule", "public-surface"];
        a.extend(extra.clone());
        let r = brief(w, &a);
        assert_eq!(code(&r), "override_without_reason", "{extra:?}: {r}");
        assert!(r["error"]["message"].as_str().unwrap().contains(field), "{r}");
        a.extend(["--reason", "because"]);
        let d = brief(w, &a);
        assert_eq!(d["ok"], true, "{extra:?}: {d}");
        assert!(d["data"]["override"]["fields"][field].is_object(), "{d}");
    }
}

#[test]
fn above_twelve_seats_needs_allow_large_and_above_a_thousand_is_never_allowed() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, "rdr", "reader", 20);
    doc(w, "pricing");
    ok(jg(w, &["panel", "add", "wide", "--query", "13 reader=bystander", "--allow-large"]));
    let base = ["--panel", "wide", "--lens", "recall"];
    // 13 seats without the flag
    let r = brief(w, &base);
    assert_eq!(code(&r), "panel_too_large", "{r}");
    assert_eq!(r["data"]["ceiling"], 12);
    assert!(!w.join(".jurygraph/runs").exists());
    // spending 12 of them is within the ceiling
    let mut a = base.to_vec();
    a.extend(["--spend", "12"]);
    let d = ok(brief(w, &a));
    assert!(d["large"].is_null(), "12 is not large: {d}");
    // 13 with the flag: ok, flagged large, with the cost shown
    let mut a = base.to_vec();
    a.extend(["--allow-large", "--reason", "a wide read"]);
    let d = ok(brief(w, &a));
    assert_eq!(d["briefs"].as_array().unwrap().len(), 13);
    assert_eq!(d["large"], true);
    assert_eq!(d["estimate"], json!({"briefs": 13, "tokens": 195000}));
    // more than a thousand: never, flag or not
    for extra in [vec![], vec!["--allow-large"]] {
        let mut a = vec!["--query", "1001 reader=bystander", "--lens", "recall"];
        a.extend(extra);
        let r = brief(w, &a);
        assert_eq!(code(&r), "panel_too_large", "{r}");
        assert_eq!(r["data"]["ceiling"], 1000, "{r}");
    }
}

#[test]
fn a_brief_above_twelve_seats_under_a_rule_needs_a_reason() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, "rdr", "reader", 20);
    doc(w, "pricing");
    ok(jg(w, &["panel", "add", "wide", "--query", "14 reader=bystander", "--allow-large"]));
    std::fs::write(
        w.join("jurygraph.toml"),
        "[[rule]]\nclass = \"crowd\"\npaths = [\"c/**\"]\nquorum = \"low\"\npanel = \"wide\"\nlenses = [\"recall\"]\n",
    )
    .unwrap();
    let r = ok(brief(w, &["--rule", "crowd"]));
    assert_eq!(r["capped"], true, "spend 0 is the full panel capped at 12: {r}");
    let r = brief(w, &["--rule", "crowd", "--allow-large"]);
    assert_eq!(code(&r), "override_without_reason", "{r}");
    assert!(r["error"]["message"].as_str().unwrap().contains("seats"));
    let d = ok(brief(w, &["--rule", "crowd", "--allow-large", "--reason", "a wide read"]));
    assert_eq!(d["override"]["fields"]["seats"], json!({"rule": 12, "given": 14}));
    assert_eq!(d["large"], true);
}

#[test]
fn an_ad_hoc_query_runs_a_panel_that_is_not_saved() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let d = ok(brief(
        w,
        &["--query", "2 builder=primary, 1 approver=secondary", "--lens", "critical"],
    ));
    assert_eq!(stake_counts(&d["briefs"]), (2, 1, 0), "{d}");
    assert_eq!(d["briefs"].as_array().unwrap().len(), 3);
    let again = ok(brief(
        w,
        &["--query", "2 builder=primary, 1 approver=secondary", "--lens", "critical"],
    ));
    assert_eq!(sig(&d["briefs"]), sig(&again["briefs"]), "deterministic");
    // not saved as a panel
    let r = jg(w, &["panel", "resolve", "adhoc"]);
    assert_eq!(code(&r), "not_found");
    // a stake is needed from the query or from --stake
    let r = brief(w, &["--query", "2 builder", "--lens", "critical"]);
    assert_eq!(code(&r), "persona_without_stake", "{r}");
    // a panel and a query together are two answers to one question
    let r = brief(w, &["--panel", "docs-public", "--query", "1 builder=primary", "--lens", "critical"]);
    assert_eq!(code(&r), "invalid_arguments", "{r}");
}

#[test]
fn the_estimate_is_briefs_times_tokens_per_brief() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let d = ok(brief(w, &["--panel", "docs-public", "--lens", "critical", "--spend", "5"]));
    assert_eq!(d["estimate"], json!({"briefs": 5, "tokens": 100000}), "toml says 20000");
    std::fs::remove_file(w.join("jurygraph.toml")).unwrap();
    let d = ok(brief(w, &["--panel", "docs-public", "--lens", "critical", "--spend", "5"]));
    assert_eq!(d["estimate"], json!({"briefs": 5, "tokens": 75000}), "default 15000");
}

#[test]
fn matrix_shows_the_override_in_the_table_and_the_json() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let d = ok(brief(
        w,
        &["--rule", "copy", "--spend", "2", "--reason", "two voices for the tone"],
    ));
    let run = d["runs"][0]["run"].as_i64().unwrap().to_string();
    let m = ok(jg(w, &["matrix", &run, "--table"]));
    let table = m["table"].as_str().unwrap();
    assert!(
        table.starts_with("override (rule copy): spend 1\u{2192}2 \u{2014} reason: two voices for the tone\n"),
        "{table:?}"
    );
    assert_eq!(m["override"]["reason"], "two voices for the tone");
    assert_eq!(m["override"]["rule"], "copy");
    // a run that followed its rule has neither
    let plain = ok(brief(w, &["--rule", "copy"]));
    let run = plain["runs"][0]["run"].as_i64().unwrap().to_string();
    let m = ok(jg(w, &["matrix", &run, "--table"]));
    assert!(!m["table"].as_str().unwrap().contains("override"), "{m}");
    assert!(m["override"].is_null());
}

// ---- should-run: --subject, and a warning for a panel the store lacks ----

fn should_run(w: &Path, extra: &[&str]) -> Value {
    let mut a = vec!["should-run"];
    a.extend_from_slice(extra);
    jg(w, &a)
}

#[test]
fn should_run_classifies_the_recorded_source_paths_of_a_subject() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    std::fs::write(
        w.join("jurygraph.toml"),
        "[[rule]]\nclass = \"docs\"\npaths = [\"src/**\"]\nquorum = \"low\"\npanel = \"docs-public\"\nlenses = [\"recall\"]\nspend = 2\n",
    )
    .unwrap();
    let r = should_run(w, &["--subject", "pricing"]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"]["quorum"], "low", "{r}");
    assert_eq!(r["data"]["hits"][0]["paths"], json!(["src/pricing.md"]), "{r}");
    assert_eq!(r["data"]["spend"][0]["briefs"], 2);
    // with --paths too: the union
    let r = should_run(w, &["--subject", "pricing", "--paths", "elsewhere/x.rs"]);
    assert_eq!(r["data"]["unmatched"], json!(["elsewhere/x.rs"]), "{r}");
    assert_eq!(code(&should_run(w, &["--subject", "ghost"])), "not_found");
}

#[test]
fn should_run_warns_about_a_panel_or_lens_the_store_lacks() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let r = should_run(w, &["--paths", "g/a.txt"]);
    assert_eq!(r["ok"], true, "{r}");
    let warnings = r["data"]["warnings"].as_array().unwrap();
    let text = serde_json::to_string(warnings).unwrap();
    assert!(text.contains("nope") && text.contains("nolens"), "{text}");
    // a rule whose panel and lenses exist warns of nothing
    let r = should_run(w, &["--paths", "copy/a.txt"]);
    assert!(r["data"]["warnings"].is_null(), "{r}");
}

// ---- persona generate: the two-step brief ----

#[test]
fn the_generate_brief_asks_for_the_roles_first_and_keeps_the_count_the_callers() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    let m = w.join("faq.md");
    std::fs::write(&m, "# FAQ\nWho is this for?\n").unwrap();
    let d = ok(jg(w, &["persona", "generate", "--from", m.to_str().unwrap(), "--count", "5"]));
    let text = std::fs::read_to_string(w.join(d["brief"].as_str().unwrap())).unwrap();
    let step1 = text.find("Step 1").expect("a first step");
    let step2 = text.find("Step 2").expect("a second step");
    assert!(step1 < step2);
    let first = &text[step1..step2];
    for word in ["roles", "sectors", "objections"] {
        assert!(first.contains(word), "step 1 names {word}: {first}");
    }
    assert!(text[step2..].contains("Write 5 personas"), "the count stays --count");
    assert!(text.contains("never the number of roles"), "{text}");
}

// ---- fix round 1 (review I1-I6, M1-M6) ----

#[test]
fn i1_stake_and_draw_under_a_rule_are_overrides() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    // --stake that differs from the term
    let r = brief(w, &["--rule", "copy", "--stake", "bld-000=bystander,app-000=primary"]);
    assert_eq!(code(&r), "override_without_reason", "{r}");
    assert!(r["error"]["message"].as_str().unwrap().contains("stakes"), "{r}");
    let d = ok(brief(
        w,
        &["--rule", "copy", "--stake", "bld-000=bystander", "--reason", "try the bystander view"],
    ));
    let f = &d["override"]["fields"]["stakes"];
    assert_eq!(f["rule"], json!({"bld-000": "primary"}), "{d}");
    assert_eq!(f["given"], json!({"bld-000": "bystander"}), "{d}");
    let row = run_row(w, d["runs"][0]["run"].as_i64().unwrap());
    assert!(row.override_json.unwrap()["fields"]["stakes"].is_object());
    // a --stake that says what the term says is no departure
    let same = brief(w, &["--rule", "copy", "--stake", "bld-000=primary"]);
    assert_eq!(same["ok"], true, "{same}");
    // --draw that differs from the derived seed
    let r = brief(w, &["--rule", "public-surface", "--draw", "0000000000000000"]);
    assert_eq!(code(&r), "override_without_reason", "{r}");
    assert!(r["error"]["message"].as_str().unwrap().contains("draw"), "{r}");
    let d = ok(brief(
        w,
        &["--rule", "public-surface", "--draw", "00000000000000ff", "--reason", "re-roll"],
    ));
    assert_eq!(d["override"]["fields"]["draw"]["given"], "00000000000000ff");
    // the derived seed, given by hand, is none
    let derived = hex::encode(jurygraph::domain::hash::draw_seed("pricing", "docs-public"));
    let same = brief(w, &["--rule", "public-surface", "--draw", &derived]);
    assert_eq!(same["ok"], true, "{same}");
}

#[test]
fn i2_a_persona_added_to_one_term_leaves_the_other_terms_seats() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, "bld", "builder", 1);
    pool(w, "rdr", "reader", 8);
    doc(w, "pricing");
    ok(jg(w, &["panel", "add", "p2", "--query", "1 builder=primary, 3 others=bystander"]));
    let readers = |d: &Value| -> Vec<String> {
        d["briefs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|b| b["stake"] == "bystander")
            .map(|b| b["persona"].as_str().unwrap().to_string())
            .collect()
    };
    let before = readers(&ok(brief(w, &["--panel", "p2", "--lens", "critical"])));
    assert_eq!(before.len(), 3);
    // the builder term grows from "equal to its seats" to "bigger": it is now shuffled, the readers must not move
    pool(w, "bldx", "builder", 4);
    let after = readers(&ok(brief(w, &["--panel", "p2", "--lens", "critical"])));
    assert_eq!(before, after, "adding builders never moves the bystander seats");
}

#[test]
fn i3_a_subject_that_falls_under_a_human_rule_is_never_briefed_under_another_rule() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    std::fs::create_dir_all(w.join("prices")).unwrap();
    let f = w.join("prices/table.md");
    std::fs::write(&f, "# prices\n").unwrap();
    ok(jg(w, &["subject", "add", "pricetable", "--kind", "prices", "--attach", f.to_str().unwrap()]));
    for extra in [vec![], vec!["--reason", "please"], vec!["--spend", "2", "--reason", "please"]] {
        let mut a = vec!["brief", "--subject", "pricetable", "--rule", "copy", "--model", "m"];
        a.extend(extra);
        let r = jg(w, &a);
        assert_eq!(code(&r), "human_class_under_persona_quorum", "{r}");
        assert_eq!(r["data"]["path"], "prices/table.md", "{r}");
        assert_eq!(r["data"]["class"], "money", "{r}");
    }
    assert!(!w.join(".jurygraph/runs").exists());
}

#[test]
fn i4_a_rule_with_spend_zero_caps_the_full_panel_at_twelve() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    pool(w, "rdz", "reader", 12);
    ok(jg(
        w,
        &["panel", "add", "big", "--query", "3 builder=primary, 1 newcomer=primary, 10 others=bystander", "--allow-large"],
    ));
    std::fs::write(
        w.join("jurygraph.toml"),
        "[[rule]]\nclass = \"crowd\"\npaths = [\"c/**\"]\nquorum = \"high\"\npanel = \"big\"\nlenses = [\"recall\"]\nspend = 0\n",
    )
    .unwrap();
    let d = ok(brief(w, &["--rule", "crowd"]));
    assert_eq!(d["capped"], true, "{d}");
    assert_eq!(d["runs"][0]["briefs"].as_array().unwrap().len(), 12, "{d}");
    assert!(d["override"].is_null() && d["large"].is_null(), "{d}");
    // with --allow-large the cap is 1000, and 14 seats are a departure that needs its reason
    let r = brief(w, &["--rule", "crowd", "--allow-large"]);
    assert_eq!(code(&r), "override_without_reason", "{r}");
    let d = ok(brief(w, &["--rule", "crowd", "--allow-large", "--reason", "whole room"]));
    assert_eq!(d["runs"][0]["briefs"].as_array().unwrap().len(), 14, "{d}");
    assert!(d["capped"].is_null(), "{d}");
}

#[test]
fn i5_overflowing_seat_totals_and_estimates_are_refused_never_panic_or_saved() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let r = jg(w, &["panel", "add", "wrap", "--query", "18446744073709551615 builder, 1 others"]);
    assert_eq!(code(&r), "panel_too_large", "{r}");
    let r = jg(w, &["panel", "add", "wrap", "--query", "18446744073709551615 reader=bystander, 1 builder=primary", "--allow-large"]);
    assert_eq!(code(&r), "panel_too_large", "{r}");
    let r = brief(w, &["--query", "18446744073709551615 reader=bystander, 1 builder=primary", "--lens", "recall", "--allow-large"]);
    assert_eq!(code(&r), "panel_too_large", "{r}");
    // the estimate
    std::fs::write(w.join("jurygraph.toml"), "tokens_per_brief = 9223372036854775807\n").unwrap();
    let r = brief(w, &["--panel", "docs-public", "--lens", "recall", "--spend", "3"]);
    assert_eq!(code(&r), "panel_too_large", "{r}");
    assert!(!w.join(".jurygraph/runs").exists(), "no orphan run");
}

#[test]
fn m3_the_matrix_header_names_the_rule_the_runs_and_cannot_be_forged() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let plain = ok(brief(w, &["--rule", "copy"]));
    let over = ok(brief(
        w,
        &["--rule", "copy", "--spend", "2", "--reason", "cheap\nworks 3 3 3"],
    ));
    let a = plain["runs"][0]["run"].as_i64().unwrap().to_string();
    let b = over["runs"][0]["run"].as_i64().unwrap().to_string();
    let m = ok(jg(w, &["matrix", &a, &b, "--table"]));
    let table = m["table"].as_str().unwrap();
    let first = table.lines().next().unwrap();
    assert_eq!(
        first,
        format!("run {b}: override (rule copy): spend 1\u{2192}2 \u{2014} reason: cheap\\nworks 3 3 3")
    );
    assert!(!table.lines().any(|l| l.starts_with("works 3")), "{table}");
    // alone, the run needs no prefix
    let m = ok(jg(w, &["matrix", &b, "--table"]));
    assert!(m["table"].as_str().unwrap().starts_with("override (rule copy): spend 1\u{2192}2"), "{m}");
}

#[test]
fn m4_lens_order_is_not_a_departure_and_m5_query_spacing_does_not_change_the_draw() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    world(w);
    let d = brief(w, &["--rule", "public-surface", "--lens", "recall,critical"]);
    assert_eq!(d["ok"], true, "{d}");
    let a = ok(brief(w, &["--query", "1 builder=primary, 2 others=bystander", "--lens", "recall"]));
    let b = ok(brief(w, &["--query", "1 builder=primary,2   others=bystander", "--lens", "recall"]));
    assert_eq!(sig(&a["briefs"]), sig(&b["briefs"]));
    assert_eq!(a["draw"], b["draw"]);
}

#[test]
fn m6_a_v2_store_is_refused_untouched_with_a_readable_message() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".jurygraph");
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("store.db");
    {
        let c = rusqlite::Connection::open(&db).unwrap();
        c.execute_batch(
            "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO meta VALUES ('schema_version', '2');
             CREATE TABLE run (id INTEGER PRIMARY KEY, subject_name TEXT NOT NULL);",
        )
        .unwrap();
    }
    let before = std::fs::read(&db).unwrap();
    let err = Store::open(d.path()).map(|_| ()).expect_err("a v2 store is refused");
    let msg = err.to_string();
    assert!(msg.contains("schema_version 2") && !msg.contains("Ok("), "{msg}");
    assert_eq!(std::fs::read(&db).unwrap(), before, "the refusal wrote nothing");
}
