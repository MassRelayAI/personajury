//! lens add | list (plan Task 4)
use super::{AppError, Result};
use crate::domain::hash::sha256_hex;
use crate::domain::*;
use crate::store::Store;
use serde_json::{json, Value};
use std::path::Path;

pub fn add(root: &Path, file: &Path) -> Result<Value> {
    let text = super::read_text(file)?;
    let lens = Lens::from_toml(&text).map_err(|e| {
        AppError::InvalidArguments(format!("{} is not a lens: {e}", file.display()))
    })?;
    let store = Store::open(root)?;
    let version = store.insert_lens(&lens)?;
    Ok(json!({"name": lens.name, "version": version}))
}
pub fn list(root: &Path) -> Result<Value> {
    let store = Store::open_existing(root)?;
    let items: Vec<Value> = store
        .lenses()?
        .into_iter()
        .map(|l| {
            json!({
                "name": l.name,
                "version": l.version,
                "dimensions": l.dimensions.iter().map(|d| d.id.clone()).collect::<Vec<_>>(),
                "asks_pick": l.asks_pick,
                "compares": l.compares,
                "evidence": l.evidence.iter().map(|e| e.name()).collect::<Vec<_>>(),
            })
        })
        .collect();
    Ok(json!({"items": items}))
}

/// The most dimensions a generated lens asks for: a lens a persona can hold in one read.
pub const MAX_GENERATED_DIMENSIONS: usize = 12;

/// Hash the material and write `.personajury/generate/lens-<hash>.md`: the brief that asks the agent to say what the
/// material calls good, then to draft a lens of `dimensions` dimensions in the exact TOML shape, with anchors in words
/// (D14). The engine calls no model; `lens add` validates what the agent writes, and `standard` records the material.
pub fn generate(root: &Path, from: &Path, dimensions: usize) -> Result<Value> {
    if dimensions == 0 || dimensions > MAX_GENERATED_DIMENSIONS {
        return Err(AppError::InvalidArguments(format!(
            "--dimensions is from 1 to {MAX_GENERATED_DIMENSIONS}"
        )));
    }
    let (hash, sources) = super::persona::material_of(root, from)?;
    let mut t = String::new();
    t.push_str("personajury-lens-generate 1\n\n");
    t.push_str(&format!(
        "# Write a lens of {dimensions} dimension{} from this material\n\n",
        if dimensions == 1 { "" } else { "s" }
    ));
    t.push_str(&format!("Material hash: {hash}\n\nThe material, read these files:\n\n"));
    for s in &sources {
        t.push_str(&format!("- {s}\n"));
    }
    t.push_str(
        "\n## Step 1: what the material says good looks like\n\n\
List what the material says good looks like and what it calls failure: the qualities, thresholds, rules and examples it \
holds up, each with the file it came from. Give the list to the caller as a `good_looks_like` list in your answer; it is \
not part of the lens file. Add no standard the material does not state: where it is silent, say so instead of filling the \
gap.\n",
    );
    t.push_str(&format!(
        "\n## Step 2: the lens\n\nDraft one lens file, `<name>.toml`, of exactly {dimensions} dimension{}, in this shape and no other:\n\n```toml\n",
        if dimensions == 1 { "" } else { "s" }
    ));
    t.push_str(&format!(
        "name = \"my-lens\"            # a lower-case slug: letters, digits, . _ -\nversion = 1\nscale = [1, 5]\nasks_pick = false\nevidence = [\"seen\", \"told\", \"recalled\"]\nstandard = \"material:{hash}\"\n"
    ));
    for i in 1..=dimensions {
        t.push_str(&format!(
            "\n[[dimension]]\nid = \"quality-{i}\"\ntitle = \"the quality the material names, as a question a persona can answer\"\nanchors = {{ \"1\" = \"what failing this looks like, in words\", \"3\" = \"what passing with a flaw looks like, in words\", \"5\" = \"what the material calls good, in words\" }}\n"
        ));
    }
    t.push_str("```\n\n");
    t.push_str(
        "The rules of the shape:\n\n\
- Each dimension is one quality of step 1, with a lower-case slug `id` and a `title` a persona can answer.\n\
- Write the anchors at 1, 3 and 5, the anchor points of the scale, in words: one sentence each, what a persona would see \
or do at that score, worst first. A number or a grade is not an anchor. A verdict will say which anchor it met, so each \
sentence must be something a persona can tell apart from the next.\n\
- Keep `standard` as written: it records the material this lens was drawn from.\n\
- `evidence` lists the classes a line may carry: `seen` (a persona looked at an attachment of the run and cites it), \
`told` (text the brief supplied), `recalled` (memory). Drop a class the material gives no way to check.\n",
    );
    t.push_str(
        "\nThen add the file with:\n\n`personajury lens add --file <file>`\n\nThe engine refuses a lens that is not in this shape.\n",
    );
    let dir = root.join(".personajury").join("generate");
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(dir.clone(), e))?;
    let path = dir.join(format!("lens-{hash}.md"));
    std::fs::write(&path, t.as_bytes()).map_err(|e| AppError::Io(path.clone(), e))?;
    let rel = format!(".personajury/generate/lens-{hash}.md");
    let store = Store::open(root)?;
    store.insert_material(&hash, &sources, &rel)?;
    Ok(json!({
        "material_hash": hash, "brief": rel, "brief_hash": sha256_hex(t.as_bytes()),
        "dimensions": dimensions, "sources": sources,
    }))
}
