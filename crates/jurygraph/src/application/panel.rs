//! panel add | resolve (plan Task 4)
use super::{AppError, Result};
use crate::domain::hash::panel_hash;
use crate::domain::*;
use crate::store::Store;
use serde_json::{json, Value};
use std::path::Path;

pub struct Resolved {
    pub query: String,
    pub ids: Vec<String>,
    pub hash: String,
    pub terms: Vec<(Term, usize)>,
}

/// Resolve the saved query against the current personas and store the result.
pub fn resolve_in(store: &Store, name: &str) -> Result<Resolved> {
    let row = store
        .panel(name)?
        .ok_or_else(|| AppError::NotFound(format!("no panel named {name}")))?;
    let terms = query::parse(&row.query)?;
    let personas = store.personas()?;
    let ids = query::resolve(&terms, &personas);
    // what each term took: resolve with the later terms zeroed (they still name their tag), and take the growth
    let mut got = Vec::new();
    let mut before = 0;
    for i in 0..terms.len() {
        let mut t = terms.clone();
        for x in t.iter_mut().skip(i + 1) {
            x.count = 0;
        }
        let n = query::resolve(&t, &personas).len();
        got.push(n - before);
        before = n;
    }
    let hash = panel_hash(&row.query, &ids);
    store.set_panel_resolution(name, &ids, &hash)?;
    Ok(Resolved {
        query: row.query,
        ids,
        hash,
        terms: terms.into_iter().zip(got).collect(),
    })
}

pub fn add(root: &Path, name: &str, query: &str) -> Result<Value> {
    super::slug("panel name", name)?;
    query::parse(query)?;
    let store = Store::open(root)?;
    store.upsert_panel(name, query)?;
    Ok(json!({"name": name, "query": query}))
}
pub fn resolve(root: &Path, name: &str) -> Result<Value> {
    let store = Store::open(root)?;
    let r = resolve_in(&store, name)?;
    let terms: Vec<Value> = r
        .terms
        .iter()
        .map(|(t, g)| json!({"tag": t.tag, "wanted": t.count, "got": g}))
        .collect();
    Ok(json!({"name": name, "query": r.query, "personas": r.ids, "hash": r.hash, "terms": terms}))
}
