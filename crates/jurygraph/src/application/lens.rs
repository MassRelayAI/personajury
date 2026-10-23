//! lens add | list (plan Task 4)
use super::{AppError, Result};
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
