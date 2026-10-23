//! subject add and comparator add (plan Task 4). Attachments are copied by hash; nothing is ever fetched.
use super::{AppError, Result};
use crate::domain::hash::{sha256_hex, subject_hash};
use crate::domain::*;
use crate::store::Store;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn media_of(label: &str, bytes: &[u8]) -> &'static str {
    let ext = label
        .rsplit_once('.')
        .map(|(_, e)| e.to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "json" => {
            let has_nodes = serde_json::from_slice::<Value>(bytes)
                .ok()
                .and_then(|v| v.get("nodes").map(Value::is_array))
                .unwrap_or(false);
            if has_nodes {
                "aria-json"
            } else {
                "file"
            }
        }
        "png" | "jpg" | "jpeg" | "webp" => "image",
        "diff" | "patch" => "diff",
        _ => "file",
    }
}

pub fn add(
    root: &Path,
    name: &str,
    kind: SubjectKind,
    attach: &[PathBuf],
    comparator: bool,
) -> Result<Value> {
    super::slug(
        if comparator {
            "comparator name"
        } else {
            "subject name"
        },
        name,
    )?;
    if attach.is_empty() {
        return Err(AppError::InvalidArguments(
            "attach at least one file".into(),
        ));
    }
    let mut atts: Vec<Attachment> = Vec::new();
    let mut blobs: Vec<(String, Vec<u8>)> = Vec::new();
    for p in attach {
        let meta = std::fs::metadata(p).map_err(|e| super::io_err(p, e))?;
        if !meta.is_file() {
            return Err(AppError::InvalidArguments(format!(
                "{} is not a regular file",
                p.display()
            )));
        }
        let abs = std::fs::canonicalize(p).map_err(|e| AppError::Io(p.clone(), e))?;
        let label = abs
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .ok_or_else(|| {
                AppError::InvalidArguments(format!("{} has no file name", p.display()))
            })?;
        // a label is cited as `<label>[#locator]` and offered as an option beside `cmp:<name>` (final review, minor)
        if label.contains('#') || label.starts_with("cmp:") {
            return Err(AppError::InvalidArguments(format!(
                "{label}: an attachment label (the file name) may not contain `#` or start with `cmp:`; rename the file"
            )));
        }
        if atts.iter().any(|a| a.label == label) {
            return Err(AppError::InvalidArguments(format!(
                "two attachments are labelled {label}; the label is the file name and must be unique in a subject"
            )));
        }
        let bytes = std::fs::read(&abs).map_err(|e| AppError::Io(abs.clone(), e))?;
        let hash = sha256_hex(&bytes);
        atts.push(Attachment {
            media: media_of(&label, &bytes).to_string(),
            label,
            hash: hash.clone(),
            source_path: abs.to_string_lossy().into_owned(),
        });
        blobs.push((hash, bytes));
    }
    atts.sort_by(|a, b| a.label.cmp(&b.label));
    let store = Store::open(root)?;
    if let Some(existing) = store.subject(name)? {
        if existing.is_comparator != comparator {
            let (have, want) = if existing.is_comparator {
                ("comparator", "subject")
            } else {
                ("subject", "comparator")
            };
            return Err(AppError::InvalidArguments(format!(
                "{name} is already a {have}; a {want} cannot share its name"
            )));
        }
    }
    let dir = root.join(".jurygraph").join("blobs");
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(dir.clone(), e))?;
    for (hash, bytes) in &blobs {
        let p = dir.join(hash);
        if !p.exists() {
            std::fs::write(&p, bytes).map_err(|e| AppError::Io(p.clone(), e))?;
        }
    }
    let hash = subject_hash(kind, &atts);
    let subject = Subject {
        name: name.to_string(),
        version: 0,
        kind,
        is_comparator: comparator,
        hash: hash.clone(),
        attachments: atts.clone(),
    };
    let version = store.insert_subject(&subject)?;
    let attachments: Vec<Value> = atts
        .iter()
        .map(|a| json!({"label": a.label, "media": a.media, "hash": a.hash}))
        .collect();
    Ok(json!({
        "name": name, "version": version, "kind": kind.name(), "comparator": comparator,
        "hash": hash, "attachments": attachments,
    }))
}
