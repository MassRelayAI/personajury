//! 0.6.1: fixes from a first real run on Windows (usage feedback of 2026-10-25, items 1-10).
use assert_cmd::Command;
#[path = "traits_support/mod.rs"]
mod support;
use serde_json::Value;
use std::path::Path;
use support::{add_persona, installed, jg, ok, persona_body};

fn cmd(home: &Path, ws: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("personajury")
        .unwrap()
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("xdg"))
        .env_remove("PERSONAJURY_BIN")
        .current_dir(home)
        .arg("--root")
        .arg(ws)
        .args(args)
        .output()
        .unwrap()
}
fn run(home: &Path, ws: &Path, args: &[&str]) -> Value {
    let out = cmd(home, ws, args);
    serde_json::from_slice(&out.stdout).unwrap_or_else(|_| panic!("not json: {}", String::from_utf8_lossy(&out.stdout)))
}

#[test]
fn item1_init_creates_the_workspace_so_lens_list_works_straight_after() {
    let d = tempfile::tempdir().unwrap();
    let w = d.path().join("ws");
    std::fs::create_dir_all(&w).unwrap();
    assert_eq!(run(d.path(), &w, &["init"])["ok"], true);
    let v = run(d.path(), &w, &["lens", "list"]);
    assert_eq!(v["ok"], true, "{v}");
    assert!(v["data"].to_string().contains("critical"), "{v}");
}

#[test]
fn item3_subject_add_usage_shows_name_first() {
    let d = tempfile::tempdir().unwrap();
    let out = cmd(d.path(), d.path(), &["subject", "add", "--help"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let usage = text.lines().find(|l| l.contains("Usage:")).unwrap_or_default().to_string();
    assert!(usage.contains("add <NAME>") && usage.find("<NAME>") < usage.find("--attach"), "{usage}");
}

#[test]
fn item3_name_after_double_dash_works_when_it_follows_the_files() {
    let d = tempfile::tempdir().unwrap();
    let w = d.path().join("ws");
    std::fs::create_dir_all(&w).unwrap();
    let f = d.path().join("a.md");
    std::fs::write(&f, "# a\n").unwrap();
    let v = run(d.path(), &w, &["subject", "add", "--kind", "document", "--attach", f.to_str().unwrap(), "--", "probe"]);
    assert_eq!(v["ok"], true, "{v}");
}


#[test]
fn item5_an_effort_trim_of_an_explicit_query_is_a_loud_warning_naming_what_was_asked() {
    let ws = installed();
    let w = ws.path();
    for (id, tag) in [("u1", "user"), ("u2", "user"), ("e1", "expert"), ("e2", "expert"), ("e3", "expert"), ("b1", "business")] {
        ok(add_persona(w, &persona_body(id, &[tag], serde_json::Value::Null)));
    }
    let f = w.join("page.md");
    std::fs::write(&f, "# page\nline\n").unwrap();
    ok(jg(w, &["subject", "add", "s", "--kind", "document", "--attach", f.to_str().unwrap()]));
    let d = ok(jg(
        w,
        &["brief", "--subject", "s", "--query", "2 user=primary, 3 expert=secondary, 1 business=bystander", "--lens", "critical", "--model", "m", "--effort", "low"],
    ));
    let warn = d["warnings"].as_array().unwrap_or_else(|| panic!("no warnings: {d}"));
    let w0 = warn.iter().find(|x| x["code"] == "effort_trimmed_seats").unwrap_or_else(|| panic!("{d}"));
    assert_eq!(w0["asked"], 6, "{w0}");
    assert_eq!(w0["seated"], 3, "{w0}");
    assert!(w0["note"].as_str().unwrap().contains("--spend"), "{w0}");
}
