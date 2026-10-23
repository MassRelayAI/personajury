//! panel add | resolve (plan Task 4; D9 the seeded draw, D10 the seat ceilings)
use super::{AppError, Result};
use crate::domain::hash::{draw_seed, panel_hash};
use crate::domain::*;
use crate::store::Store;
use serde_json::{json, Value};
use std::path::Path;

pub struct Resolved {
    pub query: String,
    pub ids: Vec<String>,
    /// the stake the panel's own term gives each id (parallel to `ids`); none for a term written without one
    pub stakes: Vec<Option<Stake>>,
    /// the model the panel's own term gives each id (parallel to `ids`, D15)
    pub models: Vec<Option<String>>,
    pub hash: String,
    pub terms: Vec<(Term, usize)>,
    /// the seed the draw used; none when there was no subject to draw for
    pub seed: Option<[u8; 8]>,
}

pub fn hex_seed(seed: [u8; 8]) -> String {
    hex::encode(seed)
}
/// A `--draw` value: 16 hex digits.
pub fn parse_seed(s: &str) -> Result<[u8; 8]> {
    let bad = || AppError::InvalidArguments(format!("--draw {s:?} is not 16 hex digits (8 bytes)"));
    let v = hex::decode(s.trim()).map_err(|_| bad())?;
    v.try_into().map_err(|_| bad())
}

/// The seats a query asks for in all.
pub fn seats(terms: &[Term]) -> usize {
    // an overflowing total is "more than any ceiling", never a wrapped small number
    terms
        .iter()
        .try_fold(0usize, |acc, t| acc.checked_add(t.count))
        .unwrap_or(usize::MAX)
}
/// D10: 12 seats unless the caller says `--allow-large`; 1000 is the most there is.
pub fn check_seats(seats: usize, allow_large: bool) -> Result<()> {
    if seats > SEAT_HARD_MAX {
        return Err(Refusal::PanelTooLarge {
            seats,
            ceiling: SEAT_HARD_MAX,
        }
        .into());
    }
    if seats > SEAT_CEILING && !allow_large {
        return Err(Refusal::PanelTooLarge {
            seats,
            ceiling: SEAT_CEILING,
        }
        .into());
    }
    Ok(())
}

/// Draw a query's seats from the pool. Pure: nothing is stored.
pub fn draw_query(query: &str, personas: &[Persona], seed: Option<[u8; 8]>) -> Result<Resolved> {
    let terms = query::parse(query)?;
    let per_term = query::draw_terms(&terms, personas, seed);
    let mut ids = Vec::new();
    let mut stakes = Vec::new();
    let mut models = Vec::new();
    for (t, got) in terms.iter().zip(&per_term) {
        for id in got {
            ids.push(id.clone());
            stakes.push(t.stake);
            models.push(t.model.clone());
        }
    }
    let hash = panel_hash(query, &ids);
    Ok(Resolved {
        query: query.to_string(),
        ids,
        stakes,
        models,
        hash,
        terms: terms
            .into_iter()
            .zip(per_term.iter().map(Vec::len))
            .collect(),
        seed,
    })
}

/// Resolve the saved query against the current personas and store the result.
pub fn resolve_in(store: &Store, name: &str, seed: Option<[u8; 8]>) -> Result<Resolved> {
    let row = store
        .panel(name)?
        .ok_or_else(|| AppError::NotFound(format!("no panel named {name}")))?;
    let r = draw_query(&row.query, &store.personas()?, seed)?;
    store.set_panel_resolution(name, &r.ids, &r.hash)?;
    Ok(r)
}

pub fn add(root: &Path, name: &str, query: &str, allow_large: bool) -> Result<Value> {
    super::slug("panel name", name)?;
    let terms = query::parse(query)?;
    let n = seats(&terms);
    check_seats(n, allow_large)?;
    let store = Store::open(root)?;
    store.upsert_panel(name, query)?;
    let mut out = json!({"name": name, "query": query, "seats": n});
    if n > SEAT_CEILING {
        out["large"] = json!(true);
    }
    Ok(out)
}
pub fn resolve(root: &Path, name: &str, subject: Option<&str>) -> Result<Value> {
    let store = Store::open(root)?;
    let seed = match subject {
        None => None,
        Some(s) => {
            let sub = store
                .subject(s)?
                .ok_or_else(|| AppError::NotFound(format!("no subject named {s}")))?;
            Some(draw_seed(&sub.name, name))
        }
    };
    let r = resolve_in(&store, name, seed)?;
    let terms: Vec<Value> = r
        .terms
        .iter()
        .map(|(t, g)| {
            let mut v = json!({"tag": t.tag, "wanted": t.count, "got": g});
            if let Some(s) = t.stake {
                v["stake"] = json!(s.name());
            }
            v
        })
        .collect();
    let draw = match (seed, subject) {
        (Some(s), Some(sub)) => json!({"seed": hex_seed(s), "subject": sub}),
        _ => json!("none (no subject)"),
    };
    Ok(json!({"name": name, "query": r.query, "personas": r.ids, "hash": r.hash, "terms": terms, "draw": draw}))
}
