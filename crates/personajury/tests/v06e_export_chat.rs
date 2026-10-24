//! v0.6 lane E (D40, D41): `export` (raw verdict rows as ndjson or csv, never a derived number) and the chat pack
//! (`brief --chat-pack`, `verdict --from-chat`). Every test here was seen red first.
mod judging_support;
use assert_cmd::Command;
use judging_support::*;
use personajury::domain::Stake;
use serde_json::{json, Value};
use std::path::Path;

// ---------------------------------------------------------------------------------------------------------------
// export
// ---------------------------------------------------------------------------------------------------------------

/// Run the CLI and return (stdout, exit success). Raw: `export` without `--out` prints rows, not an envelope.
fn raw(root: &Path, args: &[&str]) -> (String, bool) {
    let out = Command::cargo_bin("personajury").unwrap().arg("--root").arg(root).args(args).output().unwrap();
    (String::from_utf8(out.stdout).unwrap(), out.status.success())
}
fn lines(text: &str) -> Vec<Value> {
    text.lines().map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("not json ({e}): {l}"))).collect()
}
fn code(v: &Value) -> String {
    assert_eq!(v["ok"], false, "{v}");
    v["error"]["code"].as_str().unwrap().to_string()
}
fn add_persona(w: &Path, id: &str, name: &str, traits: Value) {
    let mut body = json!({
        "id": id, "name": name, "tags": ["x"], "role": "r", "context": "c", "daily_tasks": "d", "fears": "f",
        "never_reads": "n", "decides_by": "b", "stake_rules": "s"
    });
    if !traits.is_null() {
        body["traits"] = traits;
    }
    let f = w.join(format!("{id}.persona.json"));
    std::fs::write(&f, body.to_string()).unwrap();
    let r = cli_in(w, &["persona", "add", "--file", f.to_str().unwrap()]);
    assert_eq!(r["ok"], true, "{r}");
}
/// ana and bob carry traits, cara none; ana and cara are primary, bob secondary. Two runs of `critical` on the site.
/// Run 1: bob, then cara, then ana answer (not in id order), then a person's verdict on ana's seat with declared traits.
/// Run 2: ana only.
fn world() -> Ws {
    let ws = Ws::new();
    ws.site();
    assert_eq!(ws.cli(&["trait", "install"])["ok"], true);
    add_persona(ws.root(), "ana", "Ana Alpha", json!({"expertise.domain": 4, "choice.decision_style": "rational"}));
    add_persona(ws.root(), "bob", "Bob Beta", json!({"expertise.domain": 2}));
    add_persona(ws.root(), "cara", "Cara Gamma", Value::Null);
    let seats = [("ana", Stake::Primary), ("cara", Stake::Primary), ("bob", Stake::Secondary)];
    let run = ws.run("critical", &seats, false);
    assert_eq!(run, 1);
    ws.ingest(run, &critical("bob", [3, 3, 3], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("cara", [2, 3, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("ana", [4, 2, 5], &["the price is hidden"], &["page.json#n1", "page.json#n3"]));
    let human = ws.root().join("human.json");
    std::fs::write(&human, critical("ana", [2, 2, 2], &[], &["page.json#n1"]).to_string()).unwrap();
    let t = ws.root().join("human-traits.json");
    std::fs::write(&t, json!({"expertise.domain": 1}).to_string()).unwrap();
    let r = ws.cli(&[
        "verdict", "--run", "1", "--file", human.to_str().unwrap(), "--by-human", "Dana Roe, Senior Buyer",
        "--traits", t.to_str().unwrap(),
    ]);
    assert_eq!(r["ok"], true, "{r}");
    let run2 = ws.run("critical", &[("ana", Stake::Primary)], false);
    assert_eq!(run2, 2);
    ws.ingest(run2, &critical("ana", [5, 5, 5], &[], &["page.json#n1"]));
    ws
}

#[test]
fn ndjson_is_one_row_per_verdict_line_in_a_fixed_order() {
    let ws = world();
    let (text, ok) = raw(ws.root(), &["export", "1"]);
    assert!(ok, "{text}");
    let rows = lines(&text);
    assert_eq!(rows.len(), 12, "3 personas and one person, 3 lines each:\n{text}");
    // primary before secondary; inside a stratum by persona; inside a persona by dimension
    let seq: Vec<(String, String, String)> = rows
        .iter()
        .map(|r| (r["stratum"].as_str().unwrap().into(), r["persona"].as_str().unwrap().into(), r["dimension"].as_str().unwrap().into()))
        .collect();
    let mut want = Vec::new();
    for (s, p) in [("primary", "ana"), ("primary", "cara"), ("primary", "human:dana-roe:ana"), ("secondary", "bob")] {
        for d in ["blocks", "missing", "works"] {
            want.push((s.to_string(), p.to_string(), d.to_string()));
        }
    }
    assert_eq!(seq, want);
    // the keys, in order, for a persona that holds traits: the fixed columns, then trait.<id> sorted, then sensitive
    let first: Vec<&str> = rows[0].as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(
        first,
        [
            "run", "subject", "subject_version", "lens", "lens_version", "persona", "stratum", "model", "dimension", "about",
            "score", "anchor_met", "evidence", "citations", "blockers", "phase", "by_human",
            "trait.choice.decision_style", "trait.expertise.domain", "sensitive"
        ]
    );
    let works = rows.iter().find(|r| r["persona"] == "ana" && r["dimension"] == "works").unwrap();
    assert_eq!(works["run"], 1);
    assert_eq!(works["subject"], "site");
    assert_eq!(works["subject_version"], 1);
    assert_eq!(works["lens"], "critical");
    assert_eq!(works["lens_version"], 1);
    assert_eq!(works["model"], "m");
    assert_eq!(works["about"], Value::Null);
    assert_eq!(works["score"], 4);
    assert_eq!(works["evidence"], "seen");
    assert_eq!(works["citations"], json!(["page.json#n1", "page.json#n3"]));
    assert_eq!(works["blockers"], 1, "a count of the blockers, not their text");
    assert_eq!(works["phase"], "initial");
    assert_eq!(works["by_human"], Value::Null);
    assert_eq!(works["trait.expertise.domain"], 4);
    assert_eq!(works["trait.choice.decision_style"], "rational");
    assert_eq!(works["sensitive"], json!([]));
    // a persona without traits has no trait keys; a person's rows carry the person and the traits they declared
    let cara = rows.iter().find(|r| r["persona"] == "cara").unwrap();
    assert!(cara.as_object().unwrap().keys().all(|k| !k.starts_with("trait.")), "{cara}");
    let human = rows.iter().find(|r| r["persona"] == "human:dana-roe:ana").unwrap();
    assert_eq!(human["by_human"], "Dana Roe, Senior Buyer");
    assert_eq!(human["stratum"], "primary");
    assert_eq!(human["model"], Value::Null);
    assert_eq!(human["trait.expertise.domain"], 1);
}

#[test]
fn no_pooled_or_derived_number_appears() {
    let ws = world();
    let (text, _) = raw(ws.root(), &["export", "1", "2"]);
    for r in lines(&text) {
        for k in r.as_object().unwrap().keys() {
            for banned in ["mean", "median", "average", "avg", "total", "sum", "rank"] {
                assert!(!k.contains(banned), "{k}");
            }
        }
    }
}

#[test]
fn the_same_store_gives_the_same_bytes_whatever_the_order_of_the_runs() {
    let ws = world();
    let (a, _) = raw(ws.root(), &["export", "1", "2"]);
    let (b, _) = raw(ws.root(), &["export", "2", "1"]);
    let (c, _) = raw(ws.root(), &["export", "1", "2", "1"]);
    assert_eq!(a, b);
    assert_eq!(a, c, "a run named twice is exported once");
    let (again, _) = raw(ws.root(), &["export", "1", "2"]);
    assert_eq!(a, again);
    let runs: Vec<i64> = lines(&a).iter().map(|r| r["run"].as_i64().unwrap()).collect();
    assert_eq!(runs.iter().filter(|r| **r == 1).count(), 12);
    assert_eq!(runs.iter().filter(|r| **r == 2).count(), 3);
    assert!(runs.windows(2).all(|w| w[0] <= w[1]), "run is the first sort key: {runs:?}");
    let (csv1, _) = raw(ws.root(), &["export", "1", "2", "--format", "csv"]);
    let (csv2, _) = raw(ws.root(), &["export", "2", "1", "--format", "csv"]);
    assert_eq!(csv1, csv2);
}

#[test]
fn csv_is_rfc_4180_with_the_union_of_trait_columns_and_quoted_commas() {
    let ws = world();
    let (text, ok) = raw(ws.root(), &["export", "1", "--format", "csv"]);
    assert!(ok, "{text}");
    assert!(text.ends_with("\r\n") && !text.replace("\r\n", "").contains('\n') && !text.replace("\r\n", "").contains('\r'), "CRLF records");
    let shape = personajury::domain::csv::shape(&text);
    assert_eq!(shape.len(), 13, "a header and 12 rows");
    assert!(shape.iter().all(|n| *n == shape[0]), "every record has the header's fields: {shape:?}");
    let header = text.lines().next().unwrap();
    assert_eq!(
        header,
        "run,subject,subject_version,lens,lens_version,persona,stratum,model,dimension,about,score,anchor_met,evidence,citations,\
blockers,phase,by_human,trait.choice.decision_style,trait.expertise.domain,sensitive"
    );
    // the citations of one row are joined with ';' and the comma in "Name, Credential" quotes its field
    assert!(text.contains(",page.json#n1;page.json#n3,1,initial,,rational,4,"), "{text}");
    assert!(text.contains(",\"Dana Roe, Senior Buyer\","), "{text}");
    // a persona with no traits: the trait cells are empty
    let cara = text.lines().find(|l| l.contains(",cara,")).unwrap();
    assert!(cara.ends_with(",initial,,,,"), "{cara}");
}

#[test]
fn a_field_with_a_quote_a_comma_or_a_line_break_is_quoted_and_the_quote_doubled() {
    use personajury::domain::export::csv_field;
    assert_eq!(csv_field("plain"), "plain");
    assert_eq!(csv_field("a,b"), "\"a,b\"");
    assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
    assert_eq!(csv_field("two\nlines"), "\"two\nlines\"");
    assert_eq!(csv_field("cr\rhere"), "\"cr\rhere\"");
    assert_eq!(csv_field(""), "");
}

#[test]
fn the_phase_filter_keeps_the_initial_batch_the_default() {
    let ws = world();
    // a round on run 1: ana changes her mind on works
    let d = ws.cli(&["deliberate", "1"]);
    assert_eq!(d["ok"], true, "{d}");
    let mut v = critical("ana", [2, 2, 5], &[], &["page.json#n1"]);
    v["phase"] = json!("deliberated");
    let r = ws.cli(&["verdict", "--run", "1", "--json", &v.to_string(), "--deliberated"]);
    assert_eq!(r["ok"], true, "{r}");
    let phases = |args: &[&str]| -> Vec<(String, String, String)> {
        let (t, ok) = raw(ws.root(), args);
        assert!(ok, "{t}");
        lines(&t).iter().map(|r| (r["phase"].as_str().unwrap().into(), r["persona"].as_str().unwrap().into(), r["dimension"].as_str().unwrap().into())).collect()
    };
    let default = phases(&["export", "1"]);
    assert_eq!(default.len(), 12);
    assert!(default.iter().all(|(p, ..)| p == "initial"));
    assert_eq!(default, phases(&["export", "1", "--phase", "initial"]));
    let delib = phases(&["export", "1", "--phase", "deliberated"]);
    assert_eq!(delib.len(), 3);
    assert!(delib.iter().all(|(p, who, _)| p == "deliberated" && who == "ana"));
    let all = phases(&["export", "1", "--phase", "all"]);
    assert_eq!(all.len(), 15);
    let ana_blocks: Vec<&str> = all.iter().filter(|(_, p, d)| p == "ana" && d == "blocks").map(|(ph, ..)| ph.as_str()).collect();
    assert_eq!(ana_blocks, ["initial", "deliberated"], "inside a cell the initial row comes first");
    // the deliberated row carries the deliberated score
    let (t, _) = raw(ws.root(), &["export", "1", "--phase", "deliberated"]);
    let works = lines(&t).into_iter().find(|r| r["dimension"] == "works").unwrap();
    assert_eq!(works["score"], 2);
    let bad = ws.cli(&["export", "1", "--phase", "both"]);
    assert_eq!(code(&bad), "invalid_arguments");
}

#[test]
fn out_writes_the_file_and_answers_an_envelope() {
    let ws = world();
    let f = ws.root().join("rows.csv");
    let r = ws.cli(&["export", "1", "2", "--format", "csv", "--out", f.to_str().unwrap()]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"]["rows"], 15);
    assert_eq!(r["data"]["format"], "csv");
    let (stdout_form, _) = raw(ws.root(), &["export", "1", "2", "--format", "csv"]);
    assert_eq!(std::fs::read_to_string(&f).unwrap(), stdout_form);
}

#[test]
fn refusals_are_envelopes() {
    let ws = world();
    assert_eq!(code(&ws.cli(&["export", "9"])), "not_found");
    assert_eq!(code(&ws.cli(&["export", "1", "--format", "xml"])), "invalid_arguments");
    let (t, ok) = raw(ws.root(), &["export", "9"]);
    assert!(!ok);
    assert_eq!(serde_json::from_str::<Value>(&t).unwrap()["error"]["code"], "not_found");
}

#[test]
fn the_row_cap_marks_the_answer_truncated() {
    let ws = world();
    let all = personajury::application::export::show(ws.root(), &[1, 2], "ndjson", "initial", None, None).unwrap();
    assert_eq!(all["rows"], 15);
    assert_eq!(all["truncated"], false);
    let cut = personajury::application::export::show(ws.root(), &[1, 2], "ndjson", "initial", None, Some(5)).unwrap();
    assert_eq!(cut["rows"], 15, "the total is still said");
    assert_eq!(cut["truncated"], true);
    assert_eq!(cut["ndjson"].as_str().unwrap().lines().count(), 5);
    assert_eq!(personajury::domain::export::MCP_ROW_CAP, 10_000);
}

#[test]
fn sensitive_labels_of_the_persona_ride_on_every_row() {
    let ws = Ws::new();
    ws.site();
    let mut body = json!({
        "id": "ana", "name": "Ana Alpha", "tags": ["x"], "role": "a retired nurse aged 70", "context": "c", "daily_tasks": "d",
        "fears": "f", "never_reads": "n", "decides_by": "b", "stake_rules": "s"
    });
    body["tags"] = json!(["x"]);
    let f = ws.root().join("ana.json");
    std::fs::write(&f, body.to_string()).unwrap();
    assert_eq!(ws.cli(&["persona", "add", "--file", f.to_str().unwrap()])["ok"], true);
    let run = ws.run("critical", &[("ana", Stake::Primary)], false);
    ws.ingest(run, &critical("ana", [3, 3, 3], &[], &["page.json#n1"]));
    let (t, _) = raw(ws.root(), &["export", &run.to_string()]);
    let rows = lines(&t);
    assert!(!rows.is_empty());
    for r in &rows {
        assert_eq!(r["sensitive"], json!(["age"]), "{r}");
    }
    let (c, _) = raw(ws.root(), &["export", &run.to_string(), "--format", "csv"]);
    assert!(c.lines().nth(1).unwrap().ends_with(",age"), "{c}");
}

#[test]
fn the_fixture_runs_export_to_the_golden_files_byte_for_byte() {
    let ws = tempfile::tempdir().unwrap();
    let (a, b) = fixture_run(ws.path());
    assert_eq!((a, b), (1, 2));
    for (format, file) in [("ndjson", "export.ndjson"), ("csv", "export.csv")] {
        let (got, ok) = raw(ws.path(), &["export", "1", "2", "--format", format]);
        assert!(ok, "{got}");
        let golden = repo().join("fixtures/run").join(file);
        if std::env::var_os("PERSONAJURY_FIXTURE_REGEN").is_some() {
            std::fs::write(&golden, &got).unwrap();
        }
        assert_eq!(got, std::fs::read_to_string(&golden).unwrap(), "{file} moved");
    }
    // 14 verdicts, three lines each in critical; the compare lens scores each option
    let (nd, _) = raw(ws.path(), &["export", "1", "2"]);
    let rows = lines(&nd);
    assert_eq!(rows.iter().filter(|r| r["run"] == 1).count(), 7 * 3);
    assert!(rows.iter().all(|r| r["phase"] == "initial" && r["by_human"].is_null()));
}

// ---------------------------------------------------------------------------------------------------------------
// the chat pack
// ---------------------------------------------------------------------------------------------------------------

/// Three personas (ids ana, bob, cara), a one-page document subject, and a brief with `extra` flags.
fn chat_world() -> tempfile::TempDir {
    let ws = tempfile::tempdir().unwrap();
    let w = ws.path();
    assert_eq!(cli_in(w, &["trait", "install"])["ok"], true);
    for (id, tag) in [("ana", "a"), ("bob", "b"), ("cara", "c")] {
        let body = json!({
            "id": id, "name": format!("Person {id}"), "tags": [tag], "role": "r", "context": "c", "daily_tasks": "d",
            "fears": "f", "never_reads": "n", "decides_by": "b", "stake_rules": "s"
        });
        let f = w.join(format!("{id}.json"));
        std::fs::write(&f, body.to_string()).unwrap();
        assert_eq!(cli_in(w, &["persona", "add", "--file", f.to_str().unwrap()])["ok"], true);
    }
    let page = w.join("page.md");
    std::fs::write(&page, "# page\nline two\n").unwrap();
    assert_eq!(cli_in(w, &["subject", "add", "doc", "--kind", "document", "--attach", page.to_str().unwrap()])["ok"], true);
    ws
}
const QUERY: &str = "1 a=primary, 1 b=primary, 1 c=secondary";
fn brief_with_pack(w: &Path, extra: &[&str]) -> Value {
    let pack = w.join("pack.md");
    let mut a = vec!["brief", "--subject", "doc", "--query", QUERY, "--lens", "critical", "--model", "m", "--chat-pack", pack.to_str().unwrap()];
    a.extend(extra);
    cli_in(w, &a)
}
/// One juror's verdict for the document subject, as the chat would write it.
fn doc_verdict(persona: &str, score: i64) -> Value {
    let lines: Vec<Value> = ["works", "blocks", "missing"]
        .iter()
        .map(|d| json!({"dimension": d, "score": score, "anchor_met": if score >= 3 { 3 } else { 1 }, "evidence": "seen", "citations": ["page.md#L1"]}))
        .collect();
    json!({"persona": persona, "lines": lines})
}
fn block(persona: &str, v: &Value) -> String {
    format!("### {persona}\n\n```json\n{}\n```\n", serde_json::to_string_pretty(v).unwrap())
}
fn save_answer(w: &Path, text: &str) -> String {
    let f = w.join("answer.md");
    std::fs::write(&f, text).unwrap();
    f.to_string_lossy().into_owned()
}

#[test]
fn the_pack_is_one_file_with_instructions_then_every_brief_in_order() {
    let ws = chat_world();
    let w = ws.path();
    let r = brief_with_pack(w, &[]);
    assert_eq!(r["ok"], true, "{r}");
    let d = &r["data"];
    let pack = std::fs::read_to_string(w.join("pack.md")).unwrap();
    assert_eq!(d["chat_pack"], w.join("pack.md").to_string_lossy().as_ref());
    // the instruction block comes first and says the three things a chat needs
    let first_brief = pack.find("personajury-brief 1").unwrap();
    let head = &pack[..first_brief];
    assert!(head.contains("### <persona-id>"), "{head}");
    assert!(head.contains("```json"), "{head}");
    assert!(head.contains("separately"), "{head}");
    assert!(head.contains("--from-chat"), "{head}");
    // every brief, byte for byte, in the order of the answer's briefs
    let mut at = first_brief;
    for b in d["briefs"].as_array().unwrap() {
        let text = std::fs::read_to_string(w.join(b["path"].as_str().unwrap())).unwrap();
        let i = pack[at..].find(&text).unwrap_or_else(|| panic!("brief of {} not in order", b["persona"])) + at;
        at = i + text.len();
    }
    // the run is sequential without being told, so it only advises
    assert_eq!(d["independence"], "sequential");
    let run = d["run"].as_i64().unwrap();
    assert_eq!(personajury::store::Store::open(w).unwrap().run(run).unwrap().unwrap().independence, "sequential");
}

#[test]
fn a_pack_with_an_independent_run_or_two_lenses_is_refused_before_anything_is_made() {
    let ws = chat_world();
    let w = ws.path();
    let r = brief_with_pack(w, &["--independence", "independent"]);
    assert_eq!(code(&r), "invalid_arguments");
    assert!(r["error"]["message"].as_str().unwrap().contains("sequential"), "{r}");
    assert!(!w.join("pack.md").exists());
    let r = brief_with_pack(w, &["--lens", "taste"]);
    assert_eq!(code(&r), "invalid_arguments");
    assert!(!w.join("pack.md").exists());
    assert_eq!(cli_in(w, &["doctor"])["data"]["counts"]["runs"], 0, "no run was created");
}

#[test]
fn a_chat_answer_with_three_jurors_ingests_three() {
    let ws = chat_world();
    let w = ws.path();
    let run = brief_with_pack(w, &[])["data"]["run"].as_i64().unwrap();
    let answer = format!(
        "Here are my three jurors.\n\n{}\nSome words between.\n\n{}\n{}",
        block("cara", &doc_verdict("cara", 3)),
        block("ana", &doc_verdict("ana", 4)),
        block("bob", &doc_verdict("bob", 2)),
    );
    let f = save_answer(w, &answer);
    let r = cli_in(w, &["verdict", "--run", &run.to_string(), "--from-chat", &f]);
    assert_eq!(r["ok"], true, "{r}");
    let got: Vec<&str> = r["data"]["ingested"].as_array().unwrap().iter().map(|i| i["persona"].as_str().unwrap()).collect();
    assert_eq!(got, ["ana", "bob", "cara"], "in the run's order, whatever the chat's");
    assert_eq!(r["data"]["refused"], json!([]));
    assert_eq!(r["data"]["missing"], json!([]));
    let (t, _) = raw(w, &["export", &run.to_string()]);
    assert_eq!(lines(&t).len(), 9, "three jurors, three lines each, through the normal checks");
}

#[test]
fn a_refused_block_is_named_and_the_others_still_land() {
    let ws = chat_world();
    let w = ws.path();
    let run = brief_with_pack(w, &[])["data"]["run"].as_i64().unwrap();
    // ana: not json; bob: a `seen` line with no citation (the normal refusal); cara: fine
    let mut uncited = doc_verdict("bob", 2);
    uncited["lines"][0]["citations"] = json!([]);
    let answer = format!(
        "### ana\n\n```json\n{{ \"persona\": \"ana\", \"lines\": [ oops\n```\n\n{}\n{}",
        block("bob", &uncited),
        block("cara", &doc_verdict("cara", 3)),
    );
    let f = save_answer(w, &answer);
    let r = cli_in(w, &["verdict", "--run", &run.to_string(), "--from-chat", &f]);
    assert_eq!(code(&r), "chat_answer_incomplete");
    let d = &r["data"];
    let ingested: Vec<&str> = d["ingested"].as_array().unwrap().iter().map(|i| i["persona"].as_str().unwrap()).collect();
    assert_eq!(ingested, ["cara"]);
    let refused: Vec<(&str, &str)> = d["refused"].as_array().unwrap().iter().map(|i| (i["persona"].as_str().unwrap(), i["code"].as_str().unwrap())).collect();
    assert_eq!(refused, [("ana", "chat_block_malformed"), ("bob", "seen_without_citation")]);
    assert!(d["refused"][0]["message"].as_str().unwrap().contains("json"), "{d}");
    assert_eq!(d["missing"], json!([]));
    let (t, _) = raw(w, &["export", &run.to_string()]);
    assert_eq!(lines(&t).iter().map(|r| r["persona"].as_str().unwrap().to_string()).collect::<std::collections::BTreeSet<_>>().len(), 1);
    // the refused juror answers again from a second chat answer: the same verb takes it
    let again = save_answer(w, &format!("{}\n{}", block("ana", &doc_verdict("ana", 4)), block("bob", &doc_verdict("bob", 2))));
    let r = cli_in(w, &["verdict", "--run", &run.to_string(), "--from-chat", &again]);
    assert_eq!(code(&r), "chat_answer_incomplete", "cara is not in this answer");
    assert_eq!(r["data"]["missing"], json!(["cara"]));
    assert_eq!(r["data"]["ingested"].as_array().unwrap().len(), 2);
}

#[test]
fn a_missing_persona_is_listed() {
    let ws = chat_world();
    let w = ws.path();
    let run = brief_with_pack(w, &[])["data"]["run"].as_i64().unwrap();
    let f = save_answer(w, &format!("{}\n{}", block("ana", &doc_verdict("ana", 4)), block("cara", &doc_verdict("cara", 3))));
    let r = cli_in(w, &["verdict", "--run", &run.to_string(), "--from-chat", &f]);
    assert_eq!(code(&r), "chat_answer_incomplete");
    assert_eq!(r["data"]["missing"], json!(["bob"]));
    assert_eq!(r["data"]["ingested"].as_array().unwrap().len(), 2);
    assert_eq!(r["data"]["refused"], json!([]));
}

#[test]
fn a_block_under_another_persons_heading_is_refused() {
    let ws = chat_world();
    let w = ws.path();
    let run = brief_with_pack(w, &[])["data"]["run"].as_i64().unwrap();
    let f = save_answer(w, &format!("{}\n{}", block("ana", &doc_verdict("bob", 4)), block("bob", &doc_verdict("bob", 2))));
    let r = cli_in(w, &["verdict", "--run", &run.to_string(), "--from-chat", &f]);
    assert_eq!(code(&r), "chat_answer_incomplete");
    let refused: Vec<(&str, &str)> = r["data"]["refused"].as_array().unwrap().iter().map(|i| (i["persona"].as_str().unwrap(), i["code"].as_str().unwrap())).collect();
    assert_eq!(refused, [("ana", "chat_block_persona_mismatch")]);
    assert_eq!(r["data"]["ingested"].as_array().unwrap().len(), 1);
}

#[test]
fn a_juror_answered_twice_or_without_a_block_is_refused() {
    let ws = chat_world();
    let w = ws.path();
    let run = brief_with_pack(w, &[])["data"]["run"].as_i64().unwrap();
    let answer = format!(
        "{}\n{}\n### bob\n\nI would say it is fine but I forgot the block.\n\n{}",
        block("ana", &doc_verdict("ana", 4)),
        block("ana", &doc_verdict("ana", 1)),
        block("cara", &doc_verdict("cara", 3)),
    );
    let f = save_answer(w, &answer);
    let r = cli_in(w, &["verdict", "--run", &run.to_string(), "--from-chat", &f]);
    assert_eq!(code(&r), "chat_answer_incomplete");
    let refused: Vec<(&str, &str)> = r["data"]["refused"].as_array().unwrap().iter().map(|i| (i["persona"].as_str().unwrap(), i["code"].as_str().unwrap())).collect();
    assert_eq!(refused, [("ana", "chat_block_malformed"), ("bob", "chat_block_malformed")]);
    assert_eq!(r["data"]["ingested"].as_array().unwrap().len(), 1);
}

#[test]
fn an_answer_with_no_juror_in_it_is_an_argument_error() {
    let ws = chat_world();
    let w = ws.path();
    let run = brief_with_pack(w, &[])["data"]["run"].as_i64().unwrap();
    let f = save_answer(w, "I am happy to help, but which page?\n");
    let r = cli_in(w, &["verdict", "--run", &run.to_string(), "--from-chat", &f]);
    assert_eq!(code(&r), "invalid_arguments");
    assert!(r["error"]["message"].as_str().unwrap().contains("### <persona-id>"), "{r}");
    let r = cli_in(w, &["verdict", "--run", &run.to_string(), "--from-chat", &w.join("nope.md").to_string_lossy()]);
    assert_eq!(code(&r), "not_found");
}

#[test]
fn a_sequential_chat_run_only_advises() {
    let ws = chat_world();
    let w = ws.path();
    let run = brief_with_pack(w, &[])["data"]["run"].as_i64().unwrap();
    let f = save_answer(
        w,
        &format!("{}\n{}\n{}", block("ana", &doc_verdict("ana", 4)), block("bob", &doc_verdict("bob", 4)), block("cara", &doc_verdict("cara", 4))),
    );
    assert_eq!(cli_in(w, &["verdict", "--run", &run.to_string(), "--from-chat", &f])["ok"], true);
    let a = cli_in(w, &["apply-check", &run.to_string()]);
    assert_eq!(a["data"]["answer"], "advise", "{a}");
    let reasons: Vec<&str> = a["data"]["reasons"].as_array().unwrap().iter().map(|r| r["code"].as_str().unwrap()).collect();
    assert!(reasons.contains(&"independence_sequential"), "{reasons:?}");
}

#[test]
fn the_mcp_tool_answers_ndjson_in_the_envelope() {
    use std::io::Write;
    let ws = world();
    let mut ch = std::process::Command::new(assert_cmd::cargo::cargo_bin("personajury"))
        .arg("--root")
        .arg(ws.root())
        .arg("mcp")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = ch.stdin.take().unwrap();
    for (id, args) in [(1, json!({"runs": [1]})), (2, json!({"runs": [1, 2], "phase": "all"})), (3, json!({"runs": [1], "format": "csv"}))] {
        let req = json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"personajury_export","arguments":args}});
        writeln!(stdin, "{req}").unwrap();
    }
    drop(stdin);
    let out = ch.wait_with_output().unwrap();
    let replies: Vec<Value> = String::from_utf8(out.stdout).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    let env = |i: usize| -> Value { serde_json::from_str(replies[i]["result"]["content"][0]["text"].as_str().unwrap()).unwrap() };
    let one = env(0);
    assert_eq!((one["ok"].clone(), one["verb"].clone()), (json!(true), json!("export")), "{one}");
    assert_eq!(one["data"]["rows"], 12);
    assert_eq!(one["data"]["truncated"], false);
    assert_eq!(one["data"]["ndjson"].as_str().unwrap().lines().count(), 12);
    assert_eq!(env(1)["data"]["rows"], 15);
    // the tool has no csv: an unknown argument is refused like any other
    assert_eq!(env(2)["error"]["code"], "invalid_arguments");
}

/// `export 1 | head -1`: the reader closes the pipe after one line; the CLI must end quietly (0), never panic (101).
#[test]
fn a_closed_pipe_ends_export_quietly() {
    use std::process::{Command as Std, Stdio};
    let ws = world();
    // make the output big enough that the writer is still writing when the reader leaves
    for _ in 0..40 {
        ws.ingest(1, &critical("bob", [3, 3, 3], &[], &["page.json#n1"]));
    }
    let bin = assert_cmd::cargo::cargo_bin("personajury");
    let mut child = Std::new(bin)
        .arg("--root")
        .arg(ws.root())
        .args(["export", "1", "2", "--phase", "all"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // close the reading end before the CLI writes anything: every write meets a closed pipe
    drop(child.stdout.take());
    let out = child.wait_with_output().unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("panicked"), "panic on a closed pipe: {err}");
    assert_ne!(out.status.code(), Some(101), "{err}");
}
