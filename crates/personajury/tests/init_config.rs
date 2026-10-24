//! The starter config (D38): `personajury init` and `setup` write a commented personajury.toml that says "no jury" for the
//! code paths and "jury, surface class" for the screens, and never overwrite one the project already has.
use assert_cmd::Command;
use serde_json::{json, Value};
use std::path::Path;

fn run(home: &Path, ws: &Path, args: &[&str]) -> Value {
    let out = Command::cargo_bin("personajury")
        .unwrap()
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("xdg"))
        .env_remove("PERSONAJURY_BIN")
        .current_dir(home)
        .arg("--root")
        .arg(ws)
        .args(args)
        .output()
        .unwrap();
    serde_json::from_slice(&out.stdout).unwrap_or_else(|_| panic!("not json: {}", String::from_utf8_lossy(&out.stdout)))
}
fn quorum_of(ws: &Path, path: &str) -> Value {
    let v = run(ws, ws, &["should-run", "--paths", path]);
    assert_eq!(v["ok"], true, "{v}");
    v["data"]["quorum"].clone()
}

#[test]
fn init_writes_a_commented_starter_that_classifies_code_as_none_and_screens_as_a_surface() {
    let d = tempfile::tempdir().unwrap();
    let w = d.path().join("ws");
    std::fs::create_dir_all(&w).unwrap();
    let v = run(d.path(), &w, &["init"]);
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["data"]["state"], "written");
    let text = std::fs::read_to_string(w.join("personajury.toml")).unwrap();
    assert!(text.lines().filter(|l| l.starts_with('#')).count() >= 8, "commented:\n{text}");
    assert!(text.contains("a starting point"), "{text}");
    for p in ["crates/a/src/lib.rs", "migrations/001.sql", "tools/x.sh", ".github/workflows/ci.yml", "router/rules/a.toml"] {
        assert_eq!(quorum_of(&w, p), json!("none"), "{p}");
    }
    for p in ["screens/home.tsx", "pages/pricing.html", "docs/guide.md"] {
        let q = quorum_of(&w, p);
        assert!(q == json!("low") || q == json!("high"), "{p}: {q}");
    }
    let hit = run(&w, &w, &["should-run", "--paths", "screens/home.tsx"])["data"]["hits"][0]["class"].clone();
    assert_eq!(hit, "surface");
}

#[test]
fn init_never_overwrites_a_config_the_project_has() {
    let d = tempfile::tempdir().unwrap();
    let w = d.path().join("ws");
    std::fs::create_dir_all(&w).unwrap();
    std::fs::write(w.join("personajury.toml"), "tokens_per_brief = 1\n").unwrap();
    let v = run(d.path(), &w, &["init"]);
    assert_eq!(v["data"]["state"], "present");
    assert_eq!(std::fs::read_to_string(w.join("personajury.toml")).unwrap(), "tokens_per_brief = 1\n");
}

#[test]
fn setup_plans_then_writes_the_starter_and_leaves_an_existing_one() {
    let d = tempfile::tempdir().unwrap();
    let w = d.path().join("ws");
    std::fs::create_dir_all(&w).unwrap();
    let args = ["setup", "--client", "codex", "--bin", "/opt/jg/bin/personajury"];
    let plan = run(d.path(), &w, &args);
    assert_eq!(plan["ok"], true, "{plan}");
    assert_eq!(plan["data"]["project_config"]["state"], "planned");
    assert!(!w.join("personajury.toml").exists());
    let mut apply = args.to_vec();
    apply.push("--apply");
    let done = run(d.path(), &w, &apply);
    assert_eq!(done["data"]["project_config"]["state"], "written", "{done}");
    assert!(w.join("personajury.toml").exists());
    let again = run(d.path(), &w, &apply);
    assert_eq!(again["data"]["project_config"]["state"], "present");
}
