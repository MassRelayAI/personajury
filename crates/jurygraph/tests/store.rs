use jurygraph::domain::*;
use jurygraph::store::{Store, StoreError};
use std::collections::BTreeMap;

fn tmp() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let s = Store::open(d.path()).unwrap();
    (d, s)
}

fn persona(id: &str, hash: &str) -> Persona {
    Persona {
        id: id.into(),
        version: 0,
        name: format!("Name {id}"),
        tags: vec!["builders".into()],
        stake_rules: "primary when it ships".into(),
        profile_json: "{\"role\":\"r\"}".into(),
        origin: Origin::Hand,
        hash: hash.into(),
    }
}

fn run_row() -> RunRow {
    RunRow {
        id: 0,
        subject_name: "site".into(),
        subject_version: 1,
        subject_hash: "sh".into(),
        panel_name: "p".into(),
        panel_hash: "ph".into(),
        lens_name: "critical".into(),
        lens_version: 1,
        model: "m".into(),
        spend: 2,
        stakes: BTreeMap::from([("a".to_string(), Stake::Primary)]),
        personas: vec!["a".into()],
        comparators: vec![("rival".into(), 1)],
        briefs: vec![],
        created_at: "2026-10-23T00:00:00Z".into(),
    }
}

fn verdict(persona: &str, score: i64, blocker: Option<&str>, pick: bool) -> ValidVerdict {
    let blockers: Vec<String> = blocker.iter().map(|b| b.to_string()).collect();
    let line = VerdictLine {
        dimension: "clarity".into(),
        score,
        anchor_met: 3,
        evidence: Evidence::Seen,
        about: None,
        citations: vec!["page#n1".into()],
        blockers: blockers.clone(),
        note: Some("n".into()),
    };
    ValidVerdict {
        persona: persona.into(),
        stake: Stake::Primary,
        lines: vec![ValidLine {
            line,
            resolved: vec!["h#n1".into()],
            citation_hash: "ch1".into(),
        }],
        pick: if pick {
            Some(PickInput {
                choice: "page".into(),
                reason: "best".into(),
            })
        } else {
            None
        },
        findings: blockers
            .iter()
            .map(|b| Finding {
                id: format!("f-{b}"),
                persona: persona.into(),
                dimension: "clarity".into(),
                about: None,
                text: b.clone(),
                citation_hash: "ch1".into(),
            })
            .collect(),
    }
}

fn subject(name: &str, hash: &str) -> Subject {
    Subject {
        name: name.into(),
        version: 0,
        kind: SubjectKind::Page,
        is_comparator: false,
        hash: hash.into(),
        attachments: vec![
            Attachment {
                label: "b".into(),
                media: "image".into(),
                hash: "hb".into(),
                source_path: "b.png".into(),
            },
            Attachment {
                label: "a".into(),
                media: "aria-json".into(),
                hash: "ha".into(),
                source_path: "a.json".into(),
            },
        ],
    }
}

#[test]
fn open_creates_store_and_schema_version() {
    let (d, s) = tmp();
    assert!(d.path().join(".jurygraph/store.db").exists());
    assert_eq!(s.schema_version().unwrap(), 2);
    assert_eq!(Store::open(d.path()).unwrap().schema_version().unwrap(), 2);
    assert!(Store::open_existing(d.path()).is_ok());
}

#[test]
fn open_existing_refuses_missing_workspace() {
    let d = tempfile::tempdir().unwrap();
    let err = Store::open_existing(d.path()).unwrap_err();
    assert!(matches!(err, StoreError::NotFound(_)));
    assert!(err.to_string().contains("no workspace at"), "{err}");
    assert!(!d.path().join(".jurygraph").exists());
}

#[test]
fn open_seeds_four_shipped_lenses() {
    let (d, s) = tmp();
    let names: Vec<String> = s.lenses().unwrap().into_iter().map(|l| l.name).collect();
    assert_eq!(names, vec!["compare", "critical", "recall", "taste"]);
    assert_eq!(s.lens("critical").unwrap().unwrap().version, 1);
    // re-opening seeds nothing new
    let s2 = Store::open(d.path()).unwrap();
    assert_eq!(s2.lens_at("taste", 2).unwrap(), None);
    assert_eq!(s2.lenses().unwrap().len(), 4);
}

#[test]
fn persona_same_content_same_version() {
    let (_d, s) = tmp();
    assert_eq!(s.insert_persona(&persona("a", "h1")).unwrap(), 1);
    assert_eq!(s.insert_persona(&persona("a", "h1")).unwrap(), 1);
    let n: i64 = s
        .conn()
        .query_row("SELECT count(*) FROM persona", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
    let got = s.persona("a").unwrap().unwrap();
    assert_eq!(got.version, 1);
    assert_eq!(got.tags, vec!["builders".to_string()]);
    assert_eq!(got.origin, Origin::Hand);
}

#[test]
fn persona_changed_content_new_version() {
    let (_d, s) = tmp();
    s.insert_persona(&persona("a", "h1")).unwrap();
    s.insert_persona(&persona("b", "hb")).unwrap();
    let mut p2 = persona("a", "h2");
    p2.origin = Origin::GeneratedFrom("m1".into());
    assert_eq!(s.insert_persona(&p2).unwrap(), 2);
    assert_eq!(s.persona("a").unwrap().unwrap().hash, "h2");
    assert_eq!(s.persona_at("a", 1).unwrap().unwrap().hash, "h1");
    assert_eq!(s.persona_at("a", 3).unwrap(), None);
    assert_eq!(s.persona("zz").unwrap(), None);
    let all = s.personas().unwrap();
    assert_eq!(
        all.iter()
            .map(|p| (p.id.as_str(), p.version))
            .collect::<Vec<_>>(),
        vec![("a", 2), ("b", 1)]
    );
    assert_eq!(all[0].origin, Origin::GeneratedFrom("m1".into()));
}

#[test]
fn lens_and_subject_versioning() {
    let (_d, s) = tmp();
    let mut l = s.lens("critical").unwrap().unwrap();
    assert_eq!(
        s.insert_lens(&l).unwrap(),
        1,
        "same content, version field ignored"
    );
    l.standard = "changed".into();
    assert_eq!(s.insert_lens(&l).unwrap(), 2);
    assert_eq!(s.lens("critical").unwrap().unwrap().standard, "changed");
    assert_eq!(s.lens_at("critical", 1).unwrap().unwrap().version, 1);

    assert_eq!(s.insert_subject(&subject("site", "x")).unwrap(), 1);
    assert_eq!(s.insert_subject(&subject("site", "x")).unwrap(), 1);
    assert_eq!(s.insert_subject(&subject("site", "y")).unwrap(), 2);
    let mut c = subject("rival", "z");
    c.is_comparator = true;
    s.insert_subject(&c).unwrap();
    let got = s.subject("site").unwrap().unwrap();
    assert_eq!(got.version, 2);
    assert_eq!(
        got.attachments
            .iter()
            .map(|a| a.label.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "b"]
    );
    assert_eq!(s.subject_at("site", 1).unwrap().unwrap().hash, "x");
    assert_eq!(s.subjects(false).unwrap().len(), 1);
    assert_eq!(s.subjects(true).unwrap()[0].name, "rival");
}

#[test]
fn material_panel_and_run_round_trip() {
    let (_d, s) = tmp();
    assert!(!s.material_exists("m").unwrap());
    s.insert_material("m", &["a.md".into()], "p/brief.md")
        .unwrap();
    s.insert_material("m", &["a.md".into()], "p/brief.md")
        .unwrap();
    assert!(s.material_exists("m").unwrap());
    s.upsert_panel("p", "5 builders").unwrap();
    s.set_panel_resolution("p", &["a".into(), "b".into()], "ph")
        .unwrap();
    s.upsert_panel("p", "6 builders").unwrap();
    let p = s.panel("p").unwrap().unwrap();
    assert_eq!(
        (p.query.as_str(), p.persona_ids.len(), p.hash.as_deref()),
        ("6 builders", 2, Some("ph"))
    );
    assert_eq!(s.panels().unwrap().len(), 1);
    assert_eq!(s.panel("none").unwrap(), None);

    let id = s.insert_run(&run_row()).unwrap();
    let id2 = s.insert_run(&run_row()).unwrap();
    assert_eq!((id, id2), (1, 2));
    let brief = BriefRef {
        persona: "a".into(),
        persona_version: 3,
        stake: Stake::Primary,
        path: "x".into(),
        hash: "h".into(),
        bytes: 9,
    };
    s.set_run_briefs(id, std::slice::from_ref(&brief)).unwrap();
    let r = s.run(id).unwrap().unwrap();
    assert_eq!(r.id, 1);
    assert_eq!(r.briefs, vec![brief]);
    assert_eq!(r.stakes, run_row().stakes);
    assert_eq!(r.comparators, vec![("rival".to_string(), 1)]);
    assert_eq!(s.runs().unwrap().len(), 2);
    assert_eq!(s.run(99).unwrap(), None);
}

#[test]
fn latest_batch_wins_history_shows_all() {
    let (_d, s) = tmp();
    let run = s.insert_run(&run_row()).unwrap();
    assert_eq!(s.next_batch(run, "a").unwrap(), 1);
    s.insert_verdict(run, 1, 1, "site", &verdict("a", 2, Some("bad"), true))
        .unwrap();
    assert_eq!(s.next_batch(run, "a").unwrap(), 2);
    assert_eq!(s.next_batch(run, "b").unwrap(), 1);
    s.insert_verdict(run, 2, 1, "site", &verdict("a", 4, Some("worse"), true))
        .unwrap();
    let latest = s.verdict_rows(run, false).unwrap();
    assert_eq!(latest.len(), 1);
    assert_eq!((latest[0].batch, latest[0].score), (2, 4));
    assert_eq!(latest[0].citations, vec!["page#n1".to_string()]);
    assert_eq!(latest[0].citation_hash, "ch1");
    assert_eq!(latest[0].blockers, vec!["worse".to_string()]);
    assert_eq!(latest[0].evidence, Evidence::Seen);
    assert_eq!(s.verdict_rows(run, true).unwrap().len(), 2);
    assert_eq!(s.picks(run, false).unwrap().len(), 1);
    assert_eq!(s.picks(run, true).unwrap().len(), 2);
    // the findings of a run are those of every batch (C3): a later batch never clears an earlier one
    let f = s.findings(run).unwrap();
    assert_eq!(
        f.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(),
        vec!["f-bad", "f-worse"]
    );
    assert_eq!(s.finding("f-bad").unwrap().unwrap().text, "bad");
    assert_eq!(s.finding("nope").unwrap(), None);
}

#[test]
fn repeated_finding_keeps_first_row() {
    let (_d, s) = tmp();
    let r1 = s.insert_run(&run_row()).unwrap();
    let r2 = s.insert_run(&run_row()).unwrap();
    s.insert_verdict(r1, 1, 1, "site", &verdict("a", 2, Some("bad"), false))
        .unwrap();
    s.insert_verdict(r2, 1, 1, "site", &verdict("a", 2, Some("bad"), false))
        .unwrap();
    assert_eq!(s.finding("f-bad").unwrap().unwrap().run_id, r1);
}

#[test]
fn verdict_is_append_only() {
    let (_d, s) = tmp();
    let run = s.insert_run(&run_row()).unwrap();
    s.insert_verdict(run, 1, 1, "site", &verdict("a", 2, None, false))
        .unwrap();
    for sql in ["UPDATE verdict SET score = 5", "DELETE FROM verdict"] {
        let e = s.conn().execute(sql, []).unwrap_err().to_string();
        assert!(e.contains("append-only"), "{sql}: {e}");
    }
}

#[test]
fn pick_is_append_only() {
    let (_d, s) = tmp();
    let run = s.insert_run(&run_row()).unwrap();
    s.insert_verdict(run, 1, 1, "site", &verdict("a", 2, None, true))
        .unwrap();
    for sql in ["UPDATE pick SET choice = 'x'", "DELETE FROM pick"] {
        let e = s.conn().execute(sql, []).unwrap_err().to_string();
        assert!(e.contains("append-only"), "{sql}: {e}");
    }
}

fn decision(finding: &str, outcome: &str) -> DecisionRow {
    DecisionRow {
        id: 0,
        finding_id: finding.into(),
        outcome: outcome.into(),
        why: Some("w".into()),
        row_ref: None,
        by: "me".into(),
        created_at: "t".into(),
        after_run: 0,
    }
}

#[test]
fn decision_is_append_only() {
    let (_d, s) = tmp();
    let run = s.insert_run(&run_row()).unwrap();
    s.insert_verdict(run, 1, 1, "site", &verdict("a", 2, Some("bad"), false))
        .unwrap();
    s.insert_decision(&decision("f-bad", "fixed")).unwrap();
    for sql in [
        "UPDATE decision SET outcome = 'row'",
        "DELETE FROM decision",
    ] {
        let e = s.conn().execute(sql, []).unwrap_err().to_string();
        assert!(e.contains("append-only"), "{sql}: {e}");
    }
}

#[test]
fn dismissed_citations_follow_latest_decision() {
    let (_d, s) = tmp();
    let run = s.insert_run(&run_row()).unwrap();
    s.insert_verdict(run, 1, 1, "site", &verdict("a", 2, Some("bad"), false))
        .unwrap();
    assert!(s.dismissed_citations("site", "a").unwrap().is_empty());
    let d1 = s.insert_decision(&decision("f-bad", "dismissed")).unwrap();
    assert_eq!(
        s.dismissed_citations("site", "a")
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>(),
        vec!["clarity\t\tch1".to_string()],
        "a cited point is its dimension and citation hash"
    );
    assert!(s.dismissed_citations("site", "other").unwrap().is_empty());
    assert!(s.dismissed_citations("other", "a").unwrap().is_empty());
    let d2 = s.insert_decision(&decision("f-bad", "fixed")).unwrap();
    assert!(d2 > d1);
    assert!(s.dismissed_citations("site", "a").unwrap().is_empty());
    assert_eq!(s.decisions("f-bad").unwrap().len(), 2);
    assert_eq!(
        s.latest_decision("f-bad").unwrap().unwrap().outcome,
        "fixed"
    );
    assert_eq!(s.latest_decision("zz").unwrap(), None);
}

#[test]
fn open_refuses_unwritable_root_with_path() {
    let d = tempfile::tempdir().unwrap();
    let file = d.path().join("not-a-dir");
    std::fs::write(&file, b"x").unwrap();
    let err = Store::open(&file).unwrap_err().to_string();
    assert!(err.contains("not-a-dir"), "{err}");
}

#[test]
fn picks_follow_latest_verdict_batch() {
    let (_d, s) = tmp();
    let run = s.insert_run(&run_row()).unwrap();
    s.insert_verdict(run, 1, 1, "site", &verdict("a", 2, None, true))
        .unwrap();
    s.insert_verdict(run, 2, 1, "site", &verdict("a", 3, None, false))
        .unwrap();
    assert!(s.picks(run, false).unwrap().is_empty());
    let all = s.picks(run, true).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].batch, 1);
}
