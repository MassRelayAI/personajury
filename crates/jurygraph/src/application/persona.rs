//! persona add | generate | list (plan Task 4)
use super::{AppError, Result};
use crate::domain::hash::{canonical_json, sha256_hex};
use crate::domain::*;
use crate::store::Store;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// A path as the briefs show it: relative to the workspace root when inside it, else as given.
pub fn display_path(root: &Path, p: &Path) -> String {
    if let (Ok(r), Ok(a)) = (std::fs::canonicalize(root), std::fs::canonicalize(p)) {
        if let Ok(rel) = a.strip_prefix(&r) {
            return rel.to_string_lossy().replace('\\', "/");
        }
    }
    p.to_string_lossy().replace('\\', "/")
}

pub fn add(root: &Path, file: &Path, generated_from: Option<&str>) -> Result<Value> {
    let text = super::read_text(file)?;
    let pf: PersonaFile = serde_json::from_str(&text).map_err(|e| {
        AppError::InvalidArguments(format!("{} is not a persona file: {e}", file.display()))
    })?;
    super::slug("persona id", &pf.id)?;
    let store = Store::open(root)?;
    let origin = match generated_from {
        None => Origin::Hand,
        Some(h) => {
            if !store.material_exists(h)? {
                return Err(AppError::NotFound(format!(
                    "no generated material with hash {h}; run `jurygraph persona generate` first"
                )));
            }
            Origin::GeneratedFrom(h.to_string())
        }
    };
    let mut tags: Vec<String> = pf.tags.iter().map(|t| t.trim().to_lowercase()).collect();
    tags.sort();
    tags.dedup();
    let profile = json!({
        "role": pf.role, "context": pf.context, "daily_tasks": pf.daily_tasks,
        "fears": pf.fears, "never_reads": pf.never_reads, "decides_by": pf.decides_by,
    });
    let whole = serde_json::to_value(&pf).expect("persona file serialises");
    let hash = sha256_hex(canonical_json(&whole).as_bytes());
    let persona = Persona {
        id: pf.id.clone(),
        version: 0,
        name: pf.name.clone(),
        tags,
        stake_rules: pf.stake_rules.clone(),
        profile_json: canonical_json(&profile),
        origin: origin.clone(),
        hash: hash.clone(),
    };
    let version = store.insert_persona(&persona)?;
    Ok(json!({"id": pf.id, "version": version, "hash": hash, "origin": origin.to_db()}))
}

fn collect(dir: &Path, rel: &str, out: &mut Vec<(String, PathBuf)>) -> Result<()> {
    let rd = std::fs::read_dir(dir).map_err(|e| AppError::Io(dir.to_path_buf(), e))?;
    for entry in rd {
        let entry = entry.map_err(|e| AppError::Io(dir.to_path_buf(), e))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let r = if rel.is_empty() {
            name
        } else {
            format!("{rel}/{name}")
        };
        let meta = std::fs::metadata(&path).map_err(|e| AppError::Io(path.clone(), e))?;
        if meta.is_dir() {
            collect(&path, &r, out)?;
        } else if meta.is_file() {
            out.push((r, path));
        }
    }
    Ok(())
}

/// Hash the material, write `.jurygraph/generate/<hash>.md` (the brief asking the agent for `count` personas), record it.
pub fn generate(root: &Path, from: &Path, count: usize) -> Result<Value> {
    if count == 0 {
        return Err(AppError::InvalidArguments(
            "--count must be at least 1".into(),
        ));
    }
    let meta = std::fs::metadata(from).map_err(|e| AppError::Io(from.to_path_buf(), e))?;
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    if meta.is_dir() {
        collect(from, "", &mut files)?;
    } else if meta.is_file() {
        let name = from
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        files.push((name, from.to_path_buf()));
    } else {
        return Err(AppError::InvalidArguments(format!(
            "{} is neither a file nor a folder",
            from.display()
        )));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    if files.is_empty() {
        return Err(AppError::InvalidArguments(format!(
            "{} holds no files",
            from.display()
        )));
    }
    let base = display_path(root, from);
    let mut lines = String::new();
    let mut sources = Vec::new();
    for (rel, path) in &files {
        let bytes = std::fs::read(path).map_err(|e| AppError::Io(path.clone(), e))?;
        lines.push_str(&format!("{rel}\t{}\n", sha256_hex(&bytes)));
        sources.push(if meta.is_dir() {
            format!("{base}/{rel}")
        } else {
            base.clone()
        });
    }
    let hash = sha256_hex(lines.as_bytes());
    let mut t = String::new();
    t.push_str("jurygraph-generate 1\n\n");
    t.push_str(&format!(
        "# Write {count} persona{} from this material\n\n",
        if count == 1 { "" } else { "s" }
    ));
    t.push_str(&format!(
        "Material hash: {hash}\n\nThe material, read these files:\n\n"
    ));
    for s in &sources {
        t.push_str(&format!("- {s}\n"));
    }
    t.push_str(
        "\n## Step 1: the roles the material names\n\n\
List the roles, the sectors and the objections the material names or implies: who is spoken to, who pays, who is \
turned away, what each of them would push back on. Give the list to the caller as a `roles` list in your answer; \
it is not part of any persona file. Mark which role is the product's user, which one buys or approves it, and which \
only passes by: those become the primary, secondary and bystander seats of a panel. The number of personas below is \
the caller's `--count`, never the number of roles you found; a role may be left without a persona, and one persona \
may combine two roles.\n",
    );
    t.push_str(&format!(
        "\n## Step 2: the personas\n\nWrite {count} personas: real kinds of people who would meet a product like the one this material describes, spread over the roles of step 1 and over the tags a panel will need. \
Write one JSON file per persona, named `<id>.json`. Each file holds exactly these keys and no others:\n\n"
    ));
    for (k, m) in [
        ("id", "a short lower-case slug, unique, e.g. `builder-ana`"),
        ("name", "the person's name"),
        ("tags", "an array of lower-case tags a panel query can name, e.g. [\"builder\"]; at least one"),
        ("role", "who this person is and what they do, in a sentence or two"),
        ("context", "why and how they arrive at the product: the situation, the time they have, what they compare it with"),
        ("daily_tasks", "what their working day looks like, in the tools and habits that matter here"),
        ("fears", "what they are afraid of getting wrong, losing or paying for"),
        ("never_reads", "what they skip or distrust on a page"),
        ("decides_by", "the order in which they look at things and what tips their decision"),
        ("stake_rules", "when this person is primary (would use it), secondary (buys, approves or supports it) or bystander (passes by)"),
    ] {
        t.push_str(&format!("- `{k}`: {m}\n"));
    }
    t.push_str(&format!(
        "\nBase every persona on the material above; do not invent facts it does not support. \
Then add each file with:\n\n`jurygraph persona add --file <file> --generated-from {hash}`\n"
    ));
    let dir = root.join(".jurygraph").join("generate");
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(dir.clone(), e))?;
    let path = dir.join(format!("{hash}.md"));
    std::fs::write(&path, t.as_bytes()).map_err(|e| AppError::Io(path.clone(), e))?;
    let rel = format!(".jurygraph/generate/{hash}.md");
    let store = Store::open(root)?;
    store.insert_material(&hash, &sources, &rel)?;
    Ok(json!({"material_hash": hash, "brief": rel, "sources": sources}))
}

pub fn list(root: &Path) -> Result<Value> {
    let store = Store::open_existing(root)?;
    let items: Vec<Value> = store
        .personas()?
        .into_iter()
        .map(|p| json!({"id": p.id, "version": p.version, "name": p.name, "tags": p.tags, "origin": p.origin.to_db()}))
        .collect();
    Ok(json!({"items": items}))
}
