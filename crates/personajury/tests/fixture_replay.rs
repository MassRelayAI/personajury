//! The fixture jury, replayed (plan Task 7, the acceptance). Builds a fresh workspace from `fixtures/` exactly as
//! `fixtures/run/setup.sh` does, checks every brief against the hashes recorded when the jury ran, ingests the fourteen
//! verdicts the persona subagents wrote, and compares the matrix and the picks with the checked-in files byte for byte.
//! A changed brief, a refused verdict or a moved score turns this red. The matrix, table and picks are engine output:
//! `PERSONAJURY_FIXTURE_REGEN=1 cargo test -p personajury --test fixture_replay` rewrites them from this replay (the
//! briefs, verdicts and decisions are never rewritten).
use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

const STAKES: &str = "builder-ana=primary,builder-mei=primary,builder-ravi=primary,newcomer-sofia=primary,\
company-george=secondary,company-hanna=secondary,investor-claire=bystander";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn j(ws: &Path, args: &[&str]) -> Value {
    let out = Command::cargo_bin("personajury")
        .unwrap()
        .arg("--root")
        .arg(ws)
        .args(args)
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!("{args:?}: not an envelope ({e}): {}", String::from_utf8_lossy(&out.stdout))
    });
    assert_eq!(v["ok"], true, "{args:?} refused: {}", v["error"]);
    v
}
fn fixture(rel: &str) -> String {
    repo().join("fixtures").join(rel).to_string_lossy().into_owned()
}
fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join("fixtures/run").join(rel)).unwrap()
}
/// Compare engine output with its checked-in file, or rewrite the file when regenerating.
fn same(rel: &str, got: &str, what: &str) {
    if std::env::var_os("PERSONAJURY_FIXTURE_REGEN").is_some() {
        std::fs::write(repo().join("fixtures/run").join(rel), got).unwrap();
    }
    assert_eq!(got, read(rel), "{what}");
}

fn build(ws: &Path) -> (i64, i64) {
    let mut personas: Vec<_> = std::fs::read_dir(repo().join("fixtures/personas"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    personas.sort();
    j(ws, &["trait", "install"]);
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
    let (cw, cp) = (
        fixture("comparators/acme-pricing/pricing.walk.json"),
        fixture("comparators/acme-pricing/pricing.png"),
    );
    j(ws, &["comparator", "add", "acme-pricing", "--kind", "prices", "--attach", &cw, &cp]);
    let model = "claude-sonnet-5-5";
    let common = ["--subject", "lantern-site", "--panel", "fixture", "--stake", STAKES, "--spend", "7", "--model", model];
    let mut critical = vec!["brief", "--lens", "critical"];
    critical.extend(common);
    // recorded before D32: the recorded verdicts score `about: pricing.png`, so this brief keeps the attachment options
    let mut compare = vec!["brief", "--lens", "compare", "--comparator", "acme-pricing", "--options", "attachments"];
    compare.extend(common);
    let a = j(ws, &critical)["data"]["run"].as_i64().unwrap();
    let b = j(ws, &compare)["data"]["run"].as_i64().unwrap();
    (a, b)
}

#[test]
fn fixture_jury_replays_byte_for_byte() {
    let ws = tempfile::tempdir().unwrap();
    let (critical, compare) = build(ws.path());

    // the briefs the jurors read are the briefs this build composes
    if std::env::var_os("PERSONAJURY_FIXTURE_REGEN").is_some() {
        let mut out = serde_json::Map::new();
        for (run, lens) in [(critical, "critical"), (compare, "compare")] {
            let dir = ws.path().join(format!(".personajury/runs/{run}"));
            let mut m = serde_json::Map::new();
            let mut names: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
            names.sort();
            for n in names.iter().filter(|n| n.ends_with(&format!("--{lens}.md"))) {
                let bytes = std::fs::read(dir.join(n)).unwrap();
                m.insert(n.trim_end_matches(&format!("--{lens}.md")).to_string(), Value::String(personajury::domain::hash::sha256_hex(&bytes)));
            }
            out.insert(lens.to_string(), Value::Object(m));
        }
        std::fs::write(repo().join("fixtures/run/briefs.json"), serde_json::to_string_pretty(&Value::Object(out)).unwrap() + "\n").unwrap();
    }
    let recorded: Value = serde_json::from_str(&read("briefs.json")).unwrap();
    for (run, lens) in [(critical, "critical"), (compare, "compare")] {
        let dir = ws.path().join(format!(".personajury/runs/{run}"));
        for (persona, hash) in recorded[lens].as_object().unwrap() {
            let bytes = std::fs::read(dir.join(format!("{persona}--{lens}.md"))).unwrap();
            assert_eq!(
                personajury::domain::hash::sha256_hex(&bytes),
                hash.as_str().unwrap(),
                "brief {persona}--{lens} differs from the one the jury read"
            );
        }
    }

    // the fourteen verdicts, as the persona subagents wrote them, are all accepted
    let mut files: Vec<_> = std::fs::read_dir(repo().join("fixtures/run/verdicts"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    assert_eq!(files.len(), 14);
    for f in &files {
        let name = f.file_stem().unwrap().to_str().unwrap();
        let run = if name.ends_with("--critical") { critical } else { compare };
        j(ws.path(), &["verdict", "--run", &run.to_string(), "--file", f.to_str().unwrap()]);
    }

    let m = j(ws.path(), &["matrix", &critical.to_string(), &compare.to_string(), "--table", "--detail", "full"]);
    let strata = serde_json::to_string_pretty(&m["data"]["strata"]).unwrap() + "\n";
    same("matrix.json", &strata, "the matrix moved");
    same("matrix.md", m["data"]["table"].as_str().unwrap(), "the table moved");

    let p = j(ws.path(), &["pick", &compare.to_string()]);
    let picks = serde_json::to_string_pretty(&p["data"]["strata"]).unwrap() + "\n";
    same("picks.json", &picks, "the picks moved");

    // every finding blocks the finish until a person's word is on it (report §10(1)) ...
    let out = Command::cargo_bin("personajury").unwrap().arg("--root").arg(ws.path()).arg("check").output().unwrap();
    let c: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(c["error"]["code"], "check_failed");
    let decisions = read("decisions.jsonl");
    let unresolved = c["data"]["unresolved"].as_array().unwrap().clone();
    assert_eq!(unresolved.len(), decisions.lines().count());
    // ... and the foreman's recorded decisions resolve every one of them. A decision names its point by persona,
    // dimension, option and text (the finding id is a hash whose formula may change); check lists those four.
    for line in decisions.lines() {
        let d: Value = serde_json::from_str(line).unwrap();
        let keys = ["persona", "dimension", "about", "text"];
        let hits: Vec<&Value> = unresolved.iter().filter(|u| keys.iter().all(|k| u[*k] == d[*k])).collect();
        assert_eq!(hits.len(), 1, "one unresolved finding per decision: {d}");
        let id = hits[0]["finding"].as_str().unwrap().to_string();
        let mut args = vec!["decide".to_string(), id];
        args.extend(d["args"].as_array().unwrap().iter().map(|a| a.as_str().unwrap().to_string()));
        args.extend(["--by".to_string(), "foreman (orchestrator, Opus)".to_string()]);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        j(ws.path(), &args);
    }
    let c = j(ws.path(), &["check"]);
    assert_eq!(c["data"]["unresolved"].as_array().unwrap().len(), 0);
}
