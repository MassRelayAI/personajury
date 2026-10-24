//! apply-check, every condition in one table (D13 as ruled 2026-10-24): `apply` only when every condition is proven;
//! anything unknown, missing or lowered answers `advise` (or worse). Each row builds its own store and names the
//! answer and the reason code it must carry. Seen red first.
mod judging_support;
use personajury::domain::Stake;
use judging_support::*;
use serde_json::{json, Value};
use std::path::Path;

/// A low rule on the site's sources, and a low `docs` rule elsewhere.
const LOW: &str = "[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/site/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n\n[[rule]]\nclass = \"docs\"\npaths = [\"docs/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n";
/// The site's sources need the high quorum; `docs` stays low.
const HIGH: &str = "[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/site/**\"]\nquorum = \"high\"\npanel = \"p\"\nlenses = [\"critical\"]\n\n[[rule]]\nclass = \"docs\"\npaths = [\"docs/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n";
const HUMAN: &str = "[[rule]]\nclass = \"money\"\npaths = [\"sources/site/**\"]\nquorum = \"human\"\n";

fn toml(ws: &Ws, body: &str) {
    std::fs::write(ws.root().join("personajury.toml"), body).unwrap();
}
/// Record the rule a run was briefed under and its effective quorum, as `brief --rule` does.
fn under(ws: &Ws, run: i64, rule: &str, quorum: &str) {
    ws.store
        .conn()
        .execute("UPDATE run SET rule_class = ?2, quorum = ?3 WHERE id = ?1", rusqlite::params![run, rule, quorum])
        .unwrap();
}
fn set(ws: &Ws, run: i64, col: &str, v: &str) {
    ws.store.conn().execute(&format!("UPDATE run SET {col} = ?2 WHERE id = ?1"), rusqlite::params![run, v]).unwrap();
}
fn clean(ws: &Ws, run: i64) {
    ws.ingest(run, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
    ws.ingest(run, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
}
/// The site, the toml, one clean run of the two seats under `public-surface` at `quorum`.
fn base(body: &str, quorum: &str) -> (Ws, i64) {
    let ws = Ws::new();
    ws.site();
    toml(&ws, body);
    let run = ws.run("critical", &TWO, false);
    under(&ws, run, "public-surface", quorum);
    (ws, run)
}
fn ovr(fields: Value) -> String {
    json!({"rule": "public-surface", "fields": fields, "reason": "cheaper", "by": "agent"}).to_string()
}

struct Case {
    name: &'static str,
    /// builds the store; returns the workspace and the apply-check arguments
    setup: fn() -> (Ws, Vec<String>),
    answer: &'static str,
    /// a reason code the answer must carry (`-` for none in particular)
    code: &'static str,
}

fn cases() -> Vec<Case> {
    vec![
        Case { name: "a clean run under its low rule", answer: "apply", code: "single_model", setup: || {
            let (ws, r) = base(LOW, "low");
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "C1 nobody judged", answer: "advise", code: "incomplete_jury", setup: || {
            let (ws, r) = base(LOW, "low");
            (ws, vec![r.to_string()])
        }},
        Case { name: "C1 the primary seat never answered", answer: "advise", code: "incomplete_jury", setup: || {
            let (ws, r) = base(LOW, "low");
            ws.ingest(r, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
            (ws, vec![r.to_string()])
        }},
        Case { name: "C1 a seat of an earlier given round never answered", answer: "advise", code: "incomplete_jury", setup: || {
            let (ws, r1) = base(LOW, "low");
            ws.ingest(r1, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
            let r2 = ws.run("critical", &TWO, false);
            under(&ws, r2, "public-surface", "low");
            clean(&ws, r2);
            (ws, vec![r1.to_string(), r2.to_string()])
        }},
        Case { name: "C3 no rule and no classifiable source", answer: "advise", code: "quorum_unknown", setup: || {
            let ws = Ws::new();
            ws.site();
            let r = ws.run("critical", &TWO, false);
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "C2 a --rule lower than the sources", answer: "advise", code: "rule_below_sources", setup: || {
            let (ws, r) = base(HIGH, "high");
            clean(&ws, r);
            (ws, vec![r.to_string(), "--rule".into(), "docs".into()])
        }},
        Case { name: "C2 the run's recorded rule lower than the sources", answer: "advise", code: "rule_below_sources", setup: || {
            let (ws, r) = base(HIGH, "high");
            under(&ws, r, "docs", "low");
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "C3 the run's recorded high quorum holds when --rule is dropped", answer: "advise", code: "effort_below_quorum", setup: || {
            let (ws, r) = base(LOW, "high");
            set(&ws, r, "effort", "low");
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "C4 seats overridden down", answer: "advise", code: "overridden_down", setup: || {
            let (ws, r) = base(LOW, "low");
            set(&ws, r, "override_json", &ovr(json!({"spend": {"rule": 3, "given": 2}})));
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "C4 lenses overridden down", answer: "advise", code: "overridden_down", setup: || {
            let (ws, r) = base(LOW, "low");
            set(&ws, r, "override_json", &ovr(json!({"lenses": {"rule": ["critical", "taste"], "given": ["critical"]}})));
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "C4 detail overridden down", answer: "advise", code: "overridden_down", setup: || {
            let (ws, r) = base(LOW, "low");
            set(&ws, r, "override_json", &ovr(json!({"detail": {"rule": "full", "given": "compact"}})));
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "C4 effort below the toml's", answer: "advise", code: "overridden_down", setup: || {
            let (ws, r) = base(&format!("effort = \"high\"\n{LOW}"), "low");
            set(&ws, r, "effort", "medium");
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "C4 briefed with no rule on a ruled source", answer: "advise", code: "overridden_down", setup: || {
            let ws = Ws::new();
            ws.site();
            toml(&ws, LOW);
            let r = ws.run("critical", &TWO, false);
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "C4 seats, lenses and detail raised", answer: "apply", code: "-", setup: || {
            let (ws, r) = base(LOW, "low");
            let f = json!({"spend": {"rule": 1, "given": 2}, "lenses": {"rule": ["critical"], "given": ["critical", "taste"]},
                "detail": {"rule": "compact", "given": "full"}});
            set(&ws, r, "override_json", &ovr(f));
            set(&ws, r, "effort", "high");
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "D17 effort low on a high quorum", answer: "advise", code: "effort_below_quorum", setup: || {
            let (ws, r) = base(HIGH, "high");
            set(&ws, r, "effort", "low");
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "D18 a sequential run", answer: "advise", code: "independence_sequential", setup: || {
            let (ws, r) = base(LOW, "low");
            set(&ws, r, "independence", "sequential");
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
        Case { name: "I1 the subject moved since the run", answer: "advise", code: "stale_run", setup: || {
            let (ws, r) = base(LOW, "low");
            clean(&ws, r);
            ws.subject("site", personajury::domain::SubjectKind::Page, false, &[("page.json", br#"{"nodes":[{"id":"n1"}]}"#)]);
            (ws, vec![r.to_string()])
        }},
        Case { name: "I1 a later round supersedes the run", answer: "advise", code: "stale_run", setup: || {
            let (ws, r1) = base(LOW, "low");
            clean(&ws, r1);
            let r2 = ws.run("critical", &TWO, false);
            under(&ws, r2, "public-surface", "low");
            clean(&ws, r2);
            (ws, vec![r1.to_string()])
        }},
        Case { name: "I2 a lens bump between rounds", answer: "advise", code: "delta_unavailable", setup: || {
            let (ws, r1) = base(LOW, "low");
            clean(&ws, r1);
            bump_critical(&ws);
            let r2 = ws.run("critical", &TWO, false);
            under(&ws, r2, "public-surface", "low");
            clean(&ws, r2);
            (ws, vec![r2.to_string()])
        }},
        Case { name: "I2 cells the previous round lacks", answer: "advise", code: "delta_unavailable", setup: || {
            let (ws, r1) = base(LOW, "low");
            ws.ingest(r1, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
            let team = [("ana", Stake::Primary), ("bob", Stake::Secondary)];
            let r2 = ws.run("critical", &team, false);
            under(&ws, r2, "public-surface", "low");
            clean(&ws, r2);
            (ws, vec![r2.to_string()])
        }},
        Case { name: "a primary blocker", answer: "escalate", code: "primary_blocker", setup: || {
            let (ws, r) = base(LOW, "low");
            ws.ingest(r, &critical("ana", [3, 3, 3], &["price hidden"], &["page.json#n1"]));
            ws.ingest(r, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
            (ws, vec![r.to_string()])
        }},
        Case { name: "C5 a primary blocker in one lens run, the persona secondary in another", answer: "escalate", code: "primary_blocker", setup: || {
            let ws = Ws::new();
            ws.site();
            ws.rival();
            toml(&ws, LOW);
            let a = ws.run("critical", &TWO, false);
            under(&ws, a, "public-surface", "low");
            let d = ws.ingest(a, &critical("ana", [3, 3, 3], &["price hidden"], &["page.json#n1"]));
            ws.ingest(a, &critical("bob", [3, 3, 3], &[], &["page.json#n1"]));
            let f = d["findings"][0].as_str().unwrap().to_string();
            assert_eq!(ws.cli(&["decide", &f, "fixed", "--by", "foreman"])["ok"], true);
            let b = ws.run("compare", &[("ana", Stake::Secondary), ("bob", Stake::Primary)], true);
            under(&ws, b, "public-surface", "low");
            ws.ingest(b, &compare("ana", "page.json", "ok"));
            ws.ingest(b, &compare("bob", "page.json", "ok"));
            (ws, vec![a.to_string(), b.to_string()])
        }},
        Case { name: "a split", answer: "escalate", code: "split", setup: || {
            let (ws, r) = base(LOW, "low");
            ws.ingest(r, &critical("ana", [2, 2, 2], &[], &["page.json#n1"]));
            ws.ingest(r, &critical("bob", [4, 4, 4], &[], &["page.json#n1"]));
            (ws, vec![r.to_string()])
        }},
        Case { name: "a delta that did not land", answer: "escalate", code: "delta_not_landed", setup: || {
            let (ws, r1) = base(LOW, "low");
            ws.ingest(r1, &critical("ana", [5, 5, 5], &[], &["page.json#n1"]));
            ws.ingest(r1, &critical("bob", [5, 5, 5], &[], &["page.json#n1"]));
            let r2 = ws.run("critical", &TWO, false);
            under(&ws, r2, "public-surface", "low");
            ws.ingest(r2, &critical("ana", [2, 2, 2], &[], &["page.json#n1"]));
            ws.ingest(r2, &critical("bob", [2, 2, 2], &[], &["page.json#n1"]));
            (ws, vec![r2.to_string()])
        }},
        Case { name: "I3 an undecided finding of the previous round", answer: "escalate", code: "check_failed", setup: || {
            let (ws, r1) = base(LOW, "low");
            ws.ingest(r1, &critical("ana", [4, 4, 4], &[], &["page.json#n1"]));
            ws.ingest(r1, &critical("bob", [4, 4, 4], &["slow"], &["page.json#n1"]));
            let r2 = ws.run("critical", &TWO, false);
            under(&ws, r2, "public-surface", "low");
            clean(&ws, r2);
            (ws, vec![r2.to_string()])
        }},
        Case { name: "D19 a human blocker", answer: "escalate", code: "human_blocker", setup: || {
            let (ws, r) = base(LOW, "low");
            clean(&ws, r);
            let v = critical("bob", [3, 3, 3], &["consent line missing"], &["page.json#n1"]);
            let d = ws.cli(&["verdict", "--run", &r.to_string(), "--json", &v.to_string(), "--by-human", "Dr Rao, nurse"]);
            assert_eq!(d["ok"], true, "{d}");
            (ws, vec![r.to_string()])
        }},
        Case { name: "a human class", answer: "human", code: "human_class", setup: || {
            let ws = Ws::new();
            ws.site();
            toml(&ws, HUMAN);
            let r = ws.run("critical", &TWO, false);
            clean(&ws, r);
            (ws, vec![r.to_string()])
        }},
    ]
}

/// Lens `critical` version 2, so the next run is on another lens version than the last.
fn bump_critical(ws: &Ws) {
    let src = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lenses/critical.toml")).unwrap();
    let f = ws.root().join("critical2.toml");
    std::fs::write(&f, src.replace("version = 1", "version = 2").replace("It does the job I came for", "It does the job")).unwrap();
    let r = ws.cli(&["lens", "add", "--file", f.to_str().unwrap()]);
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn every_condition_gives_its_answer() {
    let mut wrong = Vec::new();
    for c in cases() {
        let (ws, args) = (c.setup)();
        let mut a = vec!["apply-check".to_string()];
        a.extend(args);
        let a: Vec<&str> = a.iter().map(String::as_str).collect();
        let v = ws.cli(&a);
        let d = &v["data"];
        let codes: Vec<&str> = d["reasons"].as_array().map_or(vec![], |r| r.iter().filter_map(|x| x["code"].as_str()).collect());
        if v["ok"] != true || d["answer"] != c.answer || (c.code != "-" && !codes.contains(&c.code)) {
            wrong.push(format!("{}: want {} ({}), got {}", c.name, c.answer, c.code, v));
        }
    }
    assert!(wrong.is_empty(), "{} row(s) wrong:\n{}", wrong.len(), wrong.join("\n"));
}

#[test]
fn the_table_has_a_row_for_every_reason_code() {
    let rows: Vec<&str> = cases().iter().map(|c| c.code).collect();
    for code in [
        "incomplete_jury", "quorum_unknown", "rule_below_sources", "overridden_down", "effort_below_quorum",
        "independence_sequential", "stale_run", "delta_unavailable", "primary_blocker", "split", "delta_not_landed",
        "check_failed", "human_blocker", "human_class", "single_model",
    ] {
        assert!(rows.contains(&code), "no row of the table carries {code}");
    }
}
