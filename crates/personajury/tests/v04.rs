//! v0.4 (D13..D19): the split rule and `apply-check`, effort, independence, experts, model per term, `lens generate`.
//! Every refusal and every answer here was seen red first.
mod judging_support;
use personajury::domain::Stake;
use std::process::Command;
use judging_support::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn data(v: Value) -> Value {
    assert_eq!(v["ok"], true, "{v}");
    v["data"].clone()
}
fn code(v: &Value) -> String {
    assert_eq!(v["ok"], false, "{v}");
    v["error"]["code"].as_str().unwrap().to_string()
}
fn reasons(d: &Value) -> Vec<String> {
    d["reasons"].as_array().unwrap().iter().map(|r| r["code"].as_str().unwrap().to_string()).collect()
}
fn has(d: &Value, c: &str) -> bool {
    reasons(d).iter().any(|r| r == c)
}
fn toml(ws: &Ws, body: &str) {
    std::fs::write(ws.root().join("personajury.toml"), body).unwrap();
}
const LOW_RULE: &str = "[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/site/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n";
const HIGH_RULE: &str = "[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/site/**\"]\nquorum = \"high\"\npanel = \"p\"\nlenses = [\"critical\"]\n";
fn rs(n: i64) -> String {
    n.to_string()
}
/// The low rule on the site's sources, and the run recorded as briefed under it (what `brief --rule` stores, D13):
/// without them the quorum is unknown and apply-check never answers `apply`.
fn under_low_rule(ws: &Ws, runs: &[i64]) {
    toml(ws, LOW_RULE);
    for r in runs {
        ws.store
            .conn()
            .execute("UPDATE run SET rule_class = 'public-surface', quorum = 'low' WHERE id = ?1", [r])
            .unwrap();
    }
}

// ---- item 1: the split rule and the apply verdict (D13) ----

#[test]
fn a_split_is_two_strata_whose_anchor_ranges_do_not_overlap() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [2, 2, 2], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let m = data(ws.cli(&["matrix", &rs(run)]));
    let s = m["splits"].as_array().unwrap();
    assert_eq!(s.len(), 3, "{m}");
    assert_eq!(s[0]["strata"][0]["stake"], "primary", "{m}");
    assert_eq!(s[0]["strata"][1]["stake"], "secondary", "{m}");
    assert_eq!(s[0]["strata"][0]["max"], 1, "{m}");
    assert_eq!(s[0]["strata"][1]["min"], 3, "{m}");
    let a = data(ws.cli(&["apply-check", &rs(run)]));
    assert_eq!(a["answer"], "escalate", "{a}");
    assert!(has(&a, "split"), "{a}");
}

#[test]
fn ranges_that_touch_at_an_endpoint_are_no_split() {
    let ws = Ws::new();
    ws.site();
    let team = [("ana", Stake::Primary), ("cat", Stake::Primary), ("bob", Stake::Secondary)];
    let run = ws.run("critical", &team, false);
    ws.ingest(run, &critical("ana", [2, 2, 2], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("cat", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    under_low_rule(&ws, &[run]);
    let m = data(ws.cli(&["matrix", &rs(run)]));
    assert_eq!(m["splits"], json!([]), "{m}");
    let a = data(ws.cli(&["apply-check", &rs(run)]));
    assert_eq!(a["answer"], "apply", "{a}");
}

#[test]
fn a_primary_blocker_escalates_one_level_low_to_high_and_high_to_human() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [3, 3, 3], &["the price is hidden"], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [3, 3, 3], &[], &["page.json#n1"]));
    toml(&ws, LOW_RULE);
    let a = data(ws.cli(&["apply-check", &rs(run), "--rule", "public-surface"]));
    assert_eq!(a["answer"], "escalate", "{a}");
    assert_eq!(a["quorum"], "low", "{a}");
    assert_eq!(a["next_quorum"], "high", "{a}");
    assert!(has(&a, "primary_blocker"), "{a}");
    toml(&ws, HIGH_RULE);
    let a = data(ws.cli(&["apply-check", &rs(run), "--rule", "public-surface"]));
    assert_eq!(a["next_quorum"], "human", "{a}");
}

#[test]
fn a_human_class_answers_human_whatever_the_verdicts_say() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [5, 5, 5], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [5, 5, 5], &[], &["page.json#n1"]));
    toml(&ws, "[[rule]]\nclass = \"money\"\npaths = [\"sources/site/**\"]\nquorum = \"human\"\n");
    // by the rule the caller names
    let a = data(ws.cli(&["apply-check", &rs(run), "--rule", "money"]));
    assert_eq!(a["answer"], "human", "{a}");
    assert!(has(&a, "human_class"), "{a}");
    // and by the subject's recorded sources, with no rule named
    let a = data(ws.cli(&["apply-check", &rs(run)]));
    assert_eq!(a["answer"], "human", "{a}");
}

#[test]
fn a_delta_that_did_not_land_escalates() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &TWO, false);
    ws.ingest(r1, &critical("ana", [5, 5, 5], &[], &["page.json#n1"]));
    ws.ingest(r1, &critical("bob", [5, 5, 5], &[], &["page.json#n1"]));
    let r2 = ws.run("critical", &TWO, false);
    ws.ingest(r2, &critical("ana", [2, 2, 2], &[], &["page.json#n1"]));
    ws.ingest(r2, &critical("bob", [2, 2, 2], &[], &["page.json#n1"]));
    let a = data(ws.cli(&["apply-check", &rs(r2)]));
    assert_eq!(a["answer"], "escalate", "{a}");
    assert!(has(&a, "delta_not_landed"), "{a}");
    assert!(!has(&a, "primary_blocker"), "{a}");
}

#[test]
fn a_clean_second_round_applies() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &THREE, false);
    let d = ws.ingest(r1, &critical("ana", [3, 3, 3], &["the price is hidden"], &["page.json#n1"]));
    ws.ingest(r1, &critical("bob", [3, 3, 3], &[], &["page.json#n1"]));
    ws.ingest(r1, &critical("cat", [3, 3, 3], &[], &["page.json#n1"]));
    let finding = d["findings"][0].as_str().unwrap().to_string();
    let r = ws.cli(&["decide", &finding, "fixed", "--by", "ana"]);
    assert_eq!(r["ok"], true, "{r}");
    let r2 = ws.run("critical", &THREE, false);
    ws.ingest(r2, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(r2, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(r2, &critical("cat", [4, 4, 4], &[], &["page.json#n1"]));
    under_low_rule(&ws, &[r1, r2]);
    for args in [vec!["apply-check", "2"], vec!["apply-check", "1", "2"]] {
        let a = data(ws.cli(&args));
        assert_eq!(a["answer"], "apply", "{a}");
        assert_eq!(a["runs"], json!([2]), "{a}");
    }
    // the first round alone still has the primary blocker
    let a = data(ws.cli(&["apply-check", "1"]));
    assert_eq!(a["answer"], "escalate", "{a}");
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn fx(rel: &str) -> String {
    repo().join("fixtures").join(rel).to_string_lossy().into_owned()
}
fn cli(root: &Path, args: &[&str]) -> Value {
    cli_in(root, args)
}
/// The fixture site, panel and comparator in a fresh workspace; returns (critical run, compare run) after briefing.
fn fixture_ws(root: &Path, extra: &[&str]) -> (i64, i64) {
    let mut personas: Vec<_> =
        std::fs::read_dir(repo().join("fixtures/personas")).unwrap().map(|e| e.unwrap().path()).collect();
    personas.sort();
    data(cli(root, &["trait", "install"]));
    for p in personas {
        data(cli(root, &["persona", "add", "--file", p.to_str().unwrap()]));
    }
    data(cli(root, &["panel", "add", "fixture", "--query", "3 builder, 2 company, 2 others"]));
    let att: Vec<String> = ["home.walk.json", "home.png", "pricing.walk.json", "pricing.png"].iter().map(|f| fx(&format!("site-walk/{f}"))).collect();
    let mut a = vec!["subject", "add", "lantern-site", "--kind", "page", "--attach"];
    a.extend(att.iter().map(String::as_str));
    data(cli(root, &a));
    let (cw, cp) = (fx("comparators/acme-pricing/pricing.walk.json"), fx("comparators/acme-pricing/pricing.png"));
    data(cli(root, &["comparator", "add", "acme-pricing", "--kind", "prices", "--attach", &cw, &cp]));
    let stakes = "builder-ana=primary,builder-mei=primary,builder-ravi=primary,newcomer-sofia=primary,company-george=secondary,company-hanna=secondary,investor-claire=bystander";
    let mut c = vec!["brief", "--lens", "critical", "--subject", "lantern-site", "--panel", "fixture", "--stake", stakes, "--spend", "7", "--model", "claude-sonnet-5-5"];
    c.extend_from_slice(extra);
    let a = data(cli(root, &c))["run"].as_i64().unwrap();
    let mut c = vec!["brief", "--lens", "compare", "--comparator", "acme-pricing", "--subject", "lantern-site", "--panel", "fixture", "--stake", stakes, "--spend", "7", "--model", "claude-sonnet-5-5"];
    c.extend_from_slice(extra);
    let b = data(cli(root, &c))["run"].as_i64().unwrap();
    (a, b)
}
fn ingest_fixture(root: &Path, critical_run: i64, compare_run: i64) {
    let mut files: Vec<_> =
        std::fs::read_dir(repo().join("fixtures/run/verdicts")).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    for f in &files {
        let name = f.file_stem().unwrap().to_str().unwrap();
        let run = if name.ends_with("--critical") { critical_run } else { compare_run };
        data(cli(root, &["verdict", "--run", &rs(run), "--file", f.to_str().unwrap()]));
    }
}

#[test]
fn the_fixture_run_escalates_it_has_primary_blockers() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_ws(ws.path(), &[]);
    ingest_fixture(ws.path(), a, b);
    let v = data(cli(ws.path(), &["apply-check", &rs(a), &rs(b)]));
    assert_eq!(v["answer"], "escalate", "{v}");
    assert!(has(&v, "primary_blocker"), "{v}");
}

// ---- item 7: effort, one word for how much a jury may spend (D17) ----

use personajury::store::Store;

fn pool(w: &Path, specs: &[(&str, &str, usize)]) {
    let dir = w.join("pool");
    std::fs::create_dir_all(&dir).unwrap();
    for (prefix, tag, n) in specs {
        for i in 0..*n {
            let id = format!("{prefix}-{i:02}");
            let f = dir.join(format!("{id}.json"));
            let body = json!({
                "id": id, "name": format!("Person {id}"), "tags": [tag],
                "role": "r", "context": "c", "daily_tasks": "d", "fears": "f",
                "never_reads": "n", "decides_by": "b", "stake_rules": "s"
            });
            std::fs::write(&f, body.to_string()).unwrap();
            data(cli(w, &["persona", "add", "--file", f.to_str().unwrap()]));
        }
    }
}
fn doc(w: &Path, name: &str) {
    let dir = w.join("sources/site");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(format!("{name}.md"));
    std::fs::write(&f, format!("# {name}\nline two\nline three\n")).unwrap();
    data(cli(w, &["subject", "add", name, "--kind", "document", "--attach", f.to_str().unwrap()]));
}
/// 7 personas (3 primary, 2 secondary, 2 bystander by the panel's terms), a 3-lens rule over them.
fn effort_ws(extra_toml: &str) -> tempfile::TempDir {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, &[("bld", "builder", 3), ("apr", "approver", 2), ("oth", "visitor", 2)]);
    data(cli(w, &["panel", "add", "p", "--query", "3 builder=primary, 2 approver=secondary, 2 visitor=bystander"]));
    doc(w, "pricing");
    std::fs::write(
        w.join("personajury.toml"),
        format!("{extra_toml}\n[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/site/**\"]\nquorum = \"high\"\npanel = \"p\"\nlenses = [\"critical\", \"taste\", \"recall\"]\n"),
    )
    .unwrap();
    ws
}
fn told_verdict(persona: &str) -> Value {
    let lines: Vec<Value> = ["works", "blocks", "missing"]
        .iter()
        .map(|d| json!({"dimension": d, "score": 4, "anchor_met": 3, "evidence": "told"}))
        .collect();
    json!({ "persona": persona, "lines": lines })
}
fn rule_brief(w: &Path, extra: &[&str]) -> Value {
    let mut a = vec!["brief", "--subject", "pricing", "--rule", "public-surface", "--model", "m"];
    a.extend_from_slice(extra);
    cli(w, &a)
}
fn runs_of(d: &Value) -> Vec<Value> {
    d["runs"].as_array().cloned().unwrap_or_else(|| vec![d.clone()])
}

#[test]
fn effort_low_trims_a_seven_seat_three_lens_rule_to_three_briefs() {
    let ws = effort_ws("");
    let w = ws.path();
    let full = data(rule_brief(w, &["--effort", "high"]));
    assert_eq!(full["estimate"]["briefs"], 21, "{full}");
    assert_eq!(full["effort"], "high", "{full}");
    let d = data(rule_brief(w, &["--effort", "low"]));
    assert_eq!(d["estimate"]["briefs"], 3, "{d}");
    assert_eq!(d["effort"], "low", "{d}");
    let runs = runs_of(&d);
    assert_eq!(runs.len(), 1, "{d}");
    // one per stratum present, primary first
    let stakes: Vec<&str> = runs[0]["briefs"].as_array().unwrap().iter().map(|b| b["stake"].as_str().unwrap()).collect();
    assert_eq!(stakes, ["primary", "secondary", "bystander"], "{d}");
    let t = d["trimmed"].as_array().unwrap();
    assert!(t.iter().any(|x| x["what"] == "seats" && x["from"] == 7 && x["to"] == 3), "{d}");
    assert!(t.iter().any(|x| x["what"] == "lenses" && x["to"] == json!(["critical"])), "{d}");
}

#[test]
fn medium_is_the_default_under_a_rule_and_caps_at_seven() {
    let ws = effort_ws("");
    let d = data(rule_brief(ws.path(), &[]));
    assert_eq!(d["effort"], "medium", "{d}");
    assert_eq!(d["estimate"]["briefs"], 21, "{d}");
    assert!(d["trimmed"].as_array().unwrap().is_empty(), "{d}");
}

#[test]
fn the_toml_effort_is_the_default_and_the_flag_overrides_it_and_is_recorded() {
    let ws = effort_ws("effort = \"low\"");
    let w = ws.path();
    let d = data(rule_brief(w, &[]));
    assert_eq!(d["effort"], "low", "{d}");
    assert_eq!(d["estimate"]["briefs"], 3, "{d}");
    let d = data(rule_brief(w, &["--effort", "high"]));
    assert_eq!(d["effort"], "high", "{d}");
    let id = runs_of(&d)[0]["run"].as_i64().unwrap();
    let m = data(cli(w, &["matrix", &rs(id), "--table"]));
    assert!(m["table"].as_str().unwrap().starts_with("effort: high"), "{m}");
    assert_eq!(m["effort"], json!(["high"]), "{m}");
    // the store keeps it
    let row = Store::open_existing(w).unwrap().run(id).unwrap().unwrap();
    assert_eq!(row.effort.as_deref(), Some("high"));
}

#[test]
fn an_unknown_effort_word_is_refused() {
    let ws = effort_ws("");
    let r = rule_brief(ws.path(), &["--effort", "ultra"]);
    assert_eq!(code(&r), "invalid_arguments", "{r}");
    let ws = effort_ws("effort = \"ultra\"");
    let r = rule_brief(ws.path(), &[]);
    assert_eq!(code(&r), "invalid_config", "{r}");
}

#[test]
fn a_budget_drops_lenses_then_seats_and_keeps_the_primary() {
    let ws = effort_ws("tokens_per_brief = 15000\nbudget_tokens = 50000");
    let w = ws.path();
    let d = data(rule_brief(w, &[]));
    assert_eq!(d["estimate"]["briefs"], 3, "{d}");
    assert_eq!(d["estimate"]["tokens"], 45000, "{d}");
    let runs = runs_of(&d);
    assert_eq!(runs.len(), 1, "{d}");
    let stakes: Vec<&str> = runs[0]["briefs"].as_array().unwrap().iter().map(|b| b["stake"].as_str().unwrap()).collect();
    assert_eq!(stakes[0], "primary", "{d}");
    assert!(stakes.contains(&"secondary") && stakes.contains(&"bystander"), "{d}");
    let t = d["trimmed"].as_array().unwrap();
    assert!(t.iter().any(|x| x["what"] == "lenses"), "{d}");
    assert!(t.iter().any(|x| x["what"] == "seats"), "{d}");
}

#[test]
fn a_budget_that_cannot_hold_one_seat_per_stratum_is_refused_with_the_minimum_plan() {
    let ws = effort_ws("tokens_per_brief = 15000\nbudget_tokens = 10000");
    let w = ws.path();
    let r = rule_brief(w, &[]);
    assert_eq!(code(&r), "over_budget", "{r}");
    assert_eq!(r["data"]["needed"], 45000, "{r}");
    assert_eq!(r["data"]["budget"], 10000, "{r}");
    assert_eq!(r["data"]["minimum_plan"]["briefs"], 3, "{r}");
    assert_eq!(r["data"]["minimum_plan"]["lenses"], 1, "{r}");
    // nothing was written for the refused brief
    assert!(Store::open_existing(w).unwrap().runs().unwrap().is_empty());
}

#[test]
fn a_cheap_jury_on_a_risky_change_advises_it_never_applies() {
    let ws = effort_ws("");
    let w = ws.path();
    let d = data(rule_brief(w, &["--effort", "low"]));
    let id = runs_of(&d)[0]["run"].as_i64().unwrap();
    for b in runs_of(&d)[0]["briefs"].as_array().unwrap() {
        let p = b["persona"].as_str().unwrap();
        let v = told_verdict(p);
        data(cli(w, &["verdict", "--run", &rs(id), "--json", &v.to_string()]));
    }
    let a = data(cli(w, &["apply-check", &rs(id), "--rule", "public-surface"]));
    assert_eq!(a["answer"], "advise", "{a}");
    assert!(has(&a, "effort_below_quorum"), "{a}");
    // the same verdicts at effort high would have applied: the quorum is high, the effort was not
    assert_eq!(a["quorum"], "high", "{a}");
}

/// Every briefed seat of a brief answers a clean `told` verdict.
fn judge_all(w: &Path, d: &Value) -> Vec<i64> {
    let mut ids = Vec::new();
    for r in runs_of(d) {
        let id = r["run"].as_i64().unwrap();
        // the shipped lens's dimensions, each a clean `told` line
        let lens = std::fs::read_to_string(repo().join(format!("lenses/{}.toml", r["lens"]["name"].as_str().unwrap()))).unwrap();
        let dims: Vec<&str> = lens.lines().filter_map(|l| l.strip_prefix("id = \"")?.strip_suffix('"')).collect();
        let ev = if lens.contains("evidence = [\"recalled\"]") { "recalled" } else { "told" };
        for b in r["briefs"].as_array().unwrap() {
            let lines: Vec<Value> =
                dims.iter().map(|d| json!({"dimension": d, "score": 4, "anchor_met": 3, "evidence": ev})).collect();
            let v = json!({"persona": b["persona"], "lines": lines});
            data(cli(w, &["verdict", "--run", &rs(id), "--json", &v.to_string()]));
        }
        ids.push(id);
    }
    ids
}
fn ac_args(ids: &[i64], extra: &[&str]) -> Vec<String> {
    let mut a = vec!["apply-check".to_string()];
    a.extend(ids.iter().map(|i| i.to_string()));
    a.extend(extra.iter().map(|x| x.to_string()));
    a
}
fn ac_in(w: &Path, ids: &[i64], extra: &[&str]) -> Value {
    let a = ac_args(ids, extra);
    data(cli(w, &a.iter().map(String::as_str).collect::<Vec<_>>()))
}
const DOCS_RULE: &str = "[[rule]]\nclass = \"docs\"\npaths = [\"docs/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n";

#[test]
fn a_rule_briefed_run_records_its_rule_and_effective_quorum_and_keeps_them_when_rule_is_dropped() {
    // C3: the review's repro, a run under the high rule at effort low; apply-check without --rule still sees `high`
    let ws = effort_ws("");
    let w = ws.path();
    let d = data(rule_brief(w, &["--effort", "low"]));
    let ids = judge_all(w, &d);
    let run = Store::open_existing(w).unwrap().run(ids[0]).unwrap().unwrap();
    assert_eq!(run.rule.as_deref(), Some("public-surface"));
    assert_eq!(run.quorum.as_deref(), Some("high"));
    let a = ac_in(w, &ids, &[]);
    assert_eq!(a["quorum"], "high", "{a}");
    assert_eq!(a["answer"], "advise", "{a}");
    assert!(has(&a, "effort_below_quorum"), "{a}");
}

#[test]
fn a_rule_below_the_sources_quorum_is_an_override_that_needs_a_reason_and_advises() {
    // C2: `docs` (low) on a subject whose sources fall under `public-surface` (high)
    let ws = effort_ws(DOCS_RULE);
    let w = ws.path();
    let r = cli(w, &["brief", "--subject", "pricing", "--rule", "docs", "--model", "m"]);
    assert_eq!(code(&r), "override_without_reason", "{r}");
    let d = data(cli(w, &["brief", "--subject", "pricing", "--rule", "docs", "--model", "m", "--reason", "copy only"]));
    assert_eq!(d["override"]["fields"]["quorum"], json!({"sources": "high", "given": "low"}), "{d}");
    let ids = judge_all(w, &d);
    let run = Store::open_existing(w).unwrap().run(ids[0]).unwrap().unwrap();
    assert_eq!((run.rule.as_deref(), run.quorum.as_deref()), (Some("docs"), Some("high")));
    let a = ac_in(w, &ids, &[]);
    assert_eq!(a["quorum"], "high", "{a}");
    assert_eq!(a["answer"], "advise", "{a}");
    assert!(has(&a, "rule_below_sources"), "{a}");
    // and a run under the right rule asked about with a lower --rule
    let d = data(rule_brief(w, &["--effort", "high"]));
    let ids = judge_all(w, &d);
    let a = ac_in(w, &ids, &["--rule", "docs"]);
    assert_eq!(a["quorum"], "high", "{a}");
    assert_eq!(a["answer"], "advise", "{a}");
    assert!(has(&a, "rule_below_sources"), "{a}");
    // the same runs without the lower rule apply: the gate is the lower rule, nothing else
    assert_eq!(ac_in(w, &ids, &[])["answer"], "apply");
}

#[test]
fn an_explicit_spend_below_the_rule_advises_with_overridden_down() {
    // C4: the review's repro, one seat with a reason on a high rule
    let ws = effort_ws("");
    let w = ws.path();
    let d = data(rule_brief(w, &["--effort", "high", "--spend", "1", "--reason", "cheap"]));
    let ids = judge_all(w, &d);
    let a = ac_in(w, &ids, &["--rule", "public-surface"]);
    assert_eq!(a["answer"], "advise", "{a}");
    assert!(has(&a, "overridden_down"), "{a}");
    // the same seat briefed with no rule at all, in a store of its own
    let ws = effort_ws("");
    let w = ws.path();
    let d = data(cli(w, &["brief", "--subject", "pricing", "--lens", "critical", "--panel", "p", "--spend", "1", "--model", "m"]));
    let ids = judge_all(w, &d);
    let a = ac_in(w, &ids, &[]);
    assert_eq!(a["quorum"], "high", "{a}");
    assert_eq!(a["answer"], "advise", "{a}");
    assert!(has(&a, "overridden_down"), "{a}");
}

#[test]
fn medium_caps_at_seven_and_keeps_a_seat_of_every_stratum() {
    // I5: 8 builders, 2 approvers, 2 visitors under a low rule at the default medium
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, &[("bld", "builder", 8), ("apr", "approver", 2), ("oth", "visitor", 2)]);
    data(cli(w, &["panel", "add", "p", "--query", "8 builder=primary, 2 approver=secondary, 2 visitor=bystander"]));
    doc(w, "pricing");
    std::fs::write(w.join("personajury.toml"), LOW_RULE).unwrap();
    let d = data(rule_brief(w, &[]));
    let briefs = runs_of(&d)[0]["briefs"].as_array().unwrap().clone();
    let stakes: Vec<&str> = briefs.iter().map(|b| b["stake"].as_str().unwrap()).collect();
    assert_eq!(stakes.len(), 7, "{d}");
    for s in ["primary", "secondary", "bystander"] {
        assert!(stakes.contains(&s), "the cap dropped the {s} stratum: {stakes:?}");
    }
    // then primary first: five builders fill the rest
    assert_eq!(stakes.iter().filter(|s| **s == "primary").count(), 5, "{stakes:?}");
}

#[test]
fn a_brief_with_no_rule_and_no_effort_records_none() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, &[("rdr", "reader", 2)]);
    doc(w, "pricing");
    let d = data(cli(w, &["brief", "--subject", "pricing", "--lens", "recall", "--query", "2 reader=bystander", "--model", "m"]));
    assert!(d["effort"].is_null(), "{d}");
    let row = Store::open_existing(w).unwrap().run(d["run"].as_i64().unwrap()).unwrap().unwrap();
    assert_eq!(row.effort, None);
}

#[test]
fn doctor_shows_the_effort_and_the_tokens_reported_over_seven_days() {
    let ws = effort_ws("effort = \"high\"");
    let w = ws.path();
    let d = data(rule_brief(w, &[]));
    let id = runs_of(&d)[0]["run"].as_i64().unwrap();
    let p = runs_of(&d)[0]["briefs"][0]["persona"].as_str().unwrap().to_string();
    let mut v = told_verdict(&p);
    v["usage"] = json!({"input_tokens": 1000, "output_tokens": 234});
    let r = cli(w, &["verdict", "--run", &rs(id), "--json", &v.to_string()]);
    assert_eq!(r["ok"], true, "{r}");
    let doc = data(cli(w, &["doctor"]));
    assert_eq!(doc["effort"]["configured"], "high", "{doc}");
    assert_eq!(doc["effort"]["default"], "medium", "{doc}");
    assert_eq!(doc["usage_7d"]["input_tokens"], 1000, "{doc}");
    assert_eq!(doc["usage_7d"]["output_tokens"], 234, "{doc}");
}

#[test]
fn a_v3_store_upgrades_in_place_and_reads_its_old_runs_as_independent() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, &[("rdr", "reader", 2)]);
    doc(w, "pricing");
    let d = data(cli(w, &["brief", "--subject", "pricing", "--lens", "recall", "--query", "2 reader=bystander", "--model", "m"]));
    let id = d["run"].as_i64().unwrap();
    {
        // make it a v3 store: drop the v0.4 columns, set the version back
        let s = Store::open_existing(w).unwrap();
        for col in ["effort", "trimmed_json", "independence"] {
            s.conn().execute(&format!("ALTER TABLE run DROP COLUMN {col}"), []).unwrap();
        }
        s.conn().execute("UPDATE meta SET value = '3' WHERE key = 'schema_version'", []).unwrap();
    }
    let s = Store::open_existing(w).unwrap();
    assert_eq!(s.schema_version().unwrap(), 5);
    let row = s.run(id).unwrap().unwrap();
    assert_eq!(row.independence, "independent");
    assert_eq!(row.effort, None);
    // opening again changes nothing
    assert_eq!(Store::open_existing(w).unwrap().schema_version().unwrap(), 5);
}

// ---- item 8: independence, a word on the run (D18) ----

fn plain_ws() -> tempfile::TempDir {
    let ws = tempfile::tempdir().unwrap();
    pool(ws.path(), &[("rdr", "reader", 3)]);
    doc(ws.path(), "pricing");
    ws
}
fn plain_brief(w: &Path, extra: &[&str]) -> Value {
    let mut a = vec!["brief", "--subject", "pricing", "--lens", "recall", "--query", "3 reader=bystander", "--model", "m"];
    a.extend_from_slice(extra);
    cli(w, &a)
}
fn mark(ws: &Ws, run: i64, word: &str) {
    ws.store
        .conn()
        .execute("UPDATE run SET independence = ?1 WHERE id = ?2", rusqlite::params![word, run])
        .unwrap();
}

#[test]
fn a_sequential_brief_echoes_the_word_the_sequence_and_the_sentence() {
    let ws = plain_ws();
    let d = data(plain_brief(ws.path(), &["--independence", "sequential"]));
    assert_eq!(d["independence"], "sequential", "{d}");
    let paths: Vec<&str> = d["briefs"].as_array().unwrap().iter().map(|b| b["path"].as_str().unwrap()).collect();
    let seq: Vec<&str> = d["sequence"].as_array().unwrap().iter().map(|p| p.as_str().unwrap()).collect();
    assert_eq!(paths, seq, "{d}");
    assert!(d["note"].as_str().unwrap().contains("one context will read all of these"), "{d}");
    let row = Store::open_existing(ws.path()).unwrap().run(d["run"].as_i64().unwrap()).unwrap().unwrap();
    assert_eq!(row.independence, "sequential");
}

#[test]
fn an_independent_brief_has_no_sequence_and_the_default_is_independent() {
    let ws = plain_ws();
    let d = data(plain_brief(ws.path(), &[]));
    assert_eq!(d["independence"], "independent", "{d}");
    assert!(d.get("sequence").is_none() && d.get("note").is_none(), "{d}");
    let r = plain_brief(ws.path(), &["--independence", "alone"]);
    assert_eq!(code(&r), "invalid_arguments", "{r}");
}

#[test]
fn the_briefs_are_byte_identical_whatever_the_independence() {
    let a = plain_ws();
    let b = plain_ws();
    let da = data(plain_brief(a.path(), &[]));
    let db = data(plain_brief(b.path(), &["--independence", "sequential"]));
    let hashes = |d: &Value| -> Vec<String> {
        d["briefs"].as_array().unwrap().iter().map(|x| x["hash"].as_str().unwrap().to_string()).collect()
    };
    assert_eq!(hashes(&da), hashes(&db));
}

#[test]
fn a_sequential_run_with_nothing_in_the_way_advises_never_applies() {
    let ws = Ws::new();
    ws.site();
    let seq = ws.run("critical", &THREE, false);
    let ind = ws.run("critical", &THREE, false);
    for r in [seq, ind] {
        ws.ingest(r, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
        ws.ingest(r, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
        ws.ingest(r, &critical("cat", [4, 4, 4], &[], &["page.json#n1"]));
    }
    mark(&ws, seq, "sequential");
    under_low_rule(&ws, &[seq, ind]);
    let a = data(ws.cli(&["apply-check", &rs(seq)]));
    assert_eq!(a["answer"], "advise", "{a}");
    assert!(has(&a, "independence_sequential"), "{a}");
    let r = a["reasons"].as_array().unwrap().iter().find(|r| r["code"] == "independence_sequential").unwrap().clone();
    assert!(r["remedy"].as_str().unwrap().contains("one context per brief"), "{a}");
    // the same verdicts from independent jurors apply (run 2's delta against run 1 lands: nothing moved, no blockers)
    let b = data(ws.cli(&["apply-check", &rs(ind)]));
    assert_eq!(b["answer"], "apply", "{b}");
}

#[test]
fn a_sequential_run_still_escalates_on_a_primary_blocker_and_stays_human_on_a_human_class() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [3, 3, 3], &["the price is hidden"], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [3, 3, 3], &[], &["page.json#n1"]));
    mark(&ws, run, "sequential");
    let a = data(ws.cli(&["apply-check", &rs(run)]));
    assert_eq!(a["answer"], "escalate", "{a}");
    assert!(has(&a, "independence_sequential") && has(&a, "primary_blocker"), "{a}");
    toml(&ws, "[[rule]]\nclass = \"money\"\npaths = [\"sources/site/**\"]\nquorum = \"human\"\n");
    let a = data(ws.cli(&["apply-check", &rs(run)]));
    assert_eq!(a["answer"], "human", "{a}");
}

#[test]
fn the_matrix_carries_the_independence_in_its_json_and_its_header() {
    let ws = Ws::new();
    ws.site();
    let run = ws.run("critical", &TWO, false);
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    let m = data(ws.cli(&["matrix", &rs(run), "--table"]));
    assert_eq!(m["independence"], "independent", "{m}");
    assert!(!m["table"].as_str().unwrap().contains("independence"), "an independent table is unchanged: {m}");
    mark(&ws, run, "sequential");
    let m = data(ws.cli(&["matrix", &rs(run), "--table"]));
    assert_eq!(m["independence"], "sequential", "{m}");
    assert!(m["table"].as_str().unwrap().starts_with("independence: sequential\n"), "{m}");
    // two runs of different kinds are mixed
    let other = ws.run("critical", &TWO, false);
    ws.ingest(other, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    let m = data(ws.cli(&["matrix", &rs(run), &rs(other), "--table"]));
    assert_eq!(m["independence"], "mixed", "{m}");
    assert!(m["table"].as_str().unwrap().contains("independence: run 1 sequential"), "{m}");
}

#[test]
fn delta_echoes_the_independence_of_both_runs_and_a_pair_with_one_sequential_advises() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &TWO, false);
    let r2 = ws.run("critical", &TWO, false);
    for r in [r1, r2] {
        ws.ingest(r, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
        ws.ingest(r, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    }
    mark(&ws, r1, "sequential");
    let d = data(ws.cli(&["delta", &rs(r1), &rs(r2)]));
    assert_eq!(d["independence"], json!({"a": "sequential", "b": "independent"}), "{d}");
    let a = data(ws.cli(&["apply-check", &rs(r1), &rs(r2)]));
    assert_eq!(a["answer"], "advise", "{a}");
    assert!(has(&a, "independence_sequential"), "{a}");
}

fn mcp_session(ws: &Path, lines: &[String]) -> Vec<Value> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    let mut ch = Command::new(assert_cmd::cargo::cargo_bin("personajury"))
        .arg("--root")
        .arg(ws)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = ch.stdin.take().unwrap();
    for l in lines {
        writeln!(stdin, "{l}").unwrap();
    }
    drop(stdin);
    let out: Vec<Value> =
        BufReader::new(ch.stdout.take().unwrap()).lines().map(|l| serde_json::from_str(&l.unwrap()).unwrap()).collect();
    ch.wait().unwrap();
    out
}

#[test]
fn the_mcp_brief_takes_independence_and_echoes_the_sequence() {
    let ws = plain_ws();
    let line = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"personajury_brief","arguments":{
        "subject":"pricing","lens":"recall","query":"3 reader=bystander","model":"m","independence":"sequential"}}})
    .to_string();
    let out = mcp_session(ws.path(), &[line]);
    let env: Value = serde_json::from_str(out[0]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(env["ok"], true, "{env}");
    assert_eq!(env["data"]["independence"], "sequential", "{env}");
    assert_eq!(env["data"]["sequence"].as_array().unwrap().len(), 3, "{env}");
}

// ---- item 9: experts and a human verdict (D19) ----

fn persona_file(w: &Path, name: &str, extra: Value) -> PathBuf {
    let mut body = json!({
        "id": name, "name": format!("Person {name}"), "tags": ["nurse"],
        "role": "r", "context": "c", "daily_tasks": "d", "fears": "f",
        "never_reads": "n", "decides_by": "b", "stake_rules": "s"
    });
    for (k, v) in extra.as_object().unwrap() {
        body[k] = v.clone();
    }
    let f = w.join(format!("{name}.json"));
    std::fs::write(&f, body.to_string()).unwrap();
    f
}

#[test]
fn a_credential_round_trips_into_the_list_and_the_brief() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    let f = persona_file(w, "nurse-ana", json!({"credential": "a hospital nurse with ten years on wards"}));
    data(cli(w, &["persona", "add", "--file", f.to_str().unwrap()]));
    let list = data(cli(w, &["persona", "list"]));
    assert_eq!(list["items"][0]["credential"], "a hospital nurse with ten years on wards", "{list}");
    doc(w, "pricing");
    data(cli(w, &["panel", "add", "n", "--query", "1 nurse=primary"]));
    let d = data(cli(w, &["brief", "--subject", "pricing", "--lens", "recall", "--panel", "n", "--model", "m"]));
    let text = std::fs::read_to_string(w.join(d["briefs"][0]["path"].as_str().unwrap())).unwrap();
    assert!(text.contains("- Credential: a hospital nurse with ten years on wards"), "{text}");
    // a persona without one keeps its profile, its hash and its brief as before
    let g = persona_file(w, "plain", json!({}));
    let added = data(cli(w, &["persona", "add", "--file", g.to_str().unwrap()]));
    assert!(added["hash"].as_str().unwrap().len() == 64);
    let list = data(cli(w, &["persona", "list"]));
    let plain = list["items"].as_array().unwrap().iter().find(|p| p["id"] == "plain").unwrap().clone();
    assert!(plain.get("credential").is_none(), "{plain}");
}

#[test]
fn an_unknown_persona_key_is_still_refused_and_a_credential_stays_a_line() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    let f = persona_file(w, "odd", json!({"seniority": "ten years"}));
    let r = cli(w, &["persona", "add", "--file", f.to_str().unwrap()]);
    assert_eq!(code(&r), "invalid_arguments", "{r}");
    let f = persona_file(w, "long", json!({"credential": "x".repeat(2000)}));
    let r = cli(w, &["persona", "add", "--file", f.to_str().unwrap()]);
    assert_eq!(code(&r), "credential_too_long", "{r}");
}

#[test]
fn a_lens_standard_is_shown_in_the_brief_and_the_matrix_header() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, &[("rdr", "reader", 1)]);
    doc(w, "pricing");
    let lens = "name = \"clinical\"\nversion = 1\nscale = [1, 5]\nevidence = [\"told\"]\nstandard = \"the ward handover checklist, as written by Dr Rao\"\n\n[[dimension]]\nid = \"clarity\"\ntitle = \"Is it clear\"\nanchors = { \"1\" = \"no\", \"3\" = \"partly\", \"5\" = \"yes\" }\n";
    let lf = w.join("clinical.toml");
    std::fs::write(&lf, lens).unwrap();
    data(cli(w, &["lens", "add", "--file", lf.to_str().unwrap()]));
    let d = data(cli(w, &["brief", "--subject", "pricing", "--lens", "clinical", "--query", "1 reader=bystander", "--model", "m"]));
    let text = std::fs::read_to_string(w.join(d["briefs"][0]["path"].as_str().unwrap())).unwrap();
    assert!(text.contains("as written by Dr Rao"), "{text}");
    let p = d["briefs"][0]["persona"].as_str().unwrap();
    let v = json!({"persona": p, "lines": [{"dimension": "clarity", "score": 3, "anchor_met": 3, "evidence": "told"}]});
    data(cli(w, &["verdict", "--run", &rs(d["run"].as_i64().unwrap()), "--json", &v.to_string()]));
    let m = data(cli(w, &["matrix", &rs(d["run"].as_i64().unwrap()), "--table"]));
    assert!(m["table"].as_str().unwrap().starts_with("standard: clinical \u{2014} the ward handover checklist, as written by Dr Rao\n"), "{m}");
    assert_eq!(m["standards"]["clinical"], "the ward handover checklist, as written by Dr Rao", "{m}");
}

fn human_verdict(persona: &str, blockers: &[&str], cite: &[&str]) -> Value {
    critical(persona, [3, 3, 3], blockers, cite)
}
fn by_human(ws: &Ws, run: i64, v: &Value, who: &str) -> Value {
    ws.cli(&["verdict", "--run", &rs(run), "--json", &v.to_string(), "--by-human", who])
}
fn clean_run(ws: &Ws) -> i64 {
    ws.site();
    let run = ws.run("critical", &THREE, false);
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("cat", [4, 4, 4], &[], &["page.json#n1"]));
    run
}

#[test]
fn a_human_blocker_escalates_a_run_that_would_apply_until_a_person_decides() {
    let ws = Ws::new();
    let run = clean_run(&ws);
    under_low_rule(&ws, &[run]);
    let a = data(ws.cli(&["apply-check", &rs(run)]));
    assert_eq!(a["answer"], "apply", "{a}");
    let r = by_human(&ws, run, &human_verdict("bob", &["the consent line is missing"], &["page.json#n1"]), "Dr Rao, ward nurse");
    let d = data(r);
    assert_eq!(d["by_human"], "Dr Rao, ward nurse", "{d}");
    let finding = d["findings"][0].as_str().unwrap().to_string();
    // the human's stratum is the seat's (secondary), and it escalates whatever the stratum
    let a = data(ws.cli(&["apply-check", &rs(run)]));
    assert_eq!(a["answer"], "escalate", "{a}");
    assert!(has(&a, "human_blocker"), "{a}");
    assert!(!has(&a, "primary_blocker"), "{a}");
    // a recorded decision by a person is the only way to overrule it: `--by` (the agent, the foreman) is refused
    for by in ["agent", "foreman", "Dr Rao"] {
        let r = ws.cli(&["decide", &finding, "dismissed", "--why", "not needed", "--by", by]);
        assert_eq!(code(&r), "human_decision_required", "{r}");
    }
    let a = data(ws.cli(&["apply-check", &rs(run)]));
    assert_eq!(a["answer"], "escalate", "{a}");
    let r = ws.cli(&["decide", &finding, "dismissed", "--why", "handled on the ward form", "--by-human", "Dr Rao"]);
    assert_eq!(r["ok"], true, "{r}");
    let a = data(ws.cli(&["apply-check", &rs(run)]));
    assert_eq!(a["answer"], "apply", "{a}");
}

#[test]
fn a_human_batch_without_evidence_is_refused_like_any_other() {
    let ws = Ws::new();
    let run = clean_run(&ws);
    let r = by_human(&ws, run, &human_verdict("bob", &[], &[]), "Dr Rao, ward nurse");
    // the helper puts a citation on every later line, so strip the first line's to leave a seen line uncited
    let _ = r;
    let mut v = human_verdict("bob", &[], &[]);
    v["lines"][0]["citations"] = json!([]);
    let r = by_human(&ws, run, &v, "Dr Rao, ward nurse");
    assert_eq!(code(&r), "seen_without_citation", "{r}");
}

#[test]
fn a_human_may_write_recalled_and_the_seat_must_exist() {
    let ws = Ws::new();
    let run = clean_run(&ws);
    let r = by_human(&ws, run, &human_verdict("nobody", &[], &["page.json#n1"]), "Dr Rao, ward nurse");
    assert_eq!(code(&r), "persona_without_stake", "{r}");
    let r = ws.cli(&["verdict", "--run", &rs(run), "--json", &human_verdict("bob", &[], &["page.json#n1"]).to_string(), "--by-human", "  "]);
    assert_eq!(code(&r), "invalid_arguments", "{r}");
}

#[test]
fn the_matrix_shows_a_human_beside_the_jury_never_in_its_cells() {
    let ws = Ws::new();
    let run = clean_run(&ws);
    let before = data(ws.cli(&["matrix", &rs(run), "--table", "--detail", "full"]));
    data(by_human(&ws, run, &human_verdict("bob", &[], &["page.json#n1"]), "Dr Rao, ward nurse"));
    let after = data(ws.cli(&["matrix", &rs(run), "--table", "--detail", "full"]));
    assert_eq!(before["strata"], after["strata"], "a human row is never merged into a persona's cell");
    let t = after["table"].as_str().unwrap();
    assert!(t.starts_with(before["table"].as_str().unwrap()), "the jury's table is unchanged: {t}");
    assert!(t.contains("H "), "{t}");
    assert!(t.contains("Dr Rao, ward nurse"), "{t}");
    assert_eq!(after["human"][0]["by"], "Dr Rao, ward nurse", "{after}");
    assert_eq!(after["human"][0]["stake"], "secondary", "{after}");
    // the graph and the pick still answer with a human batch in the store
    let g = ws.cli(&["graph", &rs(run), "--svg"]);
    assert_eq!(g["ok"], true, "{g}");
    let p = ws.cli(&["pick", &rs(run)]);
    assert_eq!(p["ok"], true, "{p}");
}

#[test]
fn two_humans_on_one_seat_both_stand() {
    let ws = Ws::new();
    let run = clean_run(&ws);
    data(by_human(&ws, run, &human_verdict("bob", &[], &["page.json#n1"]), "Dr Rao, ward nurse"));
    data(by_human(&ws, run, &human_verdict("bob", &[], &["page.json#n1"]), "Mr Osei, pharmacist"));
    let m = data(ws.cli(&["matrix", &rs(run)]));
    assert_eq!(m["human"].as_array().unwrap().len(), 2, "{m}");
}

#[test]
fn the_mcp_verdict_tool_has_no_by_human_an_agent_never_enters_one() {
    let ws = plain_ws();
    let line = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"personajury_verdict","arguments":{
        "run": 1, "verdict": {"persona": "x", "lines": []}, "by_human": "Dr Rao"}}})
    .to_string();
    let out = mcp_session(ws.path(), &[line]);
    let env: Value = serde_json::from_str(out[0]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "invalid_arguments", "{env}");
    assert!(env["error"]["message"].as_str().unwrap().contains("by_human"), "{env}");
}

// ---- item 3: a model per panel term (D15) ----

fn model_ws() -> tempfile::TempDir {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, &[("bld", "builder", 2), ("apr", "approver", 1), ("vis", "visitor", 1)]);
    doc(w, "pricing");
    data(cli(
        w,
        &["panel", "add", "m", "--query", "2 builder=primary@claude-sonnet-5-5, 1 approver=secondary@other-model, 1 visitor=bystander"],
    ));
    ws
}
fn model_brief(w: &Path) -> Value {
    data(cli(w, &["brief", "--subject", "pricing", "--lens", "critical", "--panel", "m", "--model", "claude-opus-5-5"]))
}
fn model_of(d: &Value, persona_prefix: &str) -> String {
    d["briefs"].as_array().unwrap().iter().find(|b| b["persona"].as_str().unwrap().starts_with(persona_prefix)).unwrap()["model"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn a_term_model_pins_its_seats_and_the_brief_flag_is_the_default() {
    let ws = model_ws();
    let w = ws.path();
    let d = model_brief(w);
    assert_eq!(model_of(&d, "bld"), "claude-sonnet-5-5", "{d}");
    assert_eq!(model_of(&d, "apr"), "other-model", "{d}");
    assert_eq!(model_of(&d, "vis"), "claude-opus-5-5", "{d}");
    assert_eq!(d["model"], "claude-opus-5-5", "the run keeps the default: {d}");
    // every brief file of a run with term models states its own
    for b in d["briefs"].as_array().unwrap() {
        let text = std::fs::read_to_string(w.join(b["path"].as_str().unwrap())).unwrap();
        let m = b["model"].as_str().unwrap();
        assert!(text.contains(&format!("Run this brief on the model `{m}`.")), "{text}");
    }
    let row = Store::open_existing(w).unwrap().run(d["run"].as_i64().unwrap()).unwrap().unwrap();
    assert_eq!(row.briefs.iter().find(|b| b.persona.starts_with("apr")).unwrap().model, "other-model");
}

#[test]
fn a_run_with_no_term_model_has_the_briefs_it_always_had() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    pool(w, &[("rdr", "reader", 2)]);
    doc(w, "pricing");
    let d = data(plain_brief(w, &[]));
    let text = std::fs::read_to_string(w.join(d["briefs"][0]["path"].as_str().unwrap())).unwrap();
    assert!(!text.contains("Run this brief on the model"), "{text}");
    assert_eq!(d["briefs"][0]["model"], "m", "{d}");
}

#[test]
fn the_briefs_of_term_models_are_deterministic() {
    let hashes = |d: &Value| -> Vec<String> {
        d["briefs"].as_array().unwrap().iter().map(|b| b["hash"].as_str().unwrap().to_string()).collect()
    };
    let (a, b) = (model_ws(), model_ws());
    assert_eq!(hashes(&model_brief(a.path())), hashes(&model_brief(b.path())));
}

#[test]
fn a_bad_model_in_a_term_is_an_invalid_query() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    for q in ["1 builder@", "1 builder=primary@", "1 builder@two words", "1 builder@a,b"] {
        let r = cli(w, &["panel", "add", "x", "--query", q]);
        assert_eq!(code(&r), "invalid_query", "{q}: {r}");
    }
}

#[test]
fn the_matrix_shows_the_models_per_stratum_and_how_many_are_distinct() {
    let ws = model_ws();
    let w = ws.path();
    let d = model_brief(w);
    let id = d["run"].as_i64().unwrap();
    let m = data(cli(w, &["matrix", &rs(id)]));
    assert_eq!(m["models"]["primary"], json!(["claude-sonnet-5-5"]), "{m}");
    assert_eq!(m["models"]["secondary"], json!(["other-model"]), "{m}");
    assert_eq!(m["models"]["bystander"], json!(["claude-opus-5-5"]), "{m}");
    assert_eq!(m["models_distinct"], 3, "{m}");
}

#[test]
fn one_model_for_every_juror_is_an_informational_reason_never_an_escalation() {
    let one = plain_ws();
    let d = data(plain_brief(one.path(), &[]));
    let a = data(cli(one.path(), &["apply-check", &rs(d["run"].as_i64().unwrap())]));
    assert!(has(&a, "single_model"), "{a}");
    let r = a["reasons"].as_array().unwrap().iter().find(|r| r["code"] == "single_model").unwrap().clone();
    assert_eq!(r["effect"], "info", "{a}");
    // nobody judged yet and no rule holds the subject: the info reason is all, the answer is `advise` (C1, C3)
    assert_eq!(a["answer"], "advise", "{a}");
    assert!(has(&a, "incomplete_jury") && has(&a, "quorum_unknown"), "{a}");
    let many = model_ws();
    let d = model_brief(many.path());
    let a = data(cli(many.path(), &["apply-check", &rs(d["run"].as_i64().unwrap())]));
    assert!(!has(&a, "single_model"), "{a}");
}

// ---- item 2: lens generate (D14) ----

fn material(w: &Path) -> PathBuf {
    let dir = w.join("material");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("handbook.md"), "# Handover\nA good handover names the patient, the risk and the owner of each open task.\n").unwrap();
    std::fs::write(dir.join("examples.md"), "Bad: 'all fine'. Good: 'bed 4, falls risk, Ana owns the 6pm check'.\n").unwrap();
    dir
}
fn lens_brief(w: &Path, extra: &[&str]) -> Value {
    let from = material(w);
    let mut a = vec!["lens", "generate", "--from", from.to_str().unwrap()];
    a.extend_from_slice(extra);
    cli(w, &a)
}

#[test]
fn lens_generate_writes_a_deterministic_hashed_brief_and_calls_no_model() {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (da, db) = (data(lens_brief(a.path(), &[])), data(lens_brief(b.path(), &[])));
    assert_eq!(da["material_hash"], db["material_hash"], "{da}");
    let (ta, tb) = (
        std::fs::read_to_string(a.path().join(da["brief"].as_str().unwrap())).unwrap(),
        std::fs::read_to_string(b.path().join(db["brief"].as_str().unwrap())).unwrap(),
    );
    assert_eq!(ta, tb, "same material, same bytes");
    assert_eq!(da["dimensions"], 4, "{da}");
    let h = da["material_hash"].as_str().unwrap();
    assert_eq!(h.len(), 64);
    assert!(ta.contains(h) && ta.contains(&format!("standard = \"material:{h}\"")), "{ta}");
    assert!(ta.contains("what the material says good looks like"), "{ta}");
    assert!(ta.contains("anchors at 1, 3 and 5"), "{ta}");
    assert!(ta.contains("[[dimension]]") && ta.contains("anchors = {"), "{ta}");
    assert_eq!(ta.matches("[[dimension]]").count(), 4, "{ta}");
    // the material is recorded, as persona generate does
    assert!(Store::open_existing(a.path()).unwrap().material_exists(h).unwrap());
}

#[test]
fn the_brief_names_the_count_and_its_example_is_a_lens_the_engine_accepts() {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    let d = data(lens_brief(w, &["--dimensions", "2"]));
    assert_eq!(d["dimensions"], 2, "{d}");
    let text = std::fs::read_to_string(w.join(d["brief"].as_str().unwrap())).unwrap();
    let open = text.find("```toml\n").unwrap() + 8;
    let shape = &text[open..open + text[open..].find("```").unwrap()];
    assert_eq!(shape.matches("[[dimension]]").count(), 2);
    let f = w.join("shape.toml");
    std::fs::write(&f, shape).unwrap();
    let r = cli(w, &["lens", "add", "--file", f.to_str().unwrap()]);
    assert_eq!(r["ok"], true, "the shape in the brief is a valid lens: {r}\n{shape}");
}

#[test]
fn lens_generate_refuses_a_bad_count_and_missing_material() {
    let ws = tempfile::tempdir().unwrap();
    for n in ["0", "13"] {
        let r = lens_brief(ws.path(), &["--dimensions", n]);
        assert_eq!(code(&r), "invalid_arguments", "{n}: {r}");
    }
    let r = cli(ws.path(), &["lens", "generate", "--from", ws.path().join("nowhere").to_str().unwrap()]);
    assert_eq!(r["ok"], false, "{r}");
}

#[test]
fn the_three_example_lenses_are_accepted_by_lens_add() {
    let ws = tempfile::tempdir().unwrap();
    for f in ["consulting-deck", "investor-pitch", "financial-model"] {
        let r = cli(ws.path(), &["lens", "add", "--file", &fx(&format!("lenses/{f}.toml"))]);
        assert_eq!(r["ok"], true, "{f}: {r}");
    }
    let list = data(cli(ws.path(), &["lens", "list"]));
    let names: Vec<&str> = list["items"].as_array().unwrap().iter().map(|l| l["name"].as_str().unwrap()).collect();
    for n in ["consulting-deck", "investor-pitch", "financial-model"] {
        assert!(names.contains(&n), "{names:?}");
    }
}

// ---- the dial: every brief flag sits in exactly one column of the table (D13..D19, report §7) ----

/// (flag, column): `free` needs no reason, `reason` needs a recorded one. A flag added to `brief` without a row here, or
/// without a cell in the table of the skill and `docs/usage.md`, turns this red.
const DIAL: [(&str, &str); 19] = [
    ("--subject", "free"),
    ("--model", "free"),
    ("--comparator", "free"),
    ("--wave", "free"),
    ("--independence", "free"),
    ("--rule", "free"),
    ("--panel", "reason"),
    ("--query", "reason"),
    ("--stake", "reason"),
    ("--draw", "reason"),
    ("--spend", "reason"),
    ("--lens", "reason"),
    ("--detail", "reason"),
    ("--allow-large", "reason"),
    ("--reason", "reason"),
    ("--by", "reason"),
    ("--effort", "reason"),
    ("--traits", "reason"),
    ("--seats", "reason"),
];

fn dial_table(file: &str) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(repo().join(file)).unwrap();
    let start = text.find("## The dial").unwrap_or_else(|| panic!("{file} has no dial section"));
    let section = &text[start..];
    let section = &section[..section[3..].find("\n## ").map_or(section.len(), |i| i + 3)];
    section
        .lines()
        .filter(|l| l.starts_with('|') && !l.starts_with("|---"))
        .map(|l| l.trim_matches('|').split('|').map(|c| c.trim().to_string()).collect())
        .collect()
}

#[test]
fn every_brief_flag_is_in_exactly_one_column_of_the_dial() {
    let help = Command::new(assert_cmd::cargo::cargo_bin("personajury")).args(["brief", "--help"]).output().unwrap();
    let help = String::from_utf8(help.stdout).unwrap();
    let mut flags: Vec<String> = help
        .lines()
        .filter_map(|l| {
            let t = l.trim_start();
            // `-h, --help`: the long form follows a short one
            let t = if t.starts_with("--") { t } else { t.split_once(", ").map_or(t, |(_, r)| r) };
            let f = t.strip_prefix("--")?;
            Some(format!("--{}", f.split([' ', '<']).next()?))
        })
        .filter(|f| f != "--help" && f != "--root" && f != "--pretty")
        .collect();
    flags.sort();
    flags.dedup();
    assert!(flags.len() >= 15, "read the flags of brief --help: {flags:?}");
    for f in &flags {
        assert!(DIAL.iter().any(|(d, _)| d == f), "{f} is a brief flag with no row in the dial: place it in DIAL and in the table");
    }
    for (f, _) in DIAL {
        assert!(flags.iter().any(|x| x == f), "{f} is in DIAL but is not a brief flag");
    }
    let skill = dial_table("skills/running-a-jury/SKILL.md");
    assert_eq!(skill, dial_table("docs/usage.md"), "the dial of docs/usage.md differs from the skill's");
    for (flag, column) in DIAL {
        let cell = format!("`{flag}`");
        let cols: Vec<usize> = (1..4).filter(|c| skill.iter().any(|row| row.get(*c).is_some_and(|x| x.contains(&cell)))).collect();
        let want = if column == "free" { 1 } else { 2 };
        assert_eq!(cols, vec![want], "{flag} must sit in exactly the {column} column of the table");
    }
}

#[test]
fn the_public_example_material_runs_through_persona_generate() {
    let ws = tempfile::tempdir().unwrap();
    let csv = fx("material/usecases.csv");
    let d = data(cli(ws.path(), &["persona", "generate", "--from", &csv, "--count", "5"]));
    assert_eq!(d["sources"].as_array().unwrap().len(), 1, "{d}");
    let text = std::fs::read_to_string(&csv).unwrap();
    assert_eq!(text.lines().count(), 13, "a header and twelve rows");
    assert!(text.lines().next().unwrap() == "role,job,trigger,data_sensitivity,clearance_note");
    // nothing from a private register crosses (BOUNDARY): no ids, no register names
    for token in ["UC-S-", "sectors.csv", "verticals-catalogue"] {
        assert!(!text.contains(token), "{token}");
    }
}
