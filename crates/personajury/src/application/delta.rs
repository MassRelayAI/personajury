//! delta <run> <run> (plan Task 5)
use super::verdict::{load_lens, load_run, open, score_rows};
use super::{AppError, Result};
use crate::domain::*;
use serde_json::{json, Value};
use std::path::Path;

pub fn show(root: &Path, a: i64, b: i64) -> Result<Value> {
    show_phase(root, a, b, "initial")
}

pub fn show_phase(root: &Path, a: i64, b: i64, phase: &str) -> Result<Value> {
    let store = open(root)?;
    let (ra, rb) = (load_run(&store, a)?, load_run(&store, b)?);
    if ra.subject_name != rb.subject_name || ra.lens_name != rb.lens_name {
        return Err(AppError::InvalidArguments(format!(
            "delta needs two runs on the same subject and lens: run {a} is {}/{}, run {b} is {}/{}",
            ra.subject_name, ra.lens_name, rb.subject_name, rb.lens_name
        )));
    }
    if ra.lens_version != rb.lens_version {
        return Err(AppError::InvalidArguments(format!(
            "delta needs two runs on the same lens version: run {a} is {} v{}, run {b} is {} v{}",
            ra.lens_name, ra.lens_version, rb.lens_name, rb.lens_version
        )));
    }
    let step = load_lens(&store, &rb)?.anchor_step();
    let before = score_rows(&ra, &super::verdict::rows_in(&store, a, phase)?);
    let after = score_rows(&rb, &super::verdict::rows_in(&store, b, phase)?);
    let d = delta::compute(&before, &after, step, &ra.stakes, &rb.stakes);
    let mut d = d;
    let (paired, unpaired) = delta::split_unpaired(std::mem::take(&mut d.moves));
    d.moves = paired;
    let mut out = serde_json::to_value(&d).expect("delta serialises");
    // personas seated in one run only came or went; they are listed apart from the moves
    out["unpaired"] = json!(unpaired);
    out["a"] = json!(a);
    out["b"] = json!(b);
    out["subject"] = json!(rb.subject_name);
    out["lens"] = json!(rb.lens_name);
    out["step"] = json!(step);
    out["independence"] = json!({"a": ra.independence, "b": rb.independence});
    Ok(out)
}
