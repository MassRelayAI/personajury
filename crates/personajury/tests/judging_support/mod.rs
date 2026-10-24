//! Shared helpers of tests/judging.rs and tests/refusals_judging.rs: a workspace, hand-built subjects (the way
//! `subject add` stores them), runs inserted straight into the store, and the CLI as a black box.
#![allow(dead_code)]
use assert_cmd::Command;
use personajury::domain::{hash, *};
use personajury::store::Store;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct Ws {
    pub dir: tempfile::TempDir,
    pub store: Store,
}
impl Ws {
    pub fn new() -> Ws {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        Ws { dir, store }
    }
    pub fn root(&self) -> &Path {
        self.dir.path()
    }
    /// Write `files` as the sources of a subject (label = file name), copy the blobs, store the subject. Returns
    /// (version, the source paths by label).
    pub fn subject(
        &self,
        name: &str,
        kind: SubjectKind,
        is_comparator: bool,
        files: &[(&str, &[u8])],
    ) -> (i64, BTreeMap<String, PathBuf>) {
        let src = self.root().join("sources").join(name);
        std::fs::create_dir_all(&src).unwrap();
        let blobs = self.root().join(".personajury/blobs");
        std::fs::create_dir_all(&blobs).unwrap();
        let mut atts = Vec::new();
        let mut paths = BTreeMap::new();
        for (label, bytes) in files {
            let p = src.join(label);
            std::fs::write(&p, bytes).unwrap();
            let h = hash::sha256_hex(bytes);
            std::fs::write(blobs.join(&h), bytes).unwrap();
            let media = if label.ends_with(".json") {
                "aria-json"
            } else if label.ends_with(".png") {
                "image"
            } else if label.ends_with(".diff") {
                "diff"
            } else {
                "file"
            };
            let canon = p.canonicalize().unwrap();
            atts.push(Attachment {
                label: label.to_string(),
                media: media.into(),
                hash: h,
                source_path: canon.to_string_lossy().into(),
            });
            paths.insert(label.to_string(), canon);
        }
        atts.sort_by(|a, b| a.label.cmp(&b.label));
        let s = Subject {
            name: name.into(),
            version: 0,
            kind,
            is_comparator,
            hash: hash::subject_hash(kind, &atts),
            attachments: atts,
        };
        let v = self.store.insert_subject(&s).unwrap();
        (v, paths)
    }
    /// The standard page subject `site` with a walk (nodes n1 > n2), a screenshot and a 5-line diff.
    pub fn site(&self) -> BTreeMap<String, PathBuf> {
        self.subject(
            "site",
            SubjectKind::Page,
            false,
            &[
                (
                    "page.json",
                    br#"{"nodes":[{"id":"n1","children":[{"id":"n2"}]},{"id":"n3"}]}"#,
                ),
                ("shot.png", b"\x89PNG not really"),
                ("change.diff", b"a\nb\nc\nd\ne\n"),
            ],
        )
        .1
    }
    pub fn rival(&self) -> BTreeMap<String, PathBuf> {
        self.subject(
            "rival",
            SubjectKind::Page,
            true,
            &[("rival.json", br#"{"nodes":[{"id":"r1"}]}"#)],
        )
        .1
    }
    /// Insert a run of `site` (latest version) under `lens`, briefing `personas` (all with the given stakes); `extra`
    /// are stakes of personas that were not briefed.
    pub fn run(&self, lens: &str, personas: &[(&str, Stake)], with_rival: bool) -> i64 {
        self.run_of("site", lens, personas, with_rival)
    }
    pub fn run_of(
        &self,
        subject: &str,
        lens: &str,
        personas: &[(&str, Stake)],
        with_rival: bool,
    ) -> i64 {
        let sub = self.store.subject(subject).unwrap().unwrap();
        let l = self.store.lens(lens).unwrap().unwrap();
        let cmp = if with_rival {
            let c = self.store.subject("rival").unwrap().unwrap();
            vec![(c.name, c.version)]
        } else {
            vec![]
        };
        let r = RunRow {
            id: 0,
            subject_name: sub.name.clone(),
            subject_version: sub.version,
            subject_hash: sub.hash.clone(),
            panel_name: "p".into(),
            panel_hash: "ph".into(),
            lens_name: l.name.clone(),
            lens_version: l.version,
            model: "m".into(),
            spend: personas.len() as i64,
            stakes: personas.iter().map(|(p, s)| (p.to_string(), *s)).collect(),
            personas: personas.iter().map(|(p, _)| p.to_string()).collect(),
            comparators: cmp,
            briefs: personas
                .iter()
                .map(|(p, s)| BriefRef {
                    persona: p.to_string(),
                    persona_version: 1,
                    stake: *s,
                    path: format!("briefs/{p}.md"),
                    hash: "bh".into(),
                    bytes: 10,
                    model: "m".into(),
                })
                .collect(),
            created_at: "2026-10-23T00:00:00Z".into(),
            draw_seed: None,
            override_json: None,
            effort: None,
            trimmed: vec![],
            independence: "independent".into(),
            rule: None,
            quorum: None,
        };
        self.store.insert_run(&r).unwrap()
    }
    pub fn cli(&self, args: &[&str]) -> Value {
        cli_in(self.root(), args)
    }
    /// Ingest `v` into `run`, panic unless ok; return data.
    pub fn ingest(&self, run: i64, v: &Value) -> Value {
        let r = self.cli(&[
            "verdict",
            "--run",
            &run.to_string(),
            "--json",
            &v.to_string(),
        ]);
        assert_eq!(r["ok"], true, "{r}");
        r["data"].clone()
    }
}

pub fn cli_in(root: &Path, args: &[&str]) -> Value {
    let out = Command::cargo_bin("personajury")
        .unwrap()
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let v: Value = serde_json::from_str(text.trim()).unwrap_or_else(|e| {
        panic!(
            "not an envelope: {e}\nstdout: {text}\nstderr: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    assert_eq!(
        out.status.success(),
        v["ok"] == true,
        "exit code follows ok: {v}"
    );
    v
}

/// critical-lens lines (works, blocks, missing) with the given scores; `blockers` and `cite` go on the first line.
pub fn critical(persona: &str, scores: [i64; 3], blockers: &[&str], cite: &[&str]) -> Value {
    let dims = ["works", "blocks", "missing"];
    let lines: Vec<Value> = dims
        .iter()
        .zip(scores)
        .enumerate()
        .map(|(i, (d, s))| {
            json!({
                "dimension": d, "score": s, "anchor_met": if s >= 3 { 3 } else { 1 }, "evidence": "seen",
                "citations": if i == 0 { json!(cite) } else { json!(["page.json#n1"]) },
                "blockers": if i == 0 { json!(blockers) } else { json!([]) },
            })
        })
        .collect();
    json!({ "persona": persona, "lines": lines })
}

/// compare-lens lines (clarity, price, trust) plus a pick; each `seen` line cites the subject and the comparator
/// `rival` (a compare line must name or cite a comparator, D6 A2).
pub fn compare(persona: &str, choice: &str, reason: &str) -> Value {
    let lines: Vec<Value> = ["clarity", "price", "trust"]
        .iter()
        .map(|d| json!({"dimension": d, "score": 3, "anchor_met": 3, "evidence": "seen", "citations": ["page.json#n1", "cmp:rival/rival.json#r1"]}))
        .collect();
    json!({ "persona": persona, "lines": lines, "pick": {"choice": choice, "reason": reason} })
}

pub const TWO: [(&str, Stake); 2] = [("ana", Stake::Primary), ("bob", Stake::Secondary)];
/// Two primary seats and one secondary: the smallest jury that may apply (a primary stratum of one is `thin_primary`).
pub const THREE: [(&str, Stake); 3] = [("ana", Stake::Primary), ("cat", Stake::Primary), ("bob", Stake::Secondary)];

/// critical-lens verdict with no citations anywhere and `told` evidence; `blockers` on the first line.
pub fn told(persona: &str, blockers: &[&str]) -> Value {
    let lines: Vec<Value> = ["works", "blocks", "missing"]
        .iter()
        .enumerate()
        .map(|(i, d)| {
            json!({"dimension": d, "score": 2, "anchor_met": 1, "evidence": "told",
                   "blockers": if i == 0 { json!(blockers) } else { json!([]) }})
        })
        .collect();
    json!({ "persona": persona, "lines": lines })
}

pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

// ---- the fixture jury, up to its fourteen accepted verdicts and no decisions (shared by summary and the Action tests) ----
const STAKES: &str = "builder-ana=primary,builder-mei=primary,builder-ravi=primary,newcomer-sofia=primary,\
company-george=secondary,company-hanna=secondary,investor-claire=bystander";

pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
pub fn fx(rel: &str) -> String {
    repo().join("fixtures").join(rel).to_string_lossy().into_owned()
}
pub fn j(ws: &Path, args: &[&str]) -> Value {
    let v = cli_in(ws, args);
    assert_eq!(v["ok"], true, "{args:?} refused: {}", v["error"]);
    v
}
/// The fixture jury, up to its fourteen accepted verdicts and no decisions: returns the two run ids.
pub fn fixture_run(ws: &Path) -> (i64, i64) {
    let mut personas: Vec<_> = std::fs::read_dir(repo().join("fixtures/personas")).unwrap().map(|e| e.unwrap().path()).collect();
    personas.sort();
    j(ws, &["trait", "install"]);
    for p in personas {
        j(ws, &["persona", "add", "--file", p.to_str().unwrap()]);
    }
    j(ws, &["panel", "add", "fixture", "--query", "3 builder, 2 company, 2 others"]);
    let (hw, hp, pw, pp) = (fx("site-walk/home.walk.json"), fx("site-walk/home.png"), fx("site-walk/pricing.walk.json"), fx("site-walk/pricing.png"));
    j(ws, &["subject", "add", "lantern-site", "--kind", "page", "--attach", &hw, &hp, &pw, &pp]);
    let (cw, cp) = (fx("comparators/acme-pricing/pricing.walk.json"), fx("comparators/acme-pricing/pricing.png"));
    j(ws, &["comparator", "add", "acme-pricing", "--kind", "prices", "--attach", &cw, &cp]);
    let common = ["--subject", "lantern-site", "--panel", "fixture", "--stake", STAKES, "--spend", "7", "--model", "claude-sonnet-5-5"];
    let mut critical = vec!["brief", "--lens", "critical"];
    critical.extend(common);
    // recorded before D32: the recorded verdicts score `about: pricing.png`, so this brief keeps the attachment options
    let mut compare = vec!["brief", "--lens", "compare", "--comparator", "acme-pricing", "--options", "attachments"];
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
    (a, b)
}
