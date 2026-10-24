//! Shared helpers of the v0.6 trait tests: the CLI as a black box, persona files with traits, an installed catalog.
#![allow(dead_code)]
use assert_cmd::Command;
use serde_json::{json, Value};
use std::path::Path;

pub fn jg(ws: &Path, args: &[&str]) -> Value {
    let out = Command::cargo_bin("personajury").unwrap().arg("--root").arg(ws).args(args).output().unwrap();
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!("{args:?}: not json ({e}): {} {}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
    })
}
pub fn ok(v: Value) -> Value {
    assert_eq!(v["ok"], true, "{v}");
    v["data"].clone()
}
pub fn code(v: &Value) -> String {
    assert_eq!(v["ok"], false, "{v}");
    v["error"]["code"].as_str().unwrap().to_string()
}
pub fn repo() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
/// A workspace with the catalog installed.
pub fn installed() -> tempfile::TempDir {
    let ws = tempfile::tempdir().unwrap();
    ok(jg(ws.path(), &["trait", "install"]));
    ws
}
/// A persona file body; `traits` is a JSON object or null.
pub fn persona_body(id: &str, tags: &[&str], traits: Value) -> Value {
    let mut v = json!({
        "id": id, "name": format!("Person {id}"), "tags": tags,
        "role": "r", "context": "c", "daily_tasks": "d", "fears": "f",
        "never_reads": "n", "decides_by": "b", "stake_rules": "s"
    });
    if !traits.is_null() {
        v["traits"] = traits;
    }
    v
}
/// Write a persona file and `persona add` it; returns the envelope.
pub fn add_persona(ws: &Path, body: &Value) -> Value {
    let dir = ws.join("pool");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(format!("{}.json", body["id"].as_str().unwrap()));
    std::fs::write(&f, body.to_string()).unwrap();
    jg(ws, &["persona", "add", "--file", f.to_str().unwrap()])
}
pub fn add_ok(ws: &Path, id: &str, tags: &[&str], traits: Value) {
    ok(add_persona(ws, &persona_body(id, tags, traits)));
}
