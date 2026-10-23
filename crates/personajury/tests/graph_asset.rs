//! docs/assets/jury-graph.svg is the graph of the recorded fixture jury, and a fresh render must equal it byte for
//! byte (like `readme-assets --check`). `JURYGRAPH_ASSET_REGEN=1 cargo test -p jurygraph --test graph_asset`
//! rewrites the file from this render. The workspace is built the way tests/fixture_replay.rs builds it.
use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

const STAKES: &str = "builder-ana=primary,builder-mei=primary,builder-ravi=primary,newcomer-sofia=primary,\
company-george=secondary,company-hanna=secondary,investor-claire=bystander";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn j(ws: &Path, args: &[&str]) -> Value {
    let out = Command::cargo_bin("jurygraph").unwrap().arg("--root").arg(ws).args(args).output().unwrap();
    let v: Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("{args:?}: not an envelope ({e}): {}", String::from_utf8_lossy(&out.stdout)));
    assert_eq!(v["ok"], true, "{args:?} refused: {}", v["error"]);
    v
}
fn fixture(rel: &str) -> String {
    repo().join("fixtures").join(rel).to_string_lossy().into_owned()
}

/// The fixture jury ingested and decided, the way the replay does it; returns (critical run, compare run).
fn jury(ws: &Path) -> (i64, i64) {
    let mut personas: Vec<_> = std::fs::read_dir(repo().join("fixtures/personas")).unwrap().map(|e| e.unwrap().path()).collect();
    personas.sort();
    for p in personas {
        j(ws, &["persona", "add", "--file", p.to_str().unwrap()]);
    }
    j(ws, &["panel", "add", "fixture", "--query", "3 builder, 2 company, 2 others"]);
    let (hw, hp, pw, pp) = (
        fixture("site-walk/home.walk.json"),
        fixture("site-walk/home.png"),
        fixture("site-walk/pricing.walk.json"),
        fixture("site-walk/pricing.png"),
    );
    j(ws, &["subject", "add", "lantern-site", "--kind", "page", "--attach", &hw, &hp, &pw, &pp]);
    let (cw, cp) = (fixture("comparators/acme-pricing/pricing.walk.json"), fixture("comparators/acme-pricing/pricing.png"));
    j(ws, &["comparator", "add", "acme-pricing", "--kind", "prices", "--attach", &cw, &cp]);
    let common = ["--subject", "lantern-site", "--panel", "fixture", "--stake", STAKES, "--spend", "7", "--model", "claude-sonnet-5-5"];
    let mut critical = vec!["brief", "--lens", "critical"];
    critical.extend(common);
    let mut compare = vec!["brief", "--lens", "compare", "--comparator", "acme-pricing"];
    compare.extend(common);
    let a = j(ws, &critical)["data"]["run"].as_i64().unwrap();
    let b = j(ws, &compare)["data"]["run"].as_i64().unwrap();
    let mut files: Vec<_> = std::fs::read_dir(repo().join("fixtures/run/verdicts")).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    for f in &files {
        let name = f.file_stem().unwrap().to_str().unwrap();
        let run = if name.ends_with("--critical") { a } else { b };
        j(ws, &["verdict", "--run", &run.to_string(), "--file", f.to_str().unwrap()]);
    }
    // the recorded decisions, found by persona, dimension, option and text as the replay does
    let out = Command::cargo_bin("jurygraph").unwrap().arg("--root").arg(ws).arg("check").output().unwrap();
    let c: Value = serde_json::from_slice(&out.stdout).unwrap();
    let unresolved = c["data"]["unresolved"].as_array().unwrap().clone();
    let decisions = std::fs::read_to_string(repo().join("fixtures/run/decisions.jsonl")).unwrap();
    for line in decisions.lines() {
        let d: Value = serde_json::from_str(line).unwrap();
        let keys = ["persona", "dimension", "about", "text"];
        let hit = unresolved.iter().find(|u| keys.iter().all(|k| u[*k] == d[*k])).expect("a finding per decision");
        let mut args = vec!["decide".to_string(), hit["finding"].as_str().unwrap().to_string()];
        args.extend(d["args"].as_array().unwrap().iter().map(|a| a.as_str().unwrap().to_string()));
        args.extend(["--by".to_string(), "foreman (orchestrator, Opus)".to_string()]);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        j(ws, &args);
    }
    (a, b)
}

#[test]
fn the_committed_graph_is_a_fresh_render_of_the_fixture_jury() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = jury(ws.path());
    let v = j(ws.path(), &["graph", &a.to_string(), &b.to_string(), "--svg"]);
    let fresh = v["data"]["svg"].as_str().unwrap();
    let path = repo().join("docs/assets/jury-graph.svg");
    if std::env::var_os("JURYGRAPH_ASSET_REGEN").is_some() {
        std::fs::write(&path, fresh).unwrap();
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(fresh, committed, "docs/assets/jury-graph.svg differs from a fresh render; regenerate with JURYGRAPH_ASSET_REGEN=1");
    // the numbers on the picture are the matrix's
    assert!(fresh.contains("7 personas (primary 4, secondary 2, bystander 1)"));
}
