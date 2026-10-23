//! decide: a person's word on a finding — fixed | dismissed <why> | row <id> (plan Task 5, D4). A finding a human batch
//! raised is decided only with `--by-human` (D19).
use super::verdict::open;
use super::{AppError, Result};
use crate::domain::{DecisionRow, Refusal};
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
    by: Option<&str>,
    by_human: Option<&str>,
) -> Result<Value> {
    if !matches!(outcome, "fixed" | "dismissed" | "row") {
        return Err(AppError::InvalidArguments(format!(
            "outcome {outcome:?} is not one of fixed, dismissed, row"
        )));
    }
    if by.is_some() && by_human.is_some() {
        return Err(AppError::InvalidArguments(
            "give --by or --by-human, not both".into(),
        ));
    }
    let Some(by) = present(by_human).or(present(by)) else {
        return Err(AppError::InvalidArguments(
            "a decision needs --by <who decided> (or --by-human <name> for a person)".into(),
        ));
    };
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
    let Some(f) = store.finding(finding)? else {
        return Err(AppError::NotFound(format!("no finding {finding}")));
    };
    // a human-raised blocker is overruled only by a person's recorded word, never by the agent or the foreman
    if super::verdict::is_human(&f.persona) && present(by_human).is_none() {
        return Err(Refusal::HumanDecisionRequired {
            finding: finding.to_string(),
        }
        .into());
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
