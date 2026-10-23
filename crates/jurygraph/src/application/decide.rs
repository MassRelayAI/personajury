//! decide: a person's word on a finding — fixed | dismissed <why> | row <id> (plan Task 5, D4)
use super::verdict::open;
use super::{AppError, Result};
use crate::domain::DecisionRow;
use crate::store::now_iso;
use serde_json::{json, Value};
use std::path::Path;

fn present(s: Option<&str>) -> Option<&str> {
    s.map(str::trim).filter(|s| !s.is_empty())
}

pub fn record(
    root: &Path,
    finding: &str,
    outcome: &str,
    why: Option<&str>,
    row: Option<&str>,
    by: &str,
) -> Result<Value> {
    if !matches!(outcome, "fixed" | "dismissed" | "row") {
        return Err(AppError::InvalidArguments(format!(
            "outcome {outcome:?} is not one of fixed, dismissed, row"
        )));
    }
    if present(Some(by)).is_none() {
        return Err(AppError::InvalidArguments(
            "a decision needs --by <who decided>".into(),
        ));
    }
    if outcome == "dismissed" && present(why).is_none() {
        return Err(AppError::InvalidArguments(
            "dismissed needs --why <the reason>".into(),
        ));
    }
    if outcome == "row" && present(row).is_none() {
        return Err(AppError::InvalidArguments(
            "row needs --row <the register row>".into(),
        ));
    }
    let store = open(root)?;
    if store.finding(finding)?.is_none() {
        return Err(AppError::NotFound(format!("no finding {finding}")));
    }
    let id = store.insert_decision(&DecisionRow {
        id: 0,
        finding_id: finding.to_string(),
        outcome: outcome.to_string(),
        why: present(why).map(String::from),
        row_ref: present(row).map(String::from),
        by: by.trim().to_string(),
        created_at: now_iso(),
        after_run: store.max_run_id()?,
    })?;
    Ok(json!({ "finding": finding, "outcome": outcome, "decision_id": id }))
}
