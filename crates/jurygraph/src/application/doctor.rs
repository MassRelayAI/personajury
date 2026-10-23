//! adapted from comparegraph crates/comparegraph/src/adapters/cli.rs (Doctor arm, doctor_brief)
//! doctor: the store, the counts, each client's registration (plan Task 6)
use super::setup;
use super::Result;
use crate::store::{Store, DB, DIR};
use serde_json::{json, Value};
use std::path::Path;

/// A zero-byte file is no store: opening it would migrate it into one.
fn has_store(root: &Path) -> bool {
    std::fs::metadata(root.join(DIR).join(DB)).is_ok_and(|m| m.is_file() && m.len() > 0)
}

fn count(store: &Store, sql: &str) -> Result<i64> {
    Ok(store.conn().query_row(sql, [], |r| r.get::<_, i64>(0))?)
}

fn counts(store: &Store) -> Result<Value> {
    Ok(json!({
        "personas": count(store, "SELECT COUNT(DISTINCT id) FROM persona")?,
        "panels": count(store, "SELECT COUNT(*) FROM panel")?,
        "lenses": count(store, "SELECT COUNT(DISTINCT name) FROM lens")?,
        "subjects": count(store, "SELECT COUNT(DISTINCT name) FROM subject WHERE is_comparator = 0")?,
        "comparators": count(store, "SELECT COUNT(DISTINCT name) FROM subject WHERE is_comparator = 1")?,
        "runs": count(store, "SELECT COUNT(*) FROM run")?,
        "verdict_lines": count(store, "SELECT COUNT(*) FROM verdict")?,
        // belongs to `check`; doctor does not compute it
        "unresolved_findings": null,
    }))
}

pub fn report(root: &Path, bin: Option<&str>) -> Result<Value> {
    let ctx = setup::Ctx::from_env(root, bin);
    let mut limitations = Vec::new();
    if !ctx.explicit_bin && ctx.dev_build {
        limitations.push(json!({ "code": "dev_build", "detail": "compared against a binary in a build directory; pass --bin <installed path>" }));
    }
    let clients: Vec<Value> = setup::registrations(&ctx)
        .iter()
        .chain(setup::extra_registrations(&ctx).iter())
        .map(|r| r.to_json(false))
        .collect();
    let engine = json!({ "model_calls": false, "network": false });
    if !has_store(root) {
        return Ok(json!({
            "store": null, "schema_version": null, "counts": null, "clients": clients,
            "engine": engine, "limitation_items": limitations,
        }));
    }
    let store = Store::open_existing(root)?;
    // a malformed jurygraph.toml is a diagnosis here, not a failure of doctor
    let per = match super::verdict::tokens_per_brief(root) {
        Ok(p) => p,
        Err(e) => {
            limitations.push(json!({ "code": "invalid_config", "detail": e.to_string() }));
            super::brief::DEFAULT_TOKENS_PER_BRIEF
        }
    };
    Ok(json!({
        "store": store.path(), "schema_version": store.schema_version()?, "counts": counts(&store)?,
        "usage": super::verdict::usage_summary_with(&store, &[], per)?,
        "clients": clients, "engine": engine, "limitation_items": limitations,
    }))
}

/// One or two plain lines for a SessionStart hook; nothing without a store.
pub fn brief_lines(root: &Path, bin: Option<&str>) -> String {
    if !has_store(root) {
        return String::new();
    }
    let Ok(store) = Store::open_existing(root) else {
        return String::new();
    };
    let (Ok(personas), Ok(runs)) = (
        count(&store, "SELECT COUNT(DISTINCT id) FROM persona"),
        count(&store, "SELECT COUNT(*) FROM run"),
    ) else {
        return String::new();
    };
    let ctx = setup::Ctx::from_env(root, bin);
    let found = setup::detected_states(&ctx);
    let off: Vec<_> = found
        .iter()
        .filter(|(_, s)| !matches!(s, setup::State::Aligned | setup::State::Plugin))
        .collect();
    let server = if found.is_empty() {
        "no agent client detected".to_string()
    } else if off.is_empty() {
        let names: Vec<_> = found.iter().map(|(c, _)| c.name()).collect();
        format!("server aligned for {}", names.join(", "))
    } else {
        let all: Vec<_> = found
            .iter()
            .map(|(c, s)| format!("{} {}", c.name(), s.name()))
            .collect();
        format!("server: {}", all.join(", "))
    };
    let mut out = format!(
        "jurygraph: {personas} personas, {runs} runs (store {}); {server}\n",
        store.path().display()
    );
    if !off.is_empty() {
        let list: Vec<_> = off
            .iter()
            .map(|(c, s)| format!("{} {}", c.name(), s.name()))
            .collect();
        // `--apply` alone only adds what is missing; a stale or conflicting entry needs `--force`
        let force = if off.iter().all(|(_, s)| *s == setup::State::Missing) {
            ""
        } else {
            " --force"
        };
        out.push_str(&format!(
            "Run `jurygraph setup --apply{force}` (state: {}).\n",
            list.join(", ")
        ));
    }
    out
}
