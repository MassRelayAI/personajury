//! The v0.5 review's findings (briefs/v05-review.md, probes p1..p7), ported as tests. Every test was seen red first.
//! D13: `apply` only when every condition is proven; gates fail closed.
mod judging_support;
use judging_support::*;
use personajury::domain::{Stake, SubjectKind};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const PB: [(&str, Stake); 2] = [("ana", Stake::Primary), ("bob", Stake::Bystander)];
const RULE_ALL: &str = "[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n";
fn low(ws: &Ws, id: i64) {
    ws.store.conn().execute("UPDATE run SET rule_class = 'public-surface', quorum = 'low' WHERE id = ?1", [id]).unwrap();
}
fn code(v: &Value) -> String {
    v["error"]["code"].as_str().unwrap_or("").to_string()
}
fn data(v: Value) -> Value {
    assert_eq!(v["ok"], true, "{v}");
    v["data"].clone()
}
fn root() -> PathBuf {
    repo().canonicalize().unwrap()
}

/// `site` escalates (a primary blocker in run a); `other` is clean (run b, the newest).
fn two_subjects() -> (Ws, i64, i64) {
    let ws = Ws::new();
    std::fs::write(ws.root().join("personajury.toml"), RULE_ALL).unwrap();
    ws.site();
    let a = ws.run("critical", &PB, false);
    let f = ws.ingest(a, &critical("ana", [2, 2, 2], &["the button is broken"], &["page.json#n1"]))["findings"][0]
        .as_str()
        .unwrap()
        .to_string();
    ws.ingest(a, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    // the finding is decided, so only the primary blocker stands: `site` escalates on its own verdicts
    assert_eq!(ws.cli(&["decide", &f, "dismissed", "--why", "x", "--by", "ana"])["ok"], true);
    low(&ws, a);
    ws.subject("other", SubjectKind::Page, false, &[("page.json", br#"{"nodes":[{"id":"n1"}]}"#)]);
    let b = ws.run_of("other", "critical", &THREE, false);
    ws.ingest(b, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(b, &critical("cat", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(b, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    low(&ws, b);
    (ws, a, b)
}

// ---- C1: --latest answers the worst over every subject that has runs ----

#[test]
fn c1_latest_answers_the_worst_subject_not_the_newest_one() {
    let (ws, a, b) = two_subjects();
    assert_eq!(data(ws.cli(&["apply-check", &a.to_string()]))["answer"], "escalate");
    assert_eq!(data(ws.cli(&["apply-check", &b.to_string()]))["answer"], "apply");
    let l = data(ws.cli(&["apply-check", "--latest"]));
    assert_eq!(l["answer"], "escalate", "the worst over every subject: {l}");
    let subs = l["subjects"].as_array().expect("each subject's own answer");
    let find = |n: &str| subs.iter().find(|s| s["subject"] == n).unwrap_or_else(|| panic!("{n} in {l}")).clone();
    assert_eq!(find("site")["answer"], "escalate");
    assert_eq!(find("other")["answer"], "apply");
    assert!(find("site")["reasons"].as_array().unwrap().iter().any(|r| r["code"] == "primary_blocker"), "{l}");
    // the flat reasons name the subject they come from
    assert!(l["reasons"].as_array().unwrap().iter().any(|r| r["code"] == "primary_blocker" && r["subject"] == "site"), "{l}");
}

#[test]
fn c1_latest_with_a_subject_judges_only_that_subject() {
    let (ws, a, b) = two_subjects();
    let o = data(ws.cli(&["apply-check", "--latest", "--subject", "other"]));
    assert_eq!(o["answer"], "apply", "{o}");
    assert_eq!(o["runs"], json!([b]));
    let s = data(ws.cli(&["apply-check", "--latest", "--subject", "site"]));
    assert_eq!(s["answer"], "escalate", "{s}");
    assert_eq!(s["runs"], json!([a]));
    assert_eq!(code(&ws.cli(&["apply-check", "--latest", "--subject", "nope"])), "not_found");
    // --subject only narrows --latest
    assert_eq!(code(&ws.cli(&["apply-check", &a.to_string(), "--subject", "site"])), "invalid_arguments");
}

#[test]
fn c1_worst_order_is_human_escalate_advise_apply() {
    let (ws, _, _) = two_subjects();
    // a third subject with no rule recorded and whose sources fall under none: it can only advise
    ws.subject("third", SubjectKind::Page, false, &[("page.json", br#"{"nodes":[{"id":"n1"}]}"#)]);
    std::fs::write(
        ws.root().join("personajury.toml"),
        "[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/site/**\", \"sources/other/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n\
         [[rule]]\nclass = \"money\"\npaths = [\"sources/fourth/**\"]\nquorum = \"human\"\n",
    )
    .unwrap();
    let c = ws.run_of("third", "critical", &PB, false);
    ws.ingest(c, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(c, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    assert_eq!(data(ws.cli(&["apply-check", "--latest", "--subject", "third"]))["answer"], "advise");
    assert_eq!(data(ws.cli(&["apply-check", "--latest"]))["answer"], "escalate", "escalate beats advise");
    // a fourth subject under a human class: human beats everything
    ws.subject("fourth", SubjectKind::Page, false, &[("page.json", br#"{"nodes":[{"id":"n1"}]}"#)]);
    let d = ws.run_of("fourth", "critical", &PB, false);
    ws.ingest(d, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(d, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    assert_eq!(data(ws.cli(&["apply-check", "--latest"]))["answer"], "human");
}

// ---- I1, M4, M5: the Action fails closed, escapes file names, pins its version ----

fn git_repo(files: &[&str]) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let git = |args: &[&str]| {
        let o = Command::new("git").current_dir(p).args(args).env("GIT_AUTHOR_NAME", "t").env("GIT_AUTHOR_EMAIL", "t@t").env("GIT_COMMITTER_NAME", "t").env("GIT_COMMITTER_EMAIL", "t@t").output().unwrap();
        assert!(o.status.success(), "git {args:?}");
    };
    git(&["init", "-q", "-b", "main"]);
    std::fs::copy(root().join("fixtures/personajury.toml"), p.join("personajury.toml")).unwrap();
    git(&["add", "personajury.toml"]);
    git(&["commit", "-q", "-m", "base"]);
    git(&["branch", "base"]);
    for f in files {
        std::fs::create_dir_all(p.join(f).parent().unwrap()).unwrap();
        std::fs::write(p.join(f), "x\n").unwrap();
    }
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "c"]);
    d
}
struct Out {
    code: i32,
    outputs: String,
    summary: String,
    stderr: String,
}
/// Run the Action's script; `fail_on: None` leaves `JG_FAIL_ON` unset (the default).
fn act(cwd: &Path, bin: &Path, store: &str, fail_on: Option<&str>) -> Out {
    let tmp = tempfile::tempdir().unwrap();
    let (o, s) = (tmp.path().join("out"), tmp.path().join("summary"));
    let mut c = Command::new("sh");
    c.arg(root().join("scripts/action-run.sh"))
        .current_dir(cwd)
        .env("JG_BIN", bin)
        .env("JG_BASE", "base")
        .env("JG_STORE", store)
        .env("RUNNER_TEMP", tmp.path())
        .env("GITHUB_OUTPUT", &o)
        .env("GITHUB_STEP_SUMMARY", &s)
        .env_remove("JG_FAIL_ON");
    if let Some(f) = fail_on {
        c.env("JG_FAIL_ON", f);
    }
    let r = c.output().unwrap();
    Out {
        code: r.status.code().unwrap_or(-1),
        outputs: std::fs::read_to_string(&o).unwrap_or_default(),
        summary: std::fs::read_to_string(&s).unwrap_or_default(),
        stderr: String::from_utf8_lossy(&r.stderr).into(),
    }
}
fn output<'a>(o: &'a Out, k: &str) -> &'a str {
    o.outputs.lines().find_map(|l| l.strip_prefix(&format!("{k}="))).unwrap_or_else(|| panic!("no output {k}:\n{}\n{}", o.outputs, o.stderr))
}
fn bin() -> PathBuf {
    assert_cmd::cargo::cargo_bin("personajury")
}
/// A store whose one finding is dismissed (check ok) and whose root toml is broken (apply-check errors).
fn broken_toml_store() -> Ws {
    let ws = Ws::new();
    ws.site();
    let r = ws.run("critical", &PB, false);
    let f = ws.ingest(r, &critical("ana", [2, 2, 2], &["broken"], &["page.json#n1"]))["findings"][0].as_str().unwrap().to_string();
    ws.ingest(r, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    assert_eq!(ws.cli(&["decide", &f, "dismissed", "--why", "x", "--by", "ana"])["ok"], true);
    std::fs::write(ws.root().join("personajury.toml"), "this is = = not toml").unwrap();
    ws
}

#[test]
fn i1_an_apply_check_error_fails_the_job_by_default() {
    let ws = broken_toml_store();
    let d = git_repo(&["site/pricing.html"]);
    let st = ws.root().to_str().unwrap();
    let o = act(d.path(), &bin(), st, None);
    assert_eq!(output(&o, "answer"), "unavailable", "{}", o.stderr);
    assert_eq!(o.code, 1, "the default fail-on names unavailable: {}", o.stderr);
    let o = act(d.path(), &bin(), st, Some("human,escalate,unavailable"));
    assert_eq!(o.code, 1, "{}", o.stderr);
    // a fail-on that leaves `unavailable` out on purpose lets it through, and still says so
    let o = act(d.path(), &bin(), st, Some("human,escalate,advise"));
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(output(&o, "answer"), "unavailable");
    assert!(o.summary.contains("unavailable"), "{}", o.summary);
}

#[test]
fn i1_a_missing_binary_or_unparsable_json_fails_the_job() {
    let d = git_repo(&["site/pricing.html"]);
    let o = act(d.path(), Path::new("/nonexistent/personajury"), "", None);
    assert_eq!(o.code, 1, "{}", o.stderr);
    assert_eq!(output(&o, "answer"), "unavailable", "{}", o.outputs);
    // a binary that prints something that is not JSON
    let t = tempfile::tempdir().unwrap();
    let fake = t.path().join("jg");
    std::fs::write(&fake, "#!/bin/sh\necho 'not json at all'\n").unwrap();
    Command::new("chmod").arg("+x").arg(&fake).status().unwrap();
    let o = act(d.path(), &fake, "", None);
    assert_eq!(o.code, 1, "{}", o.stderr);
    assert_eq!(output(&o, "answer"), "unavailable", "{}", o.outputs);
    let o = act(d.path(), &fake, "", Some("human,escalate"));
    assert_eq!(o.code, 0, "unavailable left out on purpose: {}", o.stderr);
}

#[test]
fn i1_a_check_error_is_unavailable_not_ok() {
    // a store root whose store cannot be opened: check errors (not check_failed)
    let t = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(t.path().join(".personajury")).unwrap();
    std::fs::write(t.path().join(".personajury/store.db"), "not a database").unwrap();
    let d = git_repo(&["site/pricing.html"]);
    let o = act(d.path(), &bin(), t.path().to_str().unwrap(), None);
    assert_eq!(o.code, 1, "{}", o.stderr);
    assert_eq!(output(&o, "answer"), "unavailable", "{}", o.outputs);
    assert_eq!(output(&o, "check"), "unavailable", "an error is not a failing check: {}", o.outputs);
}

#[test]
fn c1_the_action_passes_the_rule_when_should_run_matched_exactly_one() {
    let (ws, _, _) = two_subjects();
    std::fs::copy(root().join("fixtures/personajury.toml"), ws.root().join("personajury.toml")).unwrap();
    let t = tempfile::tempdir().unwrap();
    let log = t.path().join("args.log");
    let wrap = t.path().join("jg");
    std::fs::write(&wrap, format!("#!/bin/sh\necho \"$@\" >> '{}'\nexec '{}' \"$@\"\n", log.display(), bin().display())).unwrap();
    Command::new("chmod").arg("+x").arg(&wrap).status().unwrap();
    let d = git_repo(&["site/pricing.html"]);
    let o = act(d.path(), &wrap, ws.root().to_str().unwrap(), Some("human"));
    let args = std::fs::read_to_string(&log).unwrap();
    assert!(args.lines().any(|l| l.contains("apply-check --latest --rule public-surface")), "{args}\n{}", o.stderr);
    assert!(!args.contains("--subject"), "the Action judges every subject: {args}");
    // the store escalates on `site` although `other` is newer and clean
    assert_eq!(output(&o, "answer"), "escalate", "{}", o.stderr);
    // two rules matched: no --rule
    std::fs::write(&log, "").unwrap();
    let d = git_repo(&["site/pricing.html", "copy/hello.txt"]);
    act(d.path(), &wrap, ws.root().to_str().unwrap(), Some("human"));
    let args = std::fs::read_to_string(&log).unwrap();
    assert!(args.lines().any(|l| l.trim_end() == format!("--root {} apply-check --latest", ws.root().display())), "{args}");
}

#[test]
fn m4_file_names_are_escaped_in_the_step_summary() {
    let d = git_repo(&["x/![p](https:%2F%2Fevil.example%2Fpixel.png).txt", "y/<img src=x>.txt"]);
    let o = act(d.path(), &bin(), "", Some("human"));
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(!o.summary.contains("![p]("), "an image link reached the summary: {}", o.summary);
    assert!(!o.summary.replace("\\<", "").contains("<img"), "raw html reached the summary: {}", o.summary);
    assert!(o.summary.contains("2 files matched no rule"), "{}", o.summary);
}

#[test]
fn m5_the_action_pins_its_own_release_and_fails_closed_by_default() {
    let y = std::fs::read_to_string(root().join("action.yml")).unwrap();
    let changelog = std::fs::read_to_string(root().join("CHANGELOG.md")).unwrap();
    let release = changelog.lines().find_map(|l| l.strip_prefix("## [").and_then(|r| r.split_once(']')).map(|(v, _)| v.to_string()).filter(|v| v != "Unreleased")).unwrap();
    let block = |k: &str| -> String {
        let start = y.find(&format!("  {k}:\n")).unwrap_or_else(|| panic!("{k}"));
        y[start..].lines().skip(1).take_while(|l| l.starts_with("    ")).collect::<Vec<_>>().join("\n")
    };
    assert!(block("version").contains(&format!("default: v{release}")), "{}", block("version"));
    assert!(block("fail-on").contains("default: human,escalate,unavailable"), "{}", block("fail-on"));
    let ci = std::fs::read_to_string(root().join("docs/ci.md")).unwrap();
    assert!(ci.contains("unavailable") && ci.contains("human,escalate,unavailable"), "docs/ci.md documents unavailable");
}

// ---- I2: report_dir and MCP output paths stay inside the root ----

#[test]
fn i2_a_report_dir_outside_the_root_is_refused_and_nothing_is_written() {
    let ws = Ws::new();
    ws.site();
    let a = ws.run("critical", &PB, false);
    ws.ingest(a, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(a, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let outside = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(outside.path(), ws.root().join("link")).unwrap();
    for dir in [outside.path().to_str().unwrap(), "../escaped-by-test", "docs/../../escaped-by-test", "link/juries", ""] {
        if dir.is_empty() {
            continue;
        }
        std::fs::write(ws.root().join("personajury.toml"), format!("report_dir = {dir:?}\n")).unwrap();
        let r = ws.cli(&["summary", &a.to_string(), "--save"]);
        assert_eq!(code(&r), "invalid_report_dir", "{dir}: {r}");
    }
    assert!(!ws.root().parent().unwrap().join("escaped-by-test").exists());
    assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0, "nothing written outside");
    // inside the root is fine
    std::fs::write(ws.root().join("personajury.toml"), "report_dir = \"docs/juries\"\n").unwrap();
    assert_eq!(ws.cli(&["summary", &a.to_string(), "--save"])["ok"], true);
}

fn mcp(root: &Path, name: &str, args: Value) -> Value {
    use std::io::Write;
    let mut child = Command::new(bin()).arg("--root").arg(root).arg("mcp").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap();
    let req = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":args}});
    writeln!(child.stdin.take().unwrap(), "{req}").unwrap();
    let out = child.wait_with_output().unwrap();
    let reply: Value = serde_json::from_slice(out.stdout.split(|b| *b == b'\n').next().unwrap()).unwrap();
    serde_json::from_str(reply["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}

#[test]
fn i2_mcp_output_paths_stay_inside_the_root_and_a_typed_cli_path_may_be_anywhere() {
    let ws = Ws::new();
    ws.site();
    let a = ws.run("critical", &PB, false);
    ws.ingest(a, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(a, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let outside = tempfile::tempdir().unwrap();
    let abs = outside.path().join("s.md");
    for (k, p) in [("md", abs.to_str().unwrap()), ("html", "../x.html"), ("md", "a/../../x.md")] {
        let r = mcp(ws.root(), "personajury_summary", json!({"runs": [a], k: p}));
        assert_eq!(code(&r), "invalid_report_dir", "{k} {p}: {r}");
    }
    assert!(!abs.exists());
    let r = mcp(ws.root(), "personajury_summary", json!({"runs": [a], "md": "out/s.md", "html": "out/s.html"}));
    assert_eq!(r["ok"], true, "{r}");
    assert!(ws.root().join("out/s.md").exists() && ws.root().join("out/s.html").exists());
    // the person at the CLI may write anywhere
    let r = ws.cli(&["summary", &a.to_string(), "--md", abs.to_str().unwrap()]);
    assert_eq!(r["ok"], true, "{r}");
    assert!(abs.exists());
}

// ---- I3: the summary text is terminal-safe; quotes keep the juror's words, cut only between words ----

#[test]
fn i3_control_characters_and_escape_sequences_never_reach_the_text_or_the_markdown() {
    let ws = Ws::new();
    ws.site();
    let a = ws.run("critical", &PB, false);
    let t = "Line one\n\n   two  spaces \u{1b}[31mRED\u{1b}[0m \u{7}\u{7f}\u{1b}]0;title\u{7}end\u{9b}2J";
    ws.ingest(a, &critical("ana", [2, 2, 2], &[t], &["page.json#n1"]));
    ws.ingest(a, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let s = data(ws.cli(&["summary", &a.to_string(), "--width", "200"]));
    let text = s["text"].as_str().unwrap();
    let f = ws.root().join("s.md");
    data(ws.cli(&["summary", &a.to_string(), "--md", f.to_str().unwrap()]));
    let md = std::fs::read_to_string(&f).unwrap();
    for (what, out) in [("text", text), ("md", md.as_str())] {
        for c in out.chars() {
            let u = c as u32;
            assert!(c == '\n' || !(u < 0x20 || u == 0x7f || (0x80..0xa0).contains(&u)), "{what} holds control {u:#x}: {out:?}");
        }
        assert!(!out.contains("[31m") && !out.contains("]0;title"), "{what} kept an escape sequence: {out:?}");
    }
    assert!(text.contains("\"Line one two spaces RED end2J\""), "whitespace collapsed, words kept: {text}");
}

#[test]
fn i3_a_long_quote_is_cut_between_words_and_ends_with_an_ellipsis() {
    let ws = Ws::new();
    ws.site();
    let a = ws.run("critical", &PB, false);
    let t = "The pricing page hides the annual price behind a toggle that nobody notices on a phone screen at all";
    let long_word = format!("see {}", "x".repeat(150));
    ws.ingest(a, &critical("ana", [2, 2, 2], &[t, &long_word], &["page.json#n1"]));
    ws.ingest(a, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let text = data(ws.cli(&["summary", &a.to_string(), "--width", "200"]))["text"].as_str().unwrap().to_string();
    let q = text.split("on works: \"").nth(1).and_then(|r| r.split('"').next()).unwrap_or_else(|| panic!("{text}"));
    let shown = q.strip_suffix('…').unwrap_or_else(|| panic!("a cut quote ends with …: {q:?}"));
    assert!(t.starts_with(shown), "{shown:?}");
    assert!(t[shown.len()..].starts_with(' '), "cut inside a word: {shown:?}");
    assert!(!shown.ends_with(' '));
    // a word longer than the limit is never cut inside: it is dropped after the words before it
    assert!(!text.contains(&"x".repeat(10)) || text.contains(&"x".repeat(150)), "a word was cut: {text}");
}

#[test]
fn i3_the_wrap_never_cuts_inside_a_word() {
    use personajury::domain::summary::wrap;
    let w = "a".repeat(50);
    let lines = wrap(&format!("short {w} tail"), 30);
    assert!(lines.contains(&w), "the long word stays whole on its own line: {lines:?}");
    assert!(lines.iter().all(|l| !l.contains('…')), "{lines:?}");
}

// ---- polish: plain first sentence, blockers rotate across jurors ----

#[test]
fn polish_the_first_sentence_is_plain_words() {
    // the fixture: the golden carries the exact sentence; here, an advise that has no rule says why in plain words
    let ws = Ws::new();
    ws.site();
    let a = ws.run("critical", &PB, false);
    ws.ingest(a, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(a, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let text = data(ws.cli(&["summary", &a.to_string(), "--width", "200"]))["text"].as_str().unwrap().to_string();
    let first = text.lines().nth(2).unwrap();
    assert!(first.starts_with("The jury says: advise. No rule was given, so the jury can only advise"), "{text}");
    assert!(!text.contains("fixes the quorum"), "{text}");

    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let t = data(cli_in(ws.path(), &["summary", &a.to_string(), &b.to_string(), "--width", "200"]))["text"].as_str().unwrap().to_string();
    assert!(
        t.contains("The jury says: escalate. Primary jurors raised 15 blockers, the groups disagree on 2 scores, and 26 findings have no decision yet."),
        "{t}"
    );
}

#[test]
fn polish_blocker_quotes_rotate_across_jurors_primary_first() {
    let ws = Ws::new();
    ws.site();
    let p = [("ana", Stake::Primary), ("bob", Stake::Primary), ("cara", Stake::Secondary)];
    let a = ws.run("critical", &p, false);
    ws.ingest(a, &critical("ana", [2, 2, 2], &["ana one", "ana two", "ana three"], &["page.json#n1"]));
    ws.ingest(a, &critical("bob", [2, 2, 2], &["bob one"], &["page.json#n1"]));
    ws.ingest(a, &critical("cara", [2, 2, 2], &["cara one"], &["page.json#n1"]));
    let text = data(ws.cli(&["summary", &a.to_string(), "--width", "200"]))["text"].as_str().unwrap().to_string();
    let line = text.lines().find(|l| l.starts_with("Blockers")).unwrap_or_else(|| panic!("{text}"));
    let order: Vec<&str> = ["ana (", "bob (", "cara ("].iter().copied().filter(|n| line.contains(n)).collect();
    assert_eq!(order, ["ana (", "bob (", "cara ("], "one per juror before anyone's second: {line}");
    let (ia, ib, ic) = (line.find("ana (").unwrap(), line.find("bob (").unwrap(), line.find("cara (").unwrap());
    assert!(ia < ib && ib < ic, "primary first: {line}");
    assert!(!line.contains("ana two"), "{line}");
}

// ---- M1: the calibration file matches the lens and the runs ----

#[test]
fn m1_a_calibration_for_another_lens_or_run_is_refused() {
    let ws = Ws::new();
    std::fs::write(ws.root().join("personajury.toml"), RULE_ALL).unwrap();
    ws.site();
    let a = ws.run("critical", &THREE, false);
    ws.ingest(a, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(a, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(a, &critical("cat", [4, 4, 4], &[], &["page.json#n1"]));
    low(&ws, a);
    let lv = ws.store.lens("critical").unwrap().unwrap().version;
    let f = ws.root().join("cal.json");
    let good = json!({"calibrated": true, "subject": "site", "lens": "critical", "lens_version": lv, "jury_run": a});
    for (what, body) in [
        ("no fields", json!({"calibrated": true})),
        ("another lens", json!({"calibrated": true, "subject": "site", "lens": "compare", "lens_version": lv, "jury_run": a})),
        ("another run", json!({"calibrated": true, "subject": "site", "lens": "critical", "lens_version": lv, "jury_run": 999})),
        ("another version", json!({"calibrated": true, "subject": "site", "lens": "critical", "lens_version": lv + 7, "jury_run": a})),
        ("another subject", json!({"calibrated": true, "subject": "other", "lens": "critical", "lens_version": lv, "jury_run": a})),
    ] {
        std::fs::write(&f, body.to_string()).unwrap();
        let r = ws.cli(&["apply-check", &a.to_string(), "--calibration", f.to_str().unwrap()]);
        assert_eq!(code(&r), "calibration_mismatch", "{what}: {r}");
    }
    std::fs::write(&f, good.to_string()).unwrap();
    assert_eq!(data(ws.cli(&["apply-check", &a.to_string(), "--calibration", f.to_str().unwrap()]))["answer"], "apply");
}

// ---- M2, M3: CSV locators ----

fn sheet(cite: &str) -> Value {
    let lines: Vec<Value> = ["works", "blocks", "missing"]
        .iter()
        .map(|d| json!({"dimension": d, "score": 4, "anchor_met": 3, "evidence": "seen", "citations": [cite], "blockers": []}))
        .collect();
    json!({"persona": "ana", "lines": lines})
}

#[test]
fn m2_m3_csv_lowercase_is_non_canonical_and_a_mid_field_quote_is_literal() {
    let ws = Ws::new();
    ws.subject("sheet", SubjectKind::Prices, false, &[("prices.csv", b"name,size\n27\" screen,12\nb,7\nc,8\n")]);
    let run = ws.run_of("sheet", "critical", &[("ana", Stake::Primary)], false);
    let try_ = |c: &str| ws.cli(&["verdict", "--run", &run.to_string(), "--json", &sheet(c).to_string()]);
    for bad in ["prices.csv#r9c9", "prices.csv#r1c1", "prices.csv#R1c1", "prices.csv#RC1", "prices.csv#R5C1"] {
        assert_eq!(code(&try_(bad)), "citation_unresolved", "{bad}");
    }
    // the quote in `27" screen` is a literal character: the sheet keeps its 4 records
    for ok in ["prices.csv#R3C1", "prices.csv#R4C2", "prices.csv#R2C2"] {
        assert_eq!(try_(ok)["ok"], true, "{ok}: {}", try_(ok));
    }
}
