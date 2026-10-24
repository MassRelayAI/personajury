//! doctor: the report without a store, and the counts with one. Temp HOME only.
use personajury::domain::*;
use personajury::store::Store;
use serde_json::Value;

fn doctor(ws: &std::path::Path, home: &std::path::Path) -> Value {
    let out = assert_cmd::Command::cargo_bin("personajury")
        .unwrap()
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("PERSONAJURY_BIN", "/opt/jg/bin/personajury")
        .arg("--root")
        .arg(ws)
        .arg("doctor")
        .output()
        .unwrap();
    assert!(out.status.success());
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn doctor_without_store() {
    let ws = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let v = doctor(ws.path(), home.path());
    assert_eq!(v["ok"], true);
    assert_eq!(v["verb"], "doctor");
    let d = &v["data"];
    assert!(d["store"].is_null() && d["counts"].is_null() && d["schema_version"].is_null());
    assert_eq!(
        d["engine"],
        serde_json::json!({"model_calls": false, "network": false})
    );
    assert!(d["clients"].is_array());
    assert!(
        !ws.path().join(".personajury").exists(),
        "doctor must not create a store"
    );
}

#[test]
fn doctor_counts() {
    let ws = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let s = Store::open(ws.path()).unwrap();
    for (id, hash) in [("a", "h1"), ("b", "h2")] {
        s.insert_persona(&Persona {
            id: id.into(),
            version: 0,
            name: format!("N {id}"),
            tags: vec!["builders".into()],
            stake_rules: "r".into(),
            profile_json: "{}".into(),
            origin: Origin::Hand,
            hash: hash.into(),
            traits: None,
            serves: vec![],
        })
        .unwrap();
    }
    s.upsert_panel("p", "tag:builders").unwrap();
    drop(s);
    let v = doctor(ws.path(), home.path());
    let d = &v["data"];
    assert!(d["store"]
        .as_str()
        .unwrap()
        .ends_with(".personajury/store.db"));
    assert_eq!(d["schema_version"], 5);
    let c = &d["counts"];
    assert_eq!(
        (
            c["personas"].as_i64(),
            c["panels"].as_i64(),
            c["runs"].as_i64()
        ),
        (Some(2), Some(1), Some(0))
    );
    assert_eq!(c["lenses"], 4, "the four shipped lenses are seeded");
    for k in ["subjects", "comparators", "verdict_lines"] {
        assert_eq!(c[k], 0, "{k}");
    }
    assert!(
        c["unresolved_findings"].is_null(),
        "that count belongs to check"
    );
    assert_eq!(d["engine"]["model_calls"], false);
    assert_eq!(d["engine"]["network"], false);
}
