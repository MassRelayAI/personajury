//! panel add | resolve (plan Task 4; D9 the seeded draw, D10 the seat ceilings)
use super::{AppError, Result};
use crate::domain::hash::{draw_seed, panel_hash};
use crate::domain::traits::{self, TraitDef};
use crate::domain::*;
use std::collections::BTreeMap;
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

/// Draw a query's seats from the pool, covering `cover` when it is not empty (D30). Pure: nothing is stored.
pub fn draw_query(
    query: &str,
    personas: &[Persona],
    seed: Option<[u8; 8]>,
    cover: &[cover::CoverTrait],
) -> Result<Resolved> {
    let terms = query::parse(query)?;
    let per_term = query::draw_terms_cover(&terms, personas, seed, cover);
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
    let defs = store.trait_map()?;
    check_filters(&row.query, &defs)?;
    let cover_ids = store.panel_cover(name)?;
    let cover = cover_traits(&cover_ids, &defs)?;
    let r = draw_query(&row.query, &store.personas()?, seed, &cover)?;
    store.set_panel_resolution(name, &r.ids, &r.hash)?;
    Ok(r)
}

/// Every filter of a query names a trait the store holds and a value it can hold (D30).
pub fn check_filters(query: &str, defs: &BTreeMap<String, TraitDef>) -> Result<()> {
    for t in query::parse(query)? {
        let raw = query::normalise(std::slice::from_ref(&t));
        traits::check_conds(&t.filter, &raw, defs)?;
    }
    Ok(())
}
/// The cover traits with their definitions; a trait the store lacks is `unknown_trait`, a words trait cannot be covered.
pub fn cover_traits<'a>(ids: &'a [String], defs: &'a BTreeMap<String, TraitDef>) -> Result<Vec<cover::CoverTrait<'a>>> {
    ids.iter()
        .map(|id| {
            let def = defs.get(id).ok_or_else(|| Refusal::UnknownTrait { id: id.clone() })?;
            if def.shape == "words" {
                return Err(AppError::InvalidArguments(format!("{id} holds words; a cover counts bands and options")));
            }
            Ok(cover::CoverTrait { id: id.as_str(), def })
        })
        .collect()
}
/// The coverage of seated personas (D30): `seated` is each persona id with the stratum word it sits in.
pub fn coverage(store: &Store, cover_ids: &[String], seated: &[(String, String)]) -> Result<Option<Value>> {
    if cover_ids.is_empty() {
        return Ok(None);
    }
    let defs = store.trait_map()?;
    let cover = cover_traits(cover_ids, &defs)?;
    let mut people = Vec::new();
    for (id, st) in seated {
        let p = store.persona(id)?.ok_or_else(|| AppError::NotFound(format!("no persona {id}")))?;
        people.push((p, st.clone()));
    }
    let refs: Vec<(&Persona, String)> = people.iter().map(|(p, s)| (p, s.clone())).collect();
    Ok(Some(cover::coverage(&cover, &refs)))
}

pub fn add(root: &Path, name: &str, query: &str, allow_large: bool, cover_ids: &[String]) -> Result<Value> {
    super::slug("panel name", name)?;
    let terms = query::parse(query)?;
    let n = seats(&terms);
    check_seats(n, allow_large)?;
    if cover_ids.len() > traits::MAX_COVER {
        return Err(Refusal::CoverTooLarge { name: name.to_string(), cover: cover_ids.len() }.into());
    }
    let store = Store::open(root)?;
    let defs = store.trait_map()?;
    cover_traits(cover_ids, &defs)?;
    check_filters(query, &defs)?;
    store.upsert_panel(name, query)?;
    store.set_panel_cover(name, cover_ids)?;
    let mut out = json!({"name": name, "query": query, "seats": n});
    if !cover_ids.is_empty() {
        out["cover"] = json!(cover_ids);
    }
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
    let mut out = json!({"name": name, "query": r.query, "personas": r.ids, "hash": r.hash, "terms": terms, "draw": draw});
    let cover_ids = store.panel_cover(name)?;
    let seated: Vec<(String, String)> = r
        .ids
        .iter()
        .zip(&r.stakes)
        .map(|(i, s)| (i.clone(), s.map(|s| s.name()).unwrap_or("unstaked").to_string()))
        .collect();
    if let Some(c) = coverage(&store, &cover_ids, &seated)? {
        out["coverage"] = c;
    }
    Ok(out)
}
