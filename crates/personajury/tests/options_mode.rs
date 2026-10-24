//! `brief --options subject|attachments` (D32): what a juror may score `about` and `pick`, by the subject's kind.
#[path = "traits_support/mod.rs"]
mod support;
use serde_json::{json, Value};
use std::path::Path;
use support::*;

fn world() -> tempfile::TempDir {
    let ws = installed();
    let w = ws.path();
    for id in ["p1", "p2"] {
        add_ok(w, id, &["x"], Value::Null);
    }
    let src = w.join("src");
    std::fs::create_dir_all(&src).unwrap();
    for f in ["a.md", "b.md", "c.md"] {
        std::fs::write(src.join(f), format!("# {f}\nline two\n")).unwrap();
    }
    let a = src.join("a.md").to_string_lossy().to_string();
    let b = src.join("b.md").to_string_lossy().to_string();
    let c = src.join("c.md").to_string_lossy().to_string();
    ok(jg(w, &["subject", "add", "pg", "--kind", "page", "--attach", &a, &b]));
    ok(jg(w, &["subject", "add", "lg", "--kind", "logo", "--attach", &a, &b]));
    ok(jg(w, &["comparator", "add", "rv", "--kind", "page", "--attach", &c]));
    ws
}
fn brief(w: &Path, subject: &str, extra: &[&str]) -> Value {
    let mut a = vec![
        "brief", "--subject", subject, "--query", "2 x=primary", "--lens", "compare", "--comparator", "rv", "--model", "m",
    ];
    a.extend(extra);
    jg(w, &a)
}
fn text(w: &Path, d: &Value) -> String {
    std::fs::read_to_string(w.join(d["briefs"][0]["path"].as_str().unwrap())).unwrap()
}
/// The bullets of the brief's `## Options` section.
fn options(t: &str) -> Vec<String> {
    let start = t.find("## Options").expect("an Options section");
    let rest = &t[start..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |i| i + 3);
    rest[..end].lines().filter_map(|l| l.strip_prefix("- `").and_then(|x| x.strip_suffix('`'))).map(String::from).collect()
}
fn verdict(persona: &str, about: &[Option<&str>], pick: &str) -> Value {
    let mut lines = Vec::new();
    for d in ["clarity", "price", "trust"] {
        for a in about {
            let mut l = json!({"dimension": d, "score": 3, "anchor_met": 3, "evidence": "told"});
            if let Some(a) = a {
                l["about"] = json!(a);
            }
            lines.push(l);
        }
    }
    json!({"persona": persona, "lines": lines, "pick": {"choice": pick, "reason": "because"}})
}
fn ingest(w: &Path, run: i64, v: &Value) -> Value {
    jg(w, &["verdict", "--run", &run.to_string(), "--json", &v.to_string()])
}

#[test]
fn a_page_briefs_with_the_subject_and_the_comparators_not_its_attachments() {
    let ws = world();
    let d = ok(brief(ws.path(), "pg", &[]));
    assert_eq!(options(&text(ws.path(), &d)), vec!["pg", "cmp:rv"]);
    assert_eq!(d["options"]["mode"], "subject");
    assert_eq!(d["options"]["list"], json!(["pg", "cmp:rv"]));
}

#[test]
fn a_logo_lists_its_boards_as_candidates() {
    let ws = world();
    let d = ok(brief(ws.path(), "lg", &[]));
    assert_eq!(options(&text(ws.path(), &d)), vec!["a.md", "b.md", "cmp:rv"]);
    assert_eq!(d["options"]["mode"], "attachments");
}

#[test]
fn options_attachments_on_a_page_is_accepted_and_recorded() {
    let ws = world();
    let d = ok(brief(ws.path(), "pg", &["--options", "attachments"]));
    assert_eq!(options(&text(ws.path(), &d)), vec!["a.md", "b.md", "cmp:rv"]);
    let run = d["run"].as_i64().unwrap();
    let st = personajury::store::Store::open(ws.path()).unwrap();
    assert_eq!(st.run_options_mode(run).unwrap().as_deref(), Some("attachments"));
    // and a juror may score `about` a board of the page
    let r = ingest(ws.path(), run, &verdict("p1", &[Some("a.md"), Some("b.md")], "a.md"));
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn options_subject_on_a_logo_is_accepted() {
    let ws = world();
    let d = ok(brief(ws.path(), "lg", &["--options", "subject"]));
    assert_eq!(options(&text(ws.path(), &d)), vec!["lg", "cmp:rv"]);
}

#[test]
fn an_unknown_options_word_is_refused() {
    let ws = world();
    let r = brief(ws.path(), "pg", &["--options", "everything"]);
    assert_eq!(code(&r), "invalid_options_mode");
    assert_eq!(r["data"]["given"], "everything");
}

#[test]
fn about_the_subject_name_is_stored_as_the_whole_subject() {
    let ws = world();
    let d = ok(brief(ws.path(), "pg", &[]));
    let run = d["run"].as_i64().unwrap();
    let r = ingest(ws.path(), run, &verdict("p1", &[Some("pg")], "pg"));
    assert_eq!(r["ok"], true, "{r}");
    let m = ok(jg(ws.path(), &["matrix", &run.to_string(), "--detail", "full"]));
    let cells = m["strata"][0]["cells"].as_array().unwrap();
    assert_eq!(cells.len(), 3);
    assert!(cells.iter().all(|c| c["about"].is_null()), "{cells:?}");
}

#[test]
fn both_forms_of_the_subject_on_one_dimension_are_a_duplicate() {
    let ws = world();
    let run = ok(brief(ws.path(), "pg", &[]))["run"].as_i64().unwrap();
    let r = ingest(ws.path(), run, &verdict("p1", &[Some("pg"), None], "pg"));
    assert_eq!(code(&r), "duplicate_dimension");
}

#[test]
fn an_attachment_label_is_unknown_about_on_a_subject_mode_run() {
    let ws = world();
    let run = ok(brief(ws.path(), "pg", &[]))["run"].as_i64().unwrap();
    let r = ingest(ws.path(), run, &verdict("p1", &[Some("a.md")], "pg"));
    assert_eq!(code(&r), "unknown_about");
}

#[test]
fn a_pick_of_the_subject_name_is_accepted_and_an_attachment_is_not() {
    let ws = world();
    let run = ok(brief(ws.path(), "pg", &[]))["run"].as_i64().unwrap();
    let r = ingest(ws.path(), run, &verdict("p1", &[None], "a.md"));
    assert_eq!(code(&r), "pick_unknown_option");
    let r = ingest(ws.path(), run, &verdict("p1", &[None], "pg"));
    assert_eq!(r["ok"], true, "{r}");
    let p = ok(jg(ws.path(), &["pick", &run.to_string()]));
    assert_eq!(p["strata"][0]["choices"][0]["choice"], "pg");
}

#[test]
fn a_run_with_no_recorded_mode_keeps_the_attachment_options_it_always_had() {
    let ws = world();
    let run = ok(brief(ws.path(), "pg", &[]))["run"].as_i64().unwrap();
    let st = personajury::store::Store::open(ws.path()).unwrap();
    st.conn().execute("UPDATE run SET options_mode = NULL", []).unwrap();
    let r = ingest(ws.path(), run, &verdict("p1", &[Some("a.md")], "a.md"));
    assert_eq!(r["ok"], true, "{r}");
}

/// M1 (v0.6-C review): a subject briefed before keeps its latest run's mode when `--options` is left out, so the next
/// round's cells meet the previous round's; a run made before v0.6 (no recorded mode) counts as `attachments`.
#[test]
fn a_subject_briefed_before_keeps_its_previous_mode_by_default() {
    let ws = world();
    let first = ok(brief(ws.path(), "pg", &["--options", "attachments"]));
    assert_eq!(first["options"]["mode"], "attachments");
    let again = ok(brief(ws.path(), "pg", &[]));
    assert_eq!(again["options"]["mode"], "attachments", "{again}");
    // a pre-v0.6 run, its mode NULL, is attachments too
    let st = personajury::store::Store::open(ws.path()).unwrap();
    st.conn().execute("UPDATE run SET options_mode = NULL", []).unwrap();
    assert_eq!(ok(brief(ws.path(), "pg", &[]))["options"]["mode"], "attachments");
    // an explicit word still wins, and a subject never briefed takes its kind's default
    assert_eq!(ok(brief(ws.path(), "pg", &["--options", "subject"]))["options"]["mode"], "subject");
    assert_eq!(ok(brief(ws.path(), "pg", &[]))["options"]["mode"], "subject");
}
