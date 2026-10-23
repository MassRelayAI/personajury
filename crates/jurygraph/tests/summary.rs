//! v0.5 (D24): `summary`, the answer display. Sentences first, a numbers table "for checking", then "Next".
//! No bars anywhere. Every test here was seen red first.
mod judging_support;
use assert_cmd::Command;
use judging_support::*;
use serde_json::{json, Value};
use std::path::Path;

fn text(v: &Value) -> String {
    assert_eq!(v["ok"], true, "{v}");
    v["data"]["text"].as_str().unwrap().to_string()
}

#[test]
fn the_fixture_run_summary_is_the_golden_file_byte_for_byte() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let got = text(&j(ws.path(), &["summary", &a.to_string(), &b.to_string()]));
    let golden = repo().join("fixtures/run/summary.txt");
    if std::env::var_os("JURYGRAPH_FIXTURE_REGEN").is_some() {
        std::fs::write(&golden, &got).unwrap();
    }
    assert_eq!(got, std::fs::read_to_string(&golden).unwrap(), "the summary moved");
    assert!(got.lines().count() <= 30, "under about thirty lines:\n{got}");
}

#[test]
fn the_layout_is_sentences_then_the_table_then_next_and_has_no_bars() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let t = text(&j(ws.path(), &["summary", &a.to_string(), &b.to_string()]));
    for bar in ['█', '▌', '▇', '▏'] {
        assert!(!t.contains(bar), "no bars anywhere:\n{t}");
    }
    let table = t.find("for checking").expect(&t);
    let next = t.find("Next:").expect(&t);
    assert!(table > 0 && next > table, "{t}");
    assert!(t.trim_end().lines().last().unwrap().starts_with("Next:"), "{t}");
    // the table holds P·S·B medians
    assert!(t.lines().any(|l| l.matches('·').count() >= 2), "{t}");
}

#[test]
fn a_pooled_row_never_appears() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let t = text(&j(ws.path(), &["summary", &a.to_string(), &b.to_string()]));
    let lower = t.to_lowercase();
    for w in ["average", "mean", "overall", "combined", "all strata", "pooled", "total score"] {
        assert!(!lower.contains(w), "pooled word {w} in:\n{t}");
    }
    // every table cell is exactly three strata or a dash
    let start = t.find("for checking").unwrap();
    for l in t[start..].lines().skip(1).take_while(|l| !l.starts_with("Next:") && !l.trim().is_empty()) {
        for cell in l.split_whitespace().filter(|c| c.contains('·')) {
            assert_eq!(cell.matches('·').count(), 2, "a cell is P·S·B: {l}");
        }
    }
}

#[test]
fn a_pick_lens_shows_the_counts_per_stratum_in_numbers() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let t = text(&j(ws.path(), &["summary", &a.to_string(), &b.to_string()]));
    assert!(t.contains("3 of 4"), "primary picked cmp:acme-pricing 3 of 4:\n{t}");
    assert!(t.contains("primary") && t.contains("secondary"), "{t}");
}

#[test]
fn two_rounds_show_the_change_as_before_arrow_after() {
    let ws = Ws::new();
    ws.site();
    let r1 = ws.run("critical", &TWO, false);
    ws.ingest(r1, &critical("ana", [3, 3, 3], &["slow", "ugly"], &["page.json#n1"]));
    ws.ingest(r1, &critical("bob", [3, 3, 3], &[], &["page.json#n1"]));
    let r2 = ws.run("critical", &TWO, false);
    ws.ingest(r2, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(r2, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
    let t = text(&ws.cli(&["summary", &r1.to_string(), &r2.to_string()]));
    assert!(t.contains("2 → 0"), "primary blockers went 2 → 0:\n{t}");
    assert!(!t.contains('█'), "{t}");
    // one round shows no arrow
    let one = text(&ws.cli(&["summary", &r2.to_string()]));
    assert!(!one.contains('→'), "{one}");
}

#[test]
fn the_width_is_respected() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    for w in [40usize, 60, 100] {
        let t = text(&j(ws.path(), &["summary", &a.to_string(), &b.to_string(), "--width", &w.to_string()]));
        for l in t.lines() {
            assert!(l.chars().count() <= w, "width {w}: {l:?}");
        }
    }
    assert_eq!(cli_in(ws.path(), &["summary", &a.to_string(), "--width", "10"])["error"]["code"], "invalid_arguments");
}

#[test]
fn the_text_is_deterministic_and_quotes_carry_their_persona() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let t1 = text(&j(ws.path(), &["summary", &a.to_string(), &b.to_string()]));
    let t2 = text(&j(ws.path(), &["summary", &a.to_string(), &b.to_string()]));
    assert_eq!(t1, t2);
    assert!(t1.contains("builder-"), "a quoted blocker names its persona:\n{t1}");
}

#[test]
fn the_html_is_one_offline_file_with_the_graph_and_the_tables_and_the_same_bytes_twice() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let (f1, f2) = (ws.path().join("s1.html"), ws.path().join("s2.html"));
    let r = j(ws.path(), &["summary", &a.to_string(), &b.to_string(), "--html", f1.to_str().unwrap()]);
    assert_eq!(r["data"]["html"], f1.to_str().unwrap());
    j(ws.path(), &["summary", &a.to_string(), &b.to_string(), "--html", f2.to_str().unwrap()]);
    let (h1, h2) = (std::fs::read_to_string(&f1).unwrap(), std::fs::read_to_string(&f2).unwrap());
    assert_eq!(h1, h2);
    assert!(h1.contains("<svg") && h1.contains("<style>") && h1.contains("<table"), "{}", &h1[..200]);
    for bad in ["<script", "http://", "https://", "<link", "src=\"http"] {
        // the svg namespace is the only URL allowed
        let n = h1.matches(bad).count() - if bad == "http://" { h1.matches("http://www.w3.org/2000/svg").count() } else { 0 };
        assert_eq!(n, 0, "{bad} in the html");
    }
    assert!(h1.contains("primary") && h1.contains("bystander"), "per-stratum tables");
}

#[test]
fn the_mcp_tool_returns_the_same_text() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let want = text(&j(ws.path(), &["summary", &a.to_string(), &b.to_string()]));
    let mut child = std::process::Command::new(assert_cmd::cargo::cargo_bin("jurygraph"))
        .arg("--root")
        .arg(ws.path())
        .arg("mcp")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    let req = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"jurygraph_summary","arguments":{"runs":[a,b]}}});
    writeln!(child.stdin.take().unwrap(), "{req}").unwrap();
    let out = child.wait_with_output().unwrap();
    let reply: Value = serde_json::from_slice(out.stdout.split(|b| *b == b'\n').next().unwrap()).unwrap();
    let env: Value = serde_json::from_str(reply["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(env["data"]["text"], want, "{env}");
    let _ = Command::cargo_bin("jurygraph");
}

// ---- --md and --save (the maintainer's addition) ----

#[test]
fn the_markdown_summary_is_a_golden_file_with_the_sentences_the_table_the_decisions_the_hashes_and_the_date() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let f = ws.path().join("s.md");
    let r = j(ws.path(), &["summary", &a.to_string(), &b.to_string(), "--md", f.to_str().unwrap()]);
    assert_eq!(r["data"]["md"], f.to_str().unwrap());
    let got = std::fs::read_to_string(&f).unwrap();
    // the date and the hashes are data of the runs, so the file is the same on any day; the golden is for the fixture's
    // recorded briefs, whose created_at is the run's, so the date line is masked here
    let masked: String = got.lines().map(|l| if l.starts_with("Date: ") { "Date: <run date>" } else { l }).collect::<Vec<_>>().join("\n") + "\n";
    let golden = repo().join("fixtures/run/summary.md");
    if std::env::var_os("JURYGRAPH_FIXTURE_REGEN").is_some() {
        std::fs::write(&golden, &masked).unwrap();
    }
    assert_eq!(masked, std::fs::read_to_string(&golden).unwrap(), "the markdown summary moved");
    assert!(got.contains("for checking") && got.contains("Next:") && got.contains("brief sha256"), "{got}");
    assert!(got.lines().any(|l| l.starts_with("Date: 20")), "{got}");
    assert!(!got.contains('█'));
    let f2 = ws.path().join("s2.md");
    j(ws.path(), &["summary", &a.to_string(), &b.to_string(), "--md", f2.to_str().unwrap()]);
    assert_eq!(got, std::fs::read_to_string(&f2).unwrap(), "deterministic");
}

#[test]
fn save_without_a_report_dir_is_refused_and_nothing_is_written() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    let r = j_any(ws.path(), &["summary", &a.to_string(), &b.to_string(), "--save"]);
    assert_eq!(r["error"]["code"], "no_report_dir", "{r}");
    // by default nothing is written either
    j(ws.path(), &["summary", &a.to_string(), &b.to_string()]);
    assert!(!ws.path().join("docs").exists());
}

#[test]
fn save_with_a_report_dir_writes_the_dated_path() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    std::fs::write(ws.path().join("jurygraph.toml"), "report_dir = \"docs/juries\"\n").unwrap();
    let r = j(ws.path(), &["summary", &a.to_string(), &b.to_string(), "--save"]);
    let path = r["data"]["saved"].as_str().unwrap().to_string();
    let name = Path::new(&path).file_name().unwrap().to_str().unwrap().to_string();
    assert!(name.ends_with("-lantern-site.md") && name.len() == "2026-10-24-lantern-site.md".len(), "{name}");
    assert!(name.starts_with("20"), "{name}");
    assert!(path.contains("docs/juries/"), "{path}");
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(body.contains("Next:"), "{body}");
    // the same markdown as --md
    let f = ws.path().join("m.md");
    j(ws.path(), &["summary", &a.to_string(), &b.to_string(), "--md", f.to_str().unwrap()]);
    assert_eq!(body, std::fs::read_to_string(&f).unwrap());
}

fn j_any(ws: &Path, args: &[&str]) -> Value {
    cli_in(ws, args)
}
