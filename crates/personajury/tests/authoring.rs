//! Authoring verbs through the binary: persona, panel, lens, subject, comparator, brief (plan Task 4).
use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn fx(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(rel)
}
pub fn jg(ws: &Path, args: &[&str]) -> Value {
    let out = Command::cargo_bin("personajury")
        .unwrap()
        .arg("--root")
        .arg(ws)
        .args(args)
        .output()
        .unwrap();
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("not json ({e}): {}", String::from_utf8_lossy(&out.stdout)))
}
fn ok(v: Value) -> Value {
    assert_eq!(v["ok"], true, "{v}");
    v["data"].clone()
}
fn s(p: PathBuf) -> String {
    p.to_str().unwrap().to_string()
}
const PERSONAS: [&str; 7] = [
    "builder-ana",
    "builder-mei",
    "builder-ravi",
    "company-george",
    "company-hanna",
    "investor-claire",
    "newcomer-sofia",
];
fn add_personas(ws: &Path) {
    for p in PERSONAS {
        let f = s(fx(&format!("personas/{p}.json")));
        ok(jg(ws, &["persona", "add", "--file", &f]));
    }
}
fn add_pricing(ws: &Path) {
    let a = s(fx("site-walk/pricing.walk.json"));
    let b = s(fx("site-walk/pricing.png"));
    ok(jg(
        ws,
        &[
            "subject", "add", "pricing", "--kind", "page", "--attach", &a, &b,
        ],
    ));
}
fn setup(ws: &Path) {
    add_personas(ws);
    add_pricing(ws);
    ok(jg(
        ws,
        &[
            "panel",
            "add",
            "wide",
            "--query",
            "3 builder, 2 company, 1 investor, 1 newcomer",
        ],
    ));
}
const STAKES: &str = "builder-ana=primary,builder-mei=primary,builder-ravi=secondary,company-george=secondary,company-hanna=bystander,investor-claire=bystander,newcomer-sofia=primary";

fn brief(ws: &Path, extra: &[&str]) -> Value {
    let mut a = vec![
        "brief",
        "--subject",
        "pricing",
        "--panel",
        "wide",
        "--lens",
        "critical",
        "--stake",
        STAKES,
        "--model",
        "test-model",
    ];
    a.extend_from_slice(extra);
    jg(ws, &a)
}
fn read(ws: &Path, rel: &str) -> String {
    std::fs::read_to_string(ws.join(rel)).unwrap()
}

#[test]
fn persona_add_then_list() {
    let ws = tempfile::tempdir().unwrap();
    let f = s(fx("personas/builder-ana.json"));
    let d = ok(jg(ws.path(), &["persona", "add", "--file", &f]));
    assert_eq!(d["id"], "builder-ana");
    assert_eq!(d["version"], 1);
    assert_eq!(d["origin"], "hand");
    assert_eq!(d["hash"].as_str().unwrap().len(), 64);
    let again = ok(jg(ws.path(), &["persona", "add", "--file", &f]));
    assert_eq!(again["version"], 1);
    assert_eq!(again["hash"], d["hash"]);
    let l = ok(jg(ws.path(), &["persona", "list"]));
    let items = l["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"], "Ana Costa");
    assert_eq!(items[0]["tags"][0], "builder");
    // a malformed file is named in the error
    let bad = ws.path().join("bad.json");
    std::fs::write(&bad, r#"{"id":"x","surprise":1}"#).unwrap();
    let e = jg(
        ws.path(),
        &["persona", "add", "--file", bad.to_str().unwrap()],
    );
    assert_eq!(e["error"]["code"], "invalid_arguments");
    assert!(e["error"]["message"].as_str().unwrap().contains("bad.json"));
    // generated-from an unknown material is not found
    let e = jg(
        ws.path(),
        &["persona", "add", "--file", &f, "--generated-from", "00ff"],
    );
    assert_eq!(e["error"]["code"], "not_found");
}

#[test]
fn generate_same_material_same_hash() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let src = s(fx("personas"));
    let x = ok(jg(
        a.path(),
        &["persona", "generate", "--from", &src, "--count", "3"],
    ));
    let y = ok(jg(
        b.path(),
        &["persona", "generate", "--from", &src, "--count", "3"],
    ));
    assert_eq!(x["material_hash"], y["material_hash"]);
    let hash = x["material_hash"].as_str().unwrap();
    let p = format!(".personajury/generate/{hash}.md");
    assert_eq!(read(a.path(), &p), read(b.path(), &p));
    assert!(read(a.path(), &p).contains("persona add --file"));
    assert_eq!(x["sources"].as_array().unwrap().len(), 7);
    // the recorded material lets a persona carry its origin
    let f = s(fx("personas/builder-ana.json"));
    let d = ok(jg(
        a.path(),
        &["persona", "add", "--file", &f, "--generated-from", hash],
    ));
    assert_eq!(d["origin"], format!("generated-from:{hash}"));
    // different material, different hash
    let z = ok(jg(
        a.path(),
        &["persona", "generate", "--from", &f, "--count", "3"],
    ));
    assert_ne!(z["material_hash"], x["material_hash"]);
}

#[test]
fn panel_resolve_by_query() {
    let ws = tempfile::tempdir().unwrap();
    add_personas(ws.path());
    ok(jg(
        ws.path(),
        &[
            "panel",
            "add",
            "p",
            "--query",
            "2 builder, 1 company, 9 others",
        ],
    ));
    let r = ok(jg(ws.path(), &["panel", "resolve", "p"]));
    let ids: Vec<&str> = r["personas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "builder-ana",
            "builder-mei",
            "company-george",
            "investor-claire",
            "newcomer-sofia"
        ]
    );
    assert_eq!(r["terms"][0]["tag"], "builder");
    assert_eq!(r["terms"][0]["wanted"], 2);
    assert_eq!(r["terms"][0]["got"], 2);
    assert_eq!(r["terms"][2]["wanted"], 9);
    assert_eq!(r["terms"][2]["got"], 2);
    assert_eq!(r["hash"].as_str().unwrap().len(), 64);
    let e = jg(
        ws.path(),
        &["panel", "add", "q", "--query", "lots of builders"],
    );
    assert_eq!(e["error"]["code"], "invalid_query");
    let e = jg(ws.path(), &["panel", "resolve", "nope"]);
    assert_eq!(e["error"]["code"], "not_found");
}

#[test]
fn lens_add_and_list() {
    let ws = tempfile::tempdir().unwrap();
    let f = ws.path().join("mine.toml");
    std::fs::write(
        &f,
        std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lenses/recall.toml"),
        )
        .unwrap()
        .replace("name = \"recall\"", "name = \"mine\""),
    )
    .unwrap();
    let d = ok(jg(
        ws.path(),
        &["lens", "add", "--file", f.to_str().unwrap()],
    ));
    assert_eq!(d["name"], "mine");
    assert_eq!(d["version"], 1);
    let l = ok(jg(ws.path(), &["lens", "list"]));
    let names: Vec<&str> = l["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["compare", "critical", "mine", "recall", "taste"]);
    let cmp = &l["items"][0];
    assert_eq!(cmp["asks_pick"], true);
    assert_eq!(cmp["dimensions"][0], "clarity");
    std::fs::write(&f, "not = [toml").unwrap();
    let e = jg(ws.path(), &["lens", "add", "--file", f.to_str().unwrap()]);
    assert_eq!(e["error"]["code"], "invalid_arguments");
}

#[test]
fn site_lens_loads_with_lens_add() {
    let ws = tempfile::tempdir().unwrap();
    let f = s(fx("lenses/site.toml"));
    let d = ok(jg(ws.path(), &["lens", "add", "--file", &f]));
    assert_eq!(
        (d["name"].as_str(), d["version"].as_i64()),
        (Some("site"), Some(1))
    );
    let store = personajury::store::Store::open(ws.path()).unwrap();
    let l = store.lens("site").unwrap().unwrap();
    assert_eq!(l.scale, [1, 10]);
    assert_eq!(l.anchor_scores(), vec![1, 4, 7, 10]);
    assert_eq!(l.anchor_step(), 3);
    assert!(!l.compares && !l.asks_pick);
    let dims: Vec<&str> = l.dimensions.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(
        dims,
        [
            "craft",
            "clarity-5s",
            "next-action",
            "pricing",
            "distinct",
            "phone"
        ]
    );
    for d in &l.dimensions {
        assert_eq!(d.anchors.len(), 4, "{}", d.id);
    }
    // fixture data, not seeded: a fresh store does not hold it
    let fresh = tempfile::tempdir().unwrap();
    let names: Vec<String> = personajury::store::Store::open(fresh.path())
        .unwrap()
        .lenses()
        .unwrap()
        .into_iter()
        .map(|l| l.name)
        .collect();
    assert!(!names.contains(&"site".to_string()), "{names:?}");
}

#[test]
fn subject_add_copies_blobs() {
    let ws = tempfile::tempdir().unwrap();
    let a = s(fx("site-walk/pricing.walk.json"));
    let b = s(fx("site-walk/pricing.png"));
    let d = ok(jg(
        ws.path(),
        &[
            "subject", "add", "pricing", "--kind", "page", "--attach", &b, &a,
        ],
    ));
    assert_eq!(d["kind"], "page");
    assert_eq!(d["comparator"], false);
    assert_eq!(d["version"], 1);
    let at = d["attachments"].as_array().unwrap();
    assert_eq!(at[0]["label"], "pricing.png");
    assert_eq!(at[0]["media"], "image");
    assert_eq!(at[1]["label"], "pricing.walk.json");
    assert_eq!(at[1]["media"], "aria-json");
    for x in at {
        let h = x["hash"].as_str().unwrap();
        assert!(ws.path().join(".personajury/blobs").join(h).is_file());
    }
    let png = std::fs::read(fx("site-walk/pricing.png")).unwrap();
    let h = at[0]["hash"].as_str().unwrap();
    assert_eq!(
        std::fs::read(ws.path().join(".personajury/blobs").join(h)).unwrap(),
        png
    );
    // same files, same version and hash
    let again = ok(jg(
        ws.path(),
        &[
            "subject", "add", "pricing", "--kind", "page", "--attach", &a, &b,
        ],
    ));
    assert_eq!(again["version"], 1);
    assert_eq!(again["hash"], d["hash"]);
    // a directory is not a file; duplicate labels refused; a name cannot change flavour
    let e = jg(
        ws.path(),
        &[
            "subject",
            "add",
            "x",
            "--kind",
            "page",
            "--attach",
            ws.path().to_str().unwrap(),
        ],
    );
    assert_eq!(e["error"]["code"], "invalid_arguments");
    let e = jg(
        ws.path(),
        &["subject", "add", "x", "--kind", "page", "--attach", &a, &a],
    );
    assert_eq!(e["error"]["code"], "invalid_arguments");
    let e = jg(
        ws.path(),
        &[
            "comparator",
            "add",
            "pricing",
            "--kind",
            "page",
            "--attach",
            &a,
        ],
    );
    assert_eq!(e["error"]["code"], "invalid_arguments");
}

#[test]
fn brief_is_deterministic() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    setup(a.path());
    setup(b.path());
    let x = ok(brief(a.path(), &[]));
    let y = ok(brief(b.path(), &[]));
    assert_eq!(x["briefs"].as_array().unwrap().len(), 7);
    for (p, q) in x["briefs"]
        .as_array()
        .unwrap()
        .iter()
        .zip(y["briefs"].as_array().unwrap())
    {
        assert_eq!(p["hash"], q["hash"]);
        assert_eq!(p["path"], q["path"]);
        assert_eq!(
            read(a.path(), p["path"].as_str().unwrap()),
            read(b.path(), q["path"].as_str().unwrap())
        );
    }
    assert_eq!(x["bytes_total"], y["bytes_total"]);
    let first = read(a.path(), x["briefs"][0]["path"].as_str().unwrap());
    assert!(first.starts_with("personajury-brief 1\n"));
    // no model name, no run id in the text
    assert!(!first.contains("test-model"));
    // a second brief in the same workspace is the same bytes under a new run
    let z = ok(brief(a.path(), &[]));
    assert_ne!(z["run"], x["run"]);
    assert_eq!(z["briefs"][0]["hash"], x["briefs"][0]["hash"]);
}

#[test]
fn brief_orders_by_stake() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    let d = ok(brief(ws.path(), &["--spend", "4"]));
    let got: Vec<(&str, &str)> = d["briefs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| (b["persona"].as_str().unwrap(), b["stake"].as_str().unwrap()))
        .collect();
    assert_eq!(
        got,
        [
            ("builder-ana", "primary"),
            ("builder-mei", "primary"),
            ("newcomer-sofia", "primary"),
            ("builder-ravi", "secondary")
        ]
    );
    assert_eq!(d["spend"], 4);
    let p = d["briefs"][0]["path"].as_str().unwrap();
    assert_eq!(
        p,
        format!(".personajury/runs/{}/builder-ana--critical.md", d["run"])
    );
    let bytes = read(ws.path(), p).len();
    assert_eq!(d["briefs"][0]["bytes"], bytes);
}

#[test]
fn brief_pins_persona_version() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    let one = ok(brief(ws.path(), &[]));
    let run1 = one["run"].as_i64().unwrap();
    // change Ana: a second version
    let mut v: Value =
        serde_json::from_str(&std::fs::read_to_string(fx("personas/builder-ana.json")).unwrap())
            .unwrap();
    v["fears"] = Value::String("Nothing at all.".into());
    let f = ws.path().join("ana2.json");
    std::fs::write(&f, v.to_string()).unwrap();
    let d = ok(jg(
        ws.path(),
        &["persona", "add", "--file", f.to_str().unwrap()],
    ));
    assert_eq!(d["version"], 2);
    let two = ok(brief(ws.path(), &[]));
    assert_ne!(two["briefs"][0]["hash"], one["briefs"][0]["hash"]);
    assert!(read(ws.path(), two["briefs"][0]["path"].as_str().unwrap()).contains("Nothing at all."));
    // the first run's files are untouched and its refs still hold version 1
    assert!(
        read(ws.path(), one["briefs"][0]["path"].as_str().unwrap()).contains("Paying for a seat")
    );
    let store = personajury::store::Store::open(ws.path()).unwrap();
    assert_eq!(
        store.run(run1).unwrap().unwrap().briefs[0].persona_version,
        1
    );
    assert_eq!(
        store
            .run(two["run"].as_i64().unwrap())
            .unwrap()
            .unwrap()
            .briefs[0]
            .persona_version,
        2
    );
}

/// The JSON example of a brief, ingested as the verdict it shows: the example obeys the rules it states.
fn ingest_example(ws: &Path, d: &Value) -> Value {
    let text = read(ws, d["briefs"][0]["path"].as_str().unwrap());
    let start = text.find("```json\n").expect("an example") + "```json\n".len();
    let end = start + text[start..].find("\n```").unwrap();
    let run = d["run"].as_i64().unwrap().to_string();
    jg(ws, &["verdict", "--run", &run, "--json", &text[start..end]])
}

fn add_acme(ws: &Path) {
    let w = s(fx("comparators/acme-pricing/pricing.walk.json"));
    let h = s(fx("comparators/acme-pricing/pricing.html"));
    ok(jg(
        ws,
        &[
            "comparator",
            "add",
            "acme",
            "--kind",
            "page",
            "--attach",
            &w,
            &h,
        ],
    ));
}

#[test]
fn brief_describes_about_and_options_only_under_pick_or_compare() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    add_acme(ws.path());
    let d = ok(brief(ws.path(), &["--comparator", "acme"]));
    let text = read(ws.path(), d["briefs"][0]["path"].as_str().unwrap());
    assert!(
        !text.contains("\n## Options\n"),
        "critical neither picks nor compares: {text}"
    );
    assert!(
        text.contains("One line per dimension: this lens scores the subject as a whole"),
        "{text}"
    );
    assert!(
        text.contains("bottom of the scale (1) needs a blocker"),
        "{text}"
    );
    assert!(
        !text.contains("must be `about` a comparator"),
        "critical does not compare"
    );
    let c = ok(brief_cmp(ws.path(), "acme"));
    let text = read(ws.path(), c["briefs"][0]["path"].as_str().unwrap());
    for o in ["- `pricing.png`", "- `pricing.walk.json`", "- `cmp:acme`"] {
        assert!(text.contains(o), "{o}");
    }
    assert!(text.contains("One line per dimension and option"), "{text}");
    assert!(text.contains("must be `about` a comparator"), "{text}");
}

#[test]
fn brief_example_is_a_valid_verdict() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    add_acme(ws.path());
    let crit = ok(brief(ws.path(), &[]));
    assert_eq!(ingest_example(ws.path(), &crit)["ok"], true);
    let crit_cmp = ok(brief(ws.path(), &["--comparator", "acme"]));
    let r = ingest_example(ws.path(), &crit_cmp);
    assert_eq!(r["ok"], true, "{r}");
    let cmp = ok(brief_cmp(ws.path(), "acme"));
    let r = ingest_example(ws.path(), &cmp);
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn brief_lists_comparator_snapshots() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    let w = s(fx("comparators/acme-pricing/pricing.walk.json"));
    let h = s(fx("comparators/acme-pricing/pricing.html"));
    let c = ok(jg(
        ws.path(),
        &[
            "comparator",
            "add",
            "acme",
            "--kind",
            "page",
            "--attach",
            &w,
            &h,
        ],
    ));
    assert_eq!(c["comparator"], true);
    // a comparator is not a subject
    let e = jg(
        ws.path(),
        &[
            "brief",
            "--subject",
            "acme",
            "--panel",
            "wide",
            "--lens",
            "compare",
            "--stake",
            STAKES,
            "--model",
            "m",
        ],
    );
    assert_eq!(e["error"]["code"], "invalid_arguments");
    // a subject is not a comparator
    let e = brief_cmp(ws.path(), "pricing");
    assert_eq!(e["error"]["code"], "invalid_arguments");
    let d = ok(brief_cmp(ws.path(), "acme"));
    let text = read(ws.path(), d["briefs"][0]["path"].as_str().unwrap());
    assert!(text.contains("cmp:acme/pricing.walk.json"), "{text}");
    assert!(text.contains("cmp:acme/pricing.html"));
    assert!(text.contains("cmp:acme"));
    // the run holds the snapshot
    let store = personajury::store::Store::open(ws.path()).unwrap();
    let run = store.run(d["run"].as_i64().unwrap()).unwrap().unwrap();
    assert_eq!(run.comparators, vec![("acme".to_string(), 1)]);
    // duplicates are removed
    let dup = ok(jg(
        ws.path(),
        &[
            "brief",
            "--subject",
            "pricing",
            "--panel",
            "wide",
            "--lens",
            "compare",
            "--stake",
            STAKES,
            "--model",
            "m",
            "--comparator",
            "acme",
            "--comparator",
            "acme",
        ],
    ));
    let run = store.run(dup["run"].as_i64().unwrap()).unwrap().unwrap();
    assert_eq!(run.comparators.len(), 1);
}
fn brief_cmp(ws: &Path, c: &str) -> Value {
    jg(
        ws,
        &[
            "brief",
            "--subject",
            "pricing",
            "--panel",
            "wide",
            "--lens",
            "compare",
            "--stake",
            STAKES,
            "--model",
            "m",
            "--comparator",
            c,
        ],
    )
}

#[test]
fn brief_full_inlines_walk_compact_does_not() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    let compact = ok(brief(ws.path(), &["--spend", "1"]));
    let full = ok(brief(ws.path(), &["--spend", "1", "--detail", "full"]));
    let c = read(ws.path(), compact["briefs"][0]["path"].as_str().unwrap());
    let f = read(ws.path(), full["briefs"][0]["path"].as_str().unwrap());
    let walk: Value =
        serde_json::from_str(&std::fs::read_to_string(fx("site-walk/pricing.walk.json")).unwrap())
            .unwrap();
    let id = walk["nodes"][0]["id"].as_str().unwrap();
    let name_line = format!("{id} {}", walk["nodes"][0]["role"].as_str().unwrap());
    assert!(!c.contains(&name_line), "compact must not inline");
    assert!(f.contains(&name_line), "full inlines the walk");
    assert!(f.len() > c.len());
    assert!(f.contains("\n  "));
    assert_eq!(compact["detail"], "compact");
    assert_eq!(full["detail"], "full");
    assert!(full["bytes_total"].as_u64() > compact["bytes_total"].as_u64());
    // the image stays by path in both
    assert!(f.contains(".personajury/blobs/") && f.contains("pricing.png"));
}

#[test]
fn brief_full_truncates_at_cap() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    let big = ws.path().join("big.txt");
    // multi-byte chars: the cut must land on a char boundary
    std::fs::write(&big, "é".repeat(40_000)).unwrap();
    let small = ws.path().join("small.txt");
    std::fs::write(&small, "tiny text").unwrap();
    ok(jg(
        ws.path(),
        &[
            "subject",
            "add",
            "doc",
            "--kind",
            "document",
            "--attach",
            big.to_str().unwrap(),
            small.to_str().unwrap(),
        ],
    ));
    let args = |d: &'static str| {
        [
            "brief",
            "--subject",
            "doc",
            "--panel",
            "wide",
            "--lens",
            "critical",
            "--stake",
            STAKES,
            "--model",
            "m",
            "--spend",
            "1",
            "--detail",
            d,
        ]
    };
    let full = ok(jg(ws.path(), &args("full")));
    let t = read(ws.path(), full["briefs"][0]["path"].as_str().unwrap());
    assert_eq!(t.matches("[truncated at 65536 bytes]").count(), 1);
    assert!(t.contains("tiny text"));
    assert_eq!(t.matches('é').count(), 65536 / 2);
    let compact = ok(jg(ws.path(), &args("compact")));
    let c = read(ws.path(), compact["briefs"][0]["path"].as_str().unwrap());
    assert!(!c.contains("truncated") && !c.contains("tiny text"));
}

#[test]
fn compare_brief_example_scores_subject_without_about() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    add_acme(ws.path());
    let d = ok(brief_cmp(ws.path(), "acme"));
    let text = read(ws.path(), d["briefs"][0]["path"].as_str().unwrap());
    assert!(text.contains("Never put a comparator in `about`"), "{text}");
    let start = text.find("```json\n").expect("an example") + "```json\n".len();
    let end = start + text[start..].find("\n```").unwrap();
    let ex: Value = serde_json::from_str(&text[start..end]).unwrap();
    let lines = ex["lines"].as_array().unwrap();
    let mut dims: Vec<&str> = lines
        .iter()
        .map(|l| l["dimension"].as_str().unwrap())
        .collect();
    dims.sort();
    dims.dedup();
    assert!(!dims.is_empty());
    for dim in dims {
        let bare: Vec<&Value> = lines
            .iter()
            .filter(|l| l["dimension"] == dim && l.get("about").is_none())
            .collect();
        assert_eq!(bare.len(), 1, "{dim}: one line without about");
        let cites = bare[0]["citations"].as_array().unwrap();
        assert!(
            cites
                .iter()
                .any(|c| c.as_str().unwrap().starts_with("cmp:")),
            "{dim}: cites a comparator: {cites:?}"
        );
    }
    let abouts = lines
        .iter()
        .filter(|l| l["about"].as_str().is_some_and(|a| a.starts_with("cmp:")))
        .count();
    assert_eq!(abouts, 1, "exactly one line about a comparator");
    let r = ingest_example(ws.path(), &d);
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn critical_brief_has_no_options() {
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    add_acme(ws.path());
    let d = ok(brief(ws.path(), &["--comparator", "acme"]));
    let text = read(ws.path(), d["briefs"][0]["path"].as_str().unwrap());
    assert!(!text.contains("## Options"), "{text}");
    // the rules list still says, once, to leave `about` out; nothing else may mention it
    assert!(!text.contains("`about` names"), "{text}");
    assert_eq!(text.matches("`about`").count(), 1, "{text}");
    assert!(
        text.contains("this lens scores the subject as a whole; leave `about` out"),
        "{text}"
    );
}

// sha256 of the compare brief for the pricing/acme fixture, recorded before `about` was scoped to pick/compare lenses;
// re-recorded in v0.3 (D12), when the brief began to state the allowed anchor_met values;
// re-recorded in v0.5.0 (D25), restored in v0.5.1 (D28), when the brief text began to carry the product name PersonaJury
const COMPARE_BRIEF_SHA256: &str =
    "b2e3ae360c90ecddc41f0e5a84e898996a97e3d336c5057d7264f5f9e9b1df58";

#[test]
fn compare_brief_unchanged_by_about_scope() {
    use sha2::{Digest, Sha256};
    let ws = tempfile::tempdir().unwrap();
    setup(ws.path());
    add_acme(ws.path());
    let d = ok(brief_cmp(ws.path(), "acme"));
    let text = read(ws.path(), d["briefs"][0]["path"].as_str().unwrap());
    let got = format!("{:x}", Sha256::digest(text.as_bytes()));
    assert_eq!(got, COMPARE_BRIEF_SHA256);
}
