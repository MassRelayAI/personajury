//! trait install | add | list | show and trait set add | list (D29). The catalog is embedded and installed, never seeded.
use super::{AppError, Result};
use crate::domain::catalog::CATALOG;
use crate::domain::traits::{correlated_pairs, TraitDef, TraitSetDef, FAMILIES, REFUSED, REFUSED_FAMILIES};
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
    // the refused family (D29); every text of the trait is read for a sensitive word and labelled, never refused (D39)
    crate::domain::traits::screen_trait(&t)?;
    let sensitive = crate::domain::traits::trait_sensitive(&t, shipped_def(&t.id).as_ref());
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
    Ok(json!({"id": t.id, "version": version, "hash": hash, "origin": origin, "added": added, "sensitive": sensitive}))
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
            // a project's own trait is badged (P), as a credential is (C): the catalog did not vouch for it (D35)
            if origin == "project" {
                v["badge"] = json!("P");
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

/// The most trait files a `trait generate` brief asks for (D35).
pub const MAX_GENERATED_TRAITS: usize = 8;

/// `[a-z0-9_]` slug of a material's name (a file's stem, a folder's name): the family `project.<slug>` and the first
/// segment of every generated id.
fn material_slug(from: &Path) -> String {
    let name = if from.is_dir() { from.file_name() } else { from.file_stem() };
    let raw = name.map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    let mut slug = String::new();
    for c in raw.chars() {
        let c = if c.is_ascii_lowercase() || c.is_ascii_digit() { c } else { '_' };
        if c == '_' && (slug.is_empty() || slug.ends_with('_')) {
            continue;
        }
        slug.push(c);
    }
    let slug = slug.trim_end_matches('_').to_string();
    if slug.is_empty() {
        "material".into()
    } else {
        slug
    }
}

/// `trait generate --from PATH [--count N]` (D35): hash the material and write `.personajury/generate/trait-<hash>.md`,
/// the brief that asks the agent to list the ways the material's readers differ, then to draft `count` trait files in the
/// exact TOML shape. The engine calls no model. What the agent writes enters through `trait add --generated-from <hash>`,
/// which reads every text of the trait for a sensitive attribute and labels it (D39); this verb labels the material's name
/// and slug the same way, and refuses only a refused family (`culture`).
pub fn generate(root: &Path, from: &Path, count: usize) -> Result<Value> {
    if count == 0 || count > MAX_GENERATED_TRAITS {
        return Err(AppError::InvalidArguments(format!("--count is from 1 to {MAX_GENERATED_TRAITS}")));
    }
    let slug = material_slug(from);
    // the material's own name and its slug are read for a sensitive word (`género.md` is `g_nero`): labelled, not refused
    let raw = from.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let sensitive = crate::domain::traits::sensitive_of([raw.as_str(), slug.as_str()]);
    crate::domain::traits::screen_id(&format!("{slug}.example")).map_err(|e| match e {
        Refusal::RefusedTraitId { why, .. } => AppError::from(Refusal::RefusedTraitId {
            id: format!("{slug}.*"),
            why: format!("{why}; the material's name becomes the family of every id, so rename the file or folder"),
        }),
        other => AppError::from(other),
    })?;
    let (hash, sources) = super::persona::material_of(root, from)?;
    let plural = if count == 1 { "" } else { "s" };
    let mut t = String::new();
    t.push_str("personajury-trait-generate 1\n\n");
    t.push_str(&format!("# Write {count} trait file{plural} from this material\n\n"));
    t.push_str(&format!("Material hash: {hash}\n\nThe material, read these files:\n\n"));
    for s in &sources {
        t.push_str(&format!("- {s}\n"));
    }
    t.push_str(
        "\n## Step 1: how the material's readers differ\n\n\
List, in the material's own words, the ways its readers are known to differ: \"procurement versus an engineering lead\", \
\"clinicians under time pressure\". Give the list to the caller as a `readers_differ` list in your answer, each entry with the \
file it came from; it is not part of any trait file. Add no difference the material does not state: where it is silent, \
say so instead of filling the gap.\n",
    );
    t.push_str(&format!(
        "\n## Step 2: the traits\n\nDraft {count} trait file{plural}, one per difference of step 1, each in this shape and no other. A trait is a tendency or a \
standing fact that makes one reader decide differently from another, never a type or a diagnosis. A band trait:\n\n```toml\n\
id = \"{slug}.reads_the_fine_print\"\nversion = 1\nfamily = \"project.{slug}\"\ntitle = \"Reads the fine print before deciding\"\nshape = \"band\"\nscale = [1, 5]\n\
moves = [\"approve_vendor\"]\nevidence = \"project\"\nsource = \"material:{hash}\"\nitems_copied = false\n\
caveats = \"Drawn from the material alone; check it against the readers you know.\"\n\n[anchors]\n\
\"1\" = \"takes the summary page as the whole offer and signs without opening the terms behind it.\"\n\
\"3\" = \"skims the terms for price, dates and anything that looks unusual, and asks only if something stands out.\"\n\
\"5\" = \"reads every clause of the terms before anything else and will not move until each odd one is explained.\"\n```\n\n\
A category trait, when the readers fall into kinds rather than degrees:\n\n```toml\n\
id = \"{slug}.first_question\"\nversion = 1\nfamily = \"project.{slug}\"\ntitle = \"The first question asked of an offer\"\nshape = \"category\"\n\
moves = [\"approve_vendor\"]\nevidence = \"project\"\nsource = \"material:{hash}\"\nitems_copied = false\n\n[[option]]\n\
id = \"cost\"\ntitle = \"What it costs\"\nsentence = \"ask first what the whole thing costs over a year, and judge the rest against that figure.\"\n\n[[option]]\n\
id = \"fit\"\ntitle = \"Whether it fits\"\nsentence = \"ask first whether it fits the systems already in use, and treat price as a detail for later.\"\n```\n\n"
    ));
    t.push_str(&format!(
        "The rules of the shape:\n\n\
- `id` is `{slug}.<name>` in `[a-z0-9_]`; `family` is `project.{slug}`; `evidence` is `project`; keep `source` and `items_copied = false` as written.\n\
- Write each anchor (1, 3 and 5) or option `sentence` as the predicate that follows \"You tend to\": second person, lower case at the start, \
40 to 400 characters, what the reader does or looks at, never a score, a label or \"I agree\".\n\
- A band trait has exactly the anchors 1, 3 and 5; a category trait has at least two options.\n\
- Draw each trait from the material. No typology, no country dimension, no trait that reads a person as a type. {}.\n\
\nThen add each file with:\n\n`personajury trait add --file <file> --generated-from {hash}`\n\nThe engine checks every trait it is given \
and refuses one that does not fit the shape or is on the refused list; every text of it is read for a sensitive attribute, and a hit \
is labelled in the answer (`sensitive`), not refused.\n",
        refused_line()
    ));
    let dir = root.join(".personajury").join("generate");
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(dir.clone(), e))?;
    let path = dir.join(format!("trait-{hash}.md"));
    std::fs::write(&path, t.as_bytes()).map_err(|e| AppError::Io(path.clone(), e))?;
    let rel = format!(".personajury/generate/trait-{hash}.md");
    let store = Store::open(root)?;
    store.insert_material(&hash, &sources, &rel)?;
    Ok(json!({
        "material_hash": hash, "brief": rel, "brief_hash": crate::domain::hash::sha256_hex(t.as_bytes()),
        "count": count, "family": format!("project.{slug}"), "sources": sources, "sensitive": sensitive,
    }))
}

/// The refused list in one line (families and typologies), and the label rule of D39.
fn refused_line() -> String {
    let mut words: Vec<&str> = REFUSED.iter().map(|(w, _)| *w).collect();
    words.extend(REFUSED_FAMILIES);
    format!(
        "Refused ids and families: {}. A trait that names a sensitive attribute (age, gender, ethnicity, religion and the rest) \
is accepted and labelled `sensitive`: personas are simulations, composites, never a real or named person",
        words.join(", ")
    )
}
