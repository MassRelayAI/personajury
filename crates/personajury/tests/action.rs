//! v0.5 (D22): the GitHub Action. `action.yml` is a composite action whose steps call `scripts/action-run.sh`; the script
//! runs against a local binary here (no download, no network). serde_yaml is not a dependency, so the YAML is checked line
//! by line for the keys that must be there. Every test was seen red first.
mod judging_support;
use judging_support::*;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    repo().canonicalize().unwrap()
}

#[test]
fn action_yml_declares_the_inputs_the_outputs_and_composite_steps() {
    let y = std::fs::read_to_string(root().join("action.yml")).expect("action.yml at the repo root");
    for key in [
        "name:", "description:", "inputs:", "  config:", "    default: jurygraph.toml", "  base:", "  store:", "  version:", "  fail-on:",
        "outputs:", "  quorum:", "  spend:", "  effort:", "  answer:", "  check:", "runs:", "  using: composite", "  steps:",
        "scripts/install.sh", "scripts/action-run.sh", "shell: sh",
    ] {
        assert!(y.lines().any(|l| l.trim_end() == key || l.contains(key)), "action.yml lacks `{key}`");
    }
    // no model, no secret, no token: nothing but the binary download
    for bad in ["secrets.", "ANTHROPIC", "OPENAI", "API_KEY", "TOKEN", "actions/checkout@v"] {
        assert!(!y.contains(bad), "action.yml mentions {bad}");
    }
    // every output is wired to a step
    for o in ["quorum", "spend", "effort", "answer", "check"] {
        assert!(y.contains(&format!("steps.run.outputs.{o}")), "output {o} is not read from the run step");
    }
}

/// A git repo with the fixture rules on its first commit and a change to `files` on the second; returns its path.
fn repo_with_change(files: &[&str]) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let git = |args: &[&str]| {
        let o = Command::new("git").current_dir(p).args(args).env("GIT_AUTHOR_NAME", "t").env("GIT_AUTHOR_EMAIL", "t@t").env("GIT_COMMITTER_NAME", "t").env("GIT_COMMITTER_EMAIL", "t@t").output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    };
    git(&["init", "-q", "-b", "main"]);
    std::fs::copy(root().join("fixtures/jurygraph.toml"), p.join("jurygraph.toml")).unwrap();
    git(&["add", "jurygraph.toml"]);
    git(&["commit", "-q", "-m", "base"]);
    git(&["branch", "base"]);
    for f in files {
        std::fs::create_dir_all(p.join(f).parent().unwrap()).unwrap();
        std::fs::write(p.join(f), "changed\n").unwrap();
    }
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "change"]);
    d
}
struct Out {
    code: i32,
    outputs: String,
    summary: String,
    stderr: String,
}
fn action(cwd: &Path, store: &str, fail_on: &str) -> Out {
    let tmp = tempfile::tempdir().unwrap();
    let (o, s) = (tmp.path().join("out"), tmp.path().join("summary"));
    let r = Command::new("sh")
        .arg(root().join("scripts/action-run.sh"))
        .current_dir(cwd)
        .env("JG_BIN", assert_cmd::cargo::cargo_bin("jurygraph"))
        .env("JG_CONFIG", "jurygraph.toml")
        .env("JG_BASE", "base")
        .env("JG_STORE", store)
        .env("JG_FAIL_ON", fail_on)
        .env("RUNNER_TEMP", tmp.path())
        .env("GITHUB_OUTPUT", &o)
        .env("GITHUB_STEP_SUMMARY", &s)
        .output()
        .unwrap();
    Out {
        code: r.status.code().unwrap_or(-1),
        outputs: std::fs::read_to_string(&o).unwrap_or_default(),
        summary: std::fs::read_to_string(&s).unwrap_or_default(),
        stderr: String::from_utf8_lossy(&r.stderr).to_string(),
    }
}
fn output<'a>(o: &'a Out, k: &str) -> &'a str {
    o.outputs.lines().find_map(|l| l.strip_prefix(&format!("{k}="))).unwrap_or_else(|| panic!("no output {k} in:\n{}\n{}", o.outputs, o.stderr))
}

#[test]
fn the_script_classifies_the_diff_and_says_which_files_got_no_jury() {
    let d = repo_with_change(&["site/pricing.html", "README.md", "notes/todo.txt"]);
    let o = action(d.path(), "", "human");
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(output(&o, "quorum"), "high");
    assert_eq!(output(&o, "effort"), "medium");
    assert!(output(&o, "spend").contains("public-surface"), "{}", o.outputs);
    assert!(o.summary.contains("| Quorum | high |"), "{}", o.summary);
    assert!(
        o.summary.contains("2 files matched no rule and got no jury: README.md, notes/todo.txt; add a rule if one of them is something people see"),
        "{}",
        o.summary
    );
    // no store: the answer and the check are not run
    assert_eq!(output(&o, "answer"), "none");
    assert_eq!(output(&o, "check"), "skipped");
}

#[test]
fn a_change_under_a_human_class_fails_the_job_when_fail_on_names_it() {
    let d = repo_with_change(&["deploy/prod.yml"]);
    let o = action(d.path(), "", "human");
    assert_eq!(output(&o, "quorum"), "human");
    assert_eq!(o.code, 1, "{}", o.stderr);
    assert!(o.summary.contains("A person decides"), "{}", o.summary);
    let o = action(d.path(), "", "escalate");
    assert_eq!(o.code, 0, "human is not in fail-on: {}", o.stderr);
}

#[test]
fn a_change_the_rules_do_not_cover_needs_no_jury() {
    let d = repo_with_change(&["crates/x.rs"]);
    let o = action(d.path(), "", "human,escalate");
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(output(&o, "quorum"), "none");
    assert!(o.summary.contains("No jury needed"), "{}", o.summary);
}

#[test]
fn with_a_store_check_and_apply_check_run_and_a_failing_check_fails_the_job() {
    let d = repo_with_change(&["site/pricing.html"]);
    let ws = tempfile::tempdir().unwrap();
    fixture_run(ws.path());
    // the store's root holds the project's rules: the Action passes should-run's one rule to apply-check (C1)
    std::fs::copy(root().join("fixtures/jurygraph.toml"), ws.path().join("jurygraph.toml")).unwrap();
    let o = action(d.path(), ws.path().to_str().unwrap(), "human");
    assert_eq!(output(&o, "check"), "failed", "{}", o.outputs);
    assert_eq!(output(&o, "answer"), "escalate", "{}", o.outputs);
    assert_eq!(o.code, 1, "the fixture has findings with no decision: {}", o.stderr);
    assert!(o.summary.contains("escalate"), "{}", o.summary);
}

#[test]
fn escalate_fails_only_when_fail_on_names_it() {
    let d = repo_with_change(&["site/pricing.html"]);
    let ws = Ws::new();
    ws.site();
    let r = ws.run("critical", &[("ana", jurygraph::domain::Stake::Primary), ("bob", jurygraph::domain::Stake::Secondary)], false);
    let v = critical("ana", [3, 3, 3], &["the price is hidden"], &["page.json#n1"]);
    let finding = ws.ingest(r, &v)["findings"][0].as_str().unwrap().to_string();
    ws.ingest(r, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    assert_eq!(ws.cli(&["decide", &finding, "dismissed", "--why", "intended", "--by", "ana"])["ok"], json!(true));
    std::fs::copy(root().join("fixtures/jurygraph.toml"), ws.root().join("jurygraph.toml")).unwrap();
    let store = ws.root().to_str().unwrap().to_string();
    let o = action(d.path(), &store, "human");
    assert_eq!(output(&o, "check"), "ok", "{}", o.outputs);
    assert_eq!(output(&o, "answer"), "escalate", "{}", o.outputs);
    assert_eq!(o.code, 0, "{}", o.stderr);
    let o = action(d.path(), &store, "human,escalate");
    assert_eq!(o.code, 1, "{}", o.stderr);
}

#[test]
fn the_script_defaults_the_base_to_the_pull_request_base() {
    let s = std::fs::read_to_string(root().join("scripts/action-run.sh")).unwrap();
    assert!(s.contains("GITHUB_BASE_REF"), "the default base reads the PR base");
    assert!(!s.contains("curl") && !s.contains("wget"), "the script itself fetches nothing");
}

#[test]
fn ci_docs_carry_a_workflow_example() {
    let d = std::fs::read_to_string(root().join("docs/ci.md")).expect("docs/ci.md");
    assert!(d.contains("uses: massrelay-ai/jurygraph@") && d.contains("fail-on:"), "{d}");
}
