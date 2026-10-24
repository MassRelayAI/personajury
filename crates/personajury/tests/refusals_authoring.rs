//! Refusals of the authoring verbs: R8 (a persona without a stake at brief) and R10 (spend above the panel).
use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fx(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(rel)
        .to_str()
        .unwrap()
        .to_string()
}
fn jg(ws: &Path, args: &[&str]) -> Value {
    let out = Command::cargo_bin("personajury")
        .unwrap()
        .arg("--root")
        .arg(ws)
        .args(args)
        .output()
        .unwrap();
    serde_json::from_slice(&out.stdout).unwrap()
}
fn setup(ws: &Path) {
    jg(ws, &["trait", "install"]);
    for p in ["builder-ana", "builder-mei", "company-george"] {
        let r = jg(
            ws,
            &[
                "persona",
                "add",
                "--file",
                &fx(&format!("personas/{p}.json")),
            ],
        );
        assert_eq!(r["ok"], true, "{r}");
    }
    let a = fx("site-walk/pricing.walk.json");
    let r = jg(
        ws,
        &[
            "subject", "add", "pricing", "--kind", "page", "--attach", &a,
        ],
    );
    assert_eq!(r["ok"], true, "{r}");
    let r = jg(
        ws,
        &["panel", "add", "trio", "--query", "2 builder, 1 company"],
    );
    assert_eq!(r["ok"], true, "{r}");
}
fn brief(ws: &Path, stakes: &str, extra: &[&str]) -> Value {
    let mut a = vec![
        "brief",
        "--subject",
        "pricing",
        "--panel",
        "trio",
        "--lens",
        "critical",
        "--stake",
        stakes,
        "--model",
        "m",
    ];
    a.extend_from_slice(extra);
    jg(ws, &a)
}

#[test]
fn brief_without_stake_refused() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    let v = brief(ws.path(), "builder-ana=primary,builder-mei=primary", &[]);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "persona_without_stake");
    assert!(v["error"]["message"]
        .as_str()
        .unwrap()
        .contains("company-george"));
    // nothing was written: no run folder
    assert!(!ws.path().join(".personajury/runs").exists());
    // the same call with the stake given passes
    let ok = brief(
        ws.path(),
        "builder-ana=primary,builder-mei=primary,company-george=secondary",
        &[],
    );
    assert_eq!(ok["ok"], true, "{ok}");
}

#[test]
fn brief_spend_above_panel_refused() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    let s = "builder-ana=primary,builder-mei=primary,company-george=secondary";
    let v = brief(ws.path(), s, &["--spend", "4"]);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "panel_too_small");
    let ok = brief(ws.path(), s, &["--spend", "3"]);
    assert_eq!(ok["ok"], true, "{ok}");
}
