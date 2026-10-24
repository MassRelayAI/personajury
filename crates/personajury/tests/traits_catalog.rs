//! D29: the trait catalog as data. The embedded list equals the files on disk both ways; every file passes `trait add`;
//! every anchor follows "You tend to"; `trait install` adds what the store lacks and versions what changed.
#[path = "traits_support/mod.rs"]
mod support;
use personajury::domain::catalog::CATALOG;
use personajury::domain::traits::{TraitDef, TraitSetDef};
use std::collections::BTreeSet;
use support::*;

fn on_disk() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for dir in ["traits", "trait-sets"] {
        fn walk(p: &std::path::Path, root: &std::path::Path, out: &mut BTreeSet<String>) {
            for e in std::fs::read_dir(p).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    walk(&p, root, out);
                } else if p.extension().is_some_and(|x| x == "toml") && p.file_name().is_some_and(|n| n != "observe-map.toml") {
                    // the observe map is data of its own (D35), embedded as OBSERVE_MAP, not a trait or a set
                    out.insert(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
                }
            }
        }
        walk(&repo().join(dir), &repo(), &mut out);
    }
    out
}

#[test]
fn the_embedded_catalog_equals_the_files_on_disk_both_ways() {
    let disk = on_disk();
    let embedded: BTreeSet<String> = CATALOG.iter().map(|(p, _)| p.to_string()).collect();
    assert_eq!(disk, embedded, "the embedded list and the files differ");
    for (p, text) in CATALOG {
        assert_eq!(std::fs::read_to_string(repo().join(p)).unwrap(), *text, "{p}");
    }
    assert_eq!(embedded.iter().filter(|p| p.starts_with("traits/")).count(), 49);
    assert_eq!(embedded.iter().filter(|p| p.starts_with("trait-sets/")).count(), 8);
}

#[test]
fn every_catalog_file_passes_trait_add_and_every_set_passes_trait_set_add() {
    let ws = tempfile::tempdir().unwrap();
    for (p, _) in CATALOG.iter().filter(|(p, _)| p.starts_with("traits/")) {
        let d = ok(jg(ws.path(), &["trait", "add", "--file", repo().join(p).to_str().unwrap()]));
        assert_eq!(d["origin"], "catalog", "{p}");
    }
    for (p, _) in CATALOG.iter().filter(|(p, _)| p.starts_with("trait-sets/")) {
        ok(jg(ws.path(), &["trait", "set", "add", "--file", repo().join(p).to_str().unwrap()]));
    }
}

#[test]
fn the_phrasing_rule_holds_for_every_anchor_and_option() {
    for (p, text) in CATALOG.iter().filter(|(p, _)| p.starts_with("traits/")) {
        let t = TraitDef::from_toml(text).unwrap();
        let sentences: Vec<&String> = t.anchors.values().chain(t.options.iter().map(|o| &o.sentence)).collect();
        for s in sentences {
            let n = s.chars().count();
            assert!((40..=400).contains(&n), "{p}: {n} characters: {s}");
            assert!(s.chars().next().unwrap().is_lowercase(), "{p}: {s}");
            assert!(!s.starts_with("I "), "{p}: {s}");
            let low = s.to_lowercase();
            assert!(!low.contains("agree"), "{p}: {s}");
            // it reads after "You tend to"
            assert!(!low.starts_with("you "), "{p}: {s}");
        }
        assert_eq!(t.items_copied, Some(false), "{p}");
    }
    let sets: Vec<TraitSetDef> = CATALOG.iter().filter(|(p, _)| p.starts_with("trait-sets/")).map(|(_, t)| TraitSetDef::from_toml(t).unwrap()).collect();
    assert_eq!(sets.len(), 8);
    assert!(sets.iter().all(|s| s.traits.len() == 8 && s.cover.len() == 3));
}

#[test]
fn install_twice_answers_zero_the_second_time() {
    let ws = tempfile::tempdir().unwrap();
    let a = ok(jg(ws.path(), &["trait", "install"]));
    assert_eq!(a["installed"], 57);
    assert_eq!(a["skipped"], 0);
    let b = ok(jg(ws.path(), &["trait", "install"]));
    assert_eq!((b["installed"].as_i64(), b["skipped"].as_i64(), b["versions_added"].as_i64()), (Some(0), Some(57), Some(0)));
    let list = ok(jg(ws.path(), &["trait", "list"]));
    assert_eq!(list["traits"].as_array().unwrap().len(), 49);
    let sets = ok(jg(ws.path(), &["trait", "set", "list"]));
    assert_eq!(sets["sets"].as_array().unwrap().len(), 8);
    let shown = ok(jg(ws.path(), &["trait", "show", "trust.marketing_skepticism"]));
    assert!(shown["anchors"]["3"].as_str().unwrap().starts_with("read a claim"));
}

#[test]
fn a_changed_anchor_is_a_new_version() {
    let ws = tempfile::tempdir().unwrap();
    let store = personajury::store::Store::open(ws.path()).unwrap();
    let v1: Vec<(&str, &str)> = CATALOG.to_vec();
    personajury::application::traits::install_from(&store, &v1, None).unwrap();
    let changed = CATALOG
        .iter()
        .find(|(p, _)| p.ends_with("marketing_skepticism.toml"))
        .unwrap()
        .1
        .replace("assume every claim is written to sell", "treat every claim as an advertisement");
    let mut v2: Vec<(&str, &str)> = CATALOG.to_vec();
    let at = v2.iter().position(|(p, _)| p.ends_with("marketing_skepticism.toml")).unwrap();
    v2[at].1 = &changed;
    let d = personajury::application::traits::install_from(&store, &v2, None).unwrap();
    assert_eq!((d["installed"].as_i64(), d["versions_added"].as_i64(), d["skipped"].as_i64()), (Some(0), Some(1), Some(56)));
    assert_eq!(store.trait_def("trust.marketing_skepticism").unwrap().unwrap().version, 2);
}
