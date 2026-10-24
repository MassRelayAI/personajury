//! trait install | add | list | show and trait set add | list (D29). The catalog is embedded and installed, never seeded.
use super::{AppError, Result};
use crate::domain::catalog::CATALOG;
use crate::domain::traits::{correlated_pairs, TraitDef, TraitSetDef, FAMILIES};
use crate::domain::*;
use crate::store::Store;
use serde_json::{json, Value};
use std::path::Path;

/// The catalog's own definition of a shipped id, as embedded.
fn shipped_def(id: &str) -> Option<TraitDef> {
    crate::domain::catalog::shipped_text(id).and_then(|t| TraitDef::from_toml(t).ok())
}

/// `catalog` for the catalog's own text (or a catalog family's new trait), `edited` for a shipped id whose text differs
/// from the embedded one, `project` for a project's own trait or one generated from material.
fn origin_of(t: &TraitDef) -> &'static str {
    if let Some(shipped) = shipped_def(&t.id) {
        return if shipped.body_and_hash().1 == t.body_and_hash().1 { "catalog" } else { "edited" };
    }
    if t.source.starts_with("material:") || !FAMILIES.contains(&t.family.as_str()) {
        "project"
    } else {
        "catalog"
    }
}

/// Install `catalog` (path, text) into the store: what the store lacks is added; the same hash is skipped; an earlier
/// catalog version is updated to the catalog's text. A trait whose latest version is the user's own (`edited`,
/// `project`) is never overwritten: it is reported in `kept_local`, unless `force` gives the reason to overwrite it,
/// which the new version records (`forced`).
pub fn install_from(store: &Store, catalog: &[(&str, &str)], force: Option<&str>) -> Result<Value> {
    let (mut installed, mut skipped, mut versions_added) = (0, 0, 0);
    let (mut kept_local, mut forced) = (Vec::new(), Vec::new());
    let mut sets = Vec::new();
    for (path, text) in catalog {
        if path.starts_with("trait-sets/") {
            sets.push((path, text));
            continue;
        }
        let t = TraitDef::from_toml(text).map_err(|e| AppError::InvalidArguments(format!("embedded {path}: {e}")))?;
        let (_, hash) = t.body_and_hash();
        match store.trait_latest(&t.id)? {
            None => {
                let _ = store.insert_trait(&t, "catalog")?;
                installed += 1;
            }
            Some((_, h)) if h == hash => skipped += 1,
            Some((origin, _)) if origin == "catalog" => {
                let _ = store.insert_trait(&t, "catalog")?;
                versions_added += 1;
            }
            Some(_) => match force {
                Some(why) => {
                    let _ = store.insert_trait_why(&t, "catalog", Some(why))?;
                    versions_added += 1;
                    forced.push(t.id.clone());
                }
                None => kept_local.push(t.id.clone()),
            },
        }
    }
    // sets after traits: a set names traits the store must hold
    for (path, text) in sets {
        let s = TraitSetDef::from_toml(text).map_err(|e| AppError::InvalidArguments(format!("embedded {path}: {e}")))?;
        let had = store.trait_set(&s.name)?.is_some();
        match store.insert_trait_set(&s)? {
            Err(_) => skipped += 1,
            Ok(_) if had => versions_added += 1,
            Ok(_) => installed += 1,
        }
    }
    let mut out = json!({"installed": installed, "skipped": skipped, "versions_added": versions_added, "kept_local": kept_local});
    if force.is_some() {
        out["forced"] = json!(forced);
    }
    Ok(out)
}
/// `trait install [--force --reason TEXT]`.
pub fn install(root: &Path, force: bool, reason: Option<&str>) -> Result<Value> {
    let why = match (force, reason.map(str::trim)) {
        (false, None) => None,
        (true, Some(r)) if !r.is_empty() => Some(r),
        (true, _) => return Err(AppError::InvalidArguments("--force overwrites local edits: say why with --reason \"<text>\"".into())),
        (false, Some(_)) => return Err(AppError::InvalidArguments("--reason goes with --force".into())),
    };
    let store = Store::open(root)?;
    install_from(&store, CATALOG, why)
}

pub fn add(root: &Path, file: &Path, generated_from: Option<&str>) -> Result<Value> {
    let text = super::read_text(file)?;
    let t = TraitDef::from_toml(&text)?;
    // the protected-attribute screen (D29): family, id and title; a shipped id and title are the catalog's reviewed words
    crate::domain::traits::screen_trait(&t, shipped_def(&t.id).as_ref())?;
    let store = Store::open(root)?;
    if let Some(h) = generated_from {
        if !store.material_exists(h)? {
            return Err(AppError::NotFound(format!("no generated material with hash {h}")));
        }
    }
    let origin = origin_of(&t);
    let (version, added) = match store.insert_trait(&t, origin)? {
        Ok(v) => (v, true),
        Err(v) => (v, false),
    };
    let (_, hash) = t.body_and_hash();
    Ok(json!({"id": t.id, "version": version, "hash": hash, "origin": origin, "added": added}))
}

pub fn set_add(root: &Path, file: &Path) -> Result<Value> {
    let text = super::read_text(file)?;
    let s = TraitSetDef::from_toml(&text)?;
    let store = Store::open(root)?;
    let defs = store.trait_map()?;
    for id in &s.traits {
        if !defs.contains_key(id) {
            return Err(Refusal::UnknownTrait { id: id.clone() }.into());
        }
    }
    let warnings: Vec<Value> = correlated_pairs(&defs, &s.traits)
        .into_iter()
        .map(|(a, b)| json!({"code": "correlated_pair", "a": a, "b": b}))
        .collect();
    let (version, added) = match store.insert_trait_set(&s)? {
        Ok(v) => (v, true),
        Err(v) => (v, false),
    };
    let mut out = json!({"name": s.name, "version": version, "added": added});
    if !warnings.is_empty() {
        out["warnings"] = json!(warnings);
    }
    Ok(out)
}

pub fn list(root: &Path) -> Result<Value> {
    let store = Store::open_existing(root)?;
    let traits: Vec<Value> = store
        .traits()?
        .into_iter()
        .map(|(t, origin, hash, reason)| {
            let mut v = json!({"id": t.id, "version": t.version, "family": t.family, "title": t.title, "shape": t.shape,
                   "evidence": t.evidence, "origin": origin, "hash": hash});
            if let Some(r) = reason {
                v["reason"] = json!(r);
            }
            v
        })
        .collect();
    Ok(json!({"traits": traits}))
}
pub fn show(root: &Path, id: &str) -> Result<Value> {
    let store = Store::open_existing(root)?;
    let t = store.trait_def(id)?.ok_or_else(|| AppError::from(Refusal::UnknownTrait { id: id.to_string() }))?;
    Ok(serde_json::to_value(&t).expect("trait serialises"))
}
pub fn set_list(root: &Path) -> Result<Value> {
    let store = Store::open_existing(root)?;
    let sets: Vec<Value> = store
        .trait_sets()?
        .into_iter()
        .map(|s| json!({"name": s.name, "version": s.version, "traits": s.traits, "cover": s.cover, "note": s.note}))
        .collect();
    Ok(json!({"sets": sets}))
}
