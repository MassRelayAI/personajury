//! adapted from comparegraph crates/comparegraph/src/adapters/mcp.rs
//! MCP stdio server: JSON-RPC 2.0, one object per line. Five tools run the same verbs as the CLI and return the same envelope.
//! There is no tool for `setup`, `doctor`, `check` or `decide` (a person decides), and no tool can ask the matrix to
//! average across strata: the property does not exist.
use crate::adapters::cli::{envelope_for, Cli, Cmd};
use crate::adapters::envelope::Envelope;
use crate::application::AppError;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

const PROTOCOL: &str = "2025-06-18";

fn schema(props: Value, required: &[&str]) -> Value {
    json!({ "$schema": "https://json-schema.org/draft/2020-12/schema", "type": "object",
            "properties": props, "required": required, "additionalProperties": false })
}
fn tool(name: &str, desc: &str, input: Value) -> Value {
    json!({ "name": name, "description": desc, "inputSchema": input })
}
fn detail_prop() -> Value {
    json!({"type":"string","enum":["compact","full"],"description":"compact (default): attachments by path; full: text attachments inline"})
}

pub fn tools() -> Vec<Value> {
    vec![
        tool(
            "jurygraph_brief",
            "Compose one deterministic, hashed brief per persona of a panel for a lens and create the run. Returns the run id, each persona's brief (its hash and byte size) and the spend. The calling agent then gives each brief to one subagent.",
            schema(
                json!({
                    "subject": {"type":"string"}, "panel": {"type":"string"}, "lens": {"type":"string"},
                    "stakes": {"description":"a stake for every panel persona: an object persona -> primary|secondary|bystander, or an array of \"persona=stake\"",
                               "oneOf": [
                                   {"type":"object","additionalProperties":{"type":"string","enum":["primary","secondary","bystander"]}},
                                   {"type":"array","items":{"type":"string"}}]},
                    "model": {"type":"string","description":"the model the agent will run the personas on; the run is pinned to it"},
                    "spend": {"type":"integer","minimum":1,"description":"take the first N personas of the panel, primary then secondary then bystander; default all"},
                    "comparators": {"type":"array","items":{"type":"string"},"description":"comparator snapshots this run holds"},
                    "detail": detail_prop() }),
                &["subject", "panel", "lens", "stakes", "model"],
            ),
        ),
        tool(
            "jurygraph_verdict",
            "Ingest one persona's verdict for a run: an object with the persona, its batch of lines (dimension, about, score, anchor_met, evidence kind, citations, blockers) and picks. Returns the stored lines. Append-only. Refusals, by error code: seen_without_citation, citation_unresolved, comparator_not_snapshotted, score_out_of_range, unknown_dimension, unknown_about, bad_anchor, bottom_score_without_blocker, evidence_not_allowed, seen_compare_without_comparator, duplicate_dimension, missing_dimension, dismissed_point_repeated, subject_moved, pick_without_reason, pick_unknown_option, pick_missing, not_found, invalid_arguments; the message names the line.",
            schema(
                json!({ "run": {"type":"integer","minimum":1,"description":"the run id a brief returned"},
                        "verdict": {"type":"object","description":"the persona's verdict JSON, as the brief asked for it"} }),
                &["run", "verdict"],
            ),
        ),
        tool(
            "jurygraph_matrix",
            "Scores per stratum for one or more runs: every score, the median, min and max within a stratum; never a mean, never averaged across strata. Returns a compact envelope; table=true adds the markdown table as data.table.",
            schema(
                json!({ "runs": {"type":"array","items":{"type":"integer","minimum":1},"minItems":1},
                        "table": {"type":"boolean"}, "history": {"type":"boolean","description":"include superseded batches"},
                        "detail": detail_prop() }),
                &["runs"],
            ),
        ),
        tool(
            "jurygraph_delta",
            "What moved between two runs on the same subject and lens: every cell's move, blockers per stratum, the cells the later run lacks (incomplete), and landed. Returns a compact envelope; refuses runs that differ in subject, lens or lens version.",
            schema(
                json!({ "a": {"type":"integer","minimum":1,"description":"the earlier run id"},
                        "b": {"type":"integer","minimum":1,"description":"the later run id"} }),
                &["a", "b"],
            ),
        ),
        tool(
            "jurygraph_should_run",
            "Classify a diff or a list of paths against jurygraph.toml: none, low, high or human, and the spend. Returns a compact envelope; reads only.",
            schema(
                json!({ "diff": {"type":"string","description":"path of a unified diff"},
                        "paths": {"type":"array","items":{"type":"string"}},
                        "config": {"type":"string","description":"path of the jurygraph.toml; default <root>/jurygraph.toml"} }),
                &[],
            ),
        ),
    ]
}

fn req(a: &Value, k: &str) -> Result<String, String> {
    match a.get(k) {
        Some(Value::String(s)) if !s.is_empty() => Ok(s.clone()),
        Some(Value::String(_)) => Err(format!("{k} must not be empty")),
        None | Some(Value::Null) => Err(format!("missing required argument {k}")),
        Some(_) => Err(format!("{k} must be a string")),
    }
}
fn str_arg(a: &Value, k: &str) -> Result<Option<String>, String> {
    match a.get(k) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(format!("{k} must be a string")),
    }
}
fn bool_arg(a: &Value, k: &str) -> Result<bool, String> {
    match a.get(k) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(format!("{k} must be a boolean")),
    }
}
fn id_of(v: &Value, k: &str) -> Result<i64, String> {
    match v.as_i64() {
        Some(n) if n >= 1 => Ok(n),
        _ => Err(format!("{k} must be a run id (an integer >= 1)")),
    }
}
fn id_arg(a: &Value, k: &str) -> Result<i64, String> {
    match a.get(k) {
        None | Some(Value::Null) => Err(format!("missing required argument {k}")),
        Some(v) => id_of(v, k),
    }
}
fn strings_arg(a: &Value, k: &str) -> Result<Vec<String>, String> {
    match a.get(k) {
        None | Some(Value::Null) => Ok(vec![]),
        Some(Value::Array(v)) => v
            .iter()
            .map(|x| {
                x.as_str()
                    .map(String::from)
                    .ok_or_else(|| format!("{k} must be an array of strings"))
            })
            .collect(),
        Some(_) => Err(format!("{k} must be an array of strings")),
    }
}
/// stakes as the CLI's `--stake` strings: the object form becomes `persona=stake` in key order.
fn stakes_arg(a: &Value) -> Result<Vec<String>, String> {
    let list = match a.get("stakes") {
        None | Some(Value::Null) => return Err("missing required argument stakes".into()),
        Some(Value::Object(m)) => m
            .iter()
            .map(|(p, s)| {
                s.as_str().map(|s| format!("{p}={s}")).ok_or_else(|| {
                    format!("stakes.{p} must be a string (primary|secondary|bystander)")
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(Value::Array(_)) => strings_arg(a, "stakes")?,
        Some(_) => {
            return Err(
                "stakes must be an object persona -> stake or an array of \"persona=stake\"".into(),
            )
        }
    };
    if list.is_empty() {
        return Err("stakes must name a stake for every panel persona".into());
    }
    Ok(list)
}

fn command(name: &str, a: &Value) -> Result<Cmd, String> {
    if !a.is_object() {
        return Err("arguments must be an object".into());
    }
    let known: &[&str] = match name {
        "jurygraph_brief" => &[
            "subject",
            "panel",
            "lens",
            "stakes",
            "model",
            "spend",
            "comparators",
            "detail",
        ],
        "jurygraph_verdict" => &["run", "verdict"],
        "jurygraph_matrix" => &["runs", "table", "history", "detail"],
        "jurygraph_delta" => &["a", "b"],
        "jurygraph_should_run" => &["diff", "paths", "config"],
        _ => return Err(format!("unknown tool {name}")),
    };
    if let Some(k) = a
        .as_object()
        .unwrap()
        .keys()
        .find(|k| !known.contains(&k.as_str()))
    {
        return Err(format!("unknown argument {k}"));
    }
    let detail = || str_arg(a, "detail").map(|d| d.unwrap_or_else(|| "compact".into()));
    Ok(match name {
        "jurygraph_brief" => Cmd::Brief {
            subject: req(a, "subject")?,
            panel: req(a, "panel")?,
            lens: req(a, "lens")?,
            spend: match a.get("spend") {
                None | Some(Value::Null) => None,
                Some(v) => match v.as_u64() {
                    Some(n) if n >= 1 => Some(n as usize),
                    _ => return Err("spend must be an integer >= 1".into()),
                },
            },
            stakes: stakes_arg(a)?,
            model: req(a, "model")?,
            comparators: strings_arg(a, "comparators")?,
            detail: detail()?,
        },
        "jurygraph_verdict" => {
            let run = id_arg(a, "run")?;
            match a.get("verdict") {
                Some(v @ Value::Object(_)) => Cmd::Verdict {
                    run,
                    file: None,
                    json: Some(v.to_string()),
                },
                None | Some(Value::Null) => return Err("missing required argument verdict".into()),
                Some(_) => return Err("verdict must be an object".into()),
            }
        }
        "jurygraph_matrix" => {
            let runs = match a.get("runs") {
                Some(Value::Array(v)) if !v.is_empty() => v
                    .iter()
                    .map(|x| id_of(x, "runs[]"))
                    .collect::<Result<Vec<_>, _>>()?,
                Some(Value::Array(_)) => return Err("runs must name at least one run".into()),
                None | Some(Value::Null) => return Err("missing required argument runs".into()),
                Some(_) => return Err("runs must be an array of run ids".into()),
            };
            Cmd::Matrix {
                runs,
                across_strata: false,
                history: bool_arg(a, "history")?,
                table: bool_arg(a, "table")?,
                detail: detail()?,
            }
        }
        "jurygraph_delta" => Cmd::Delta {
            a: id_arg(a, "a")?,
            b: id_arg(a, "b")?,
        },
        _ => Cmd::ShouldRun {
            diff: str_arg(a, "diff")?.map(PathBuf::from),
            paths: strings_arg(a, "paths")?,
            config: str_arg(a, "config")?.map(PathBuf::from),
        },
    })
}

fn call(root: &Path, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let empty = json!({});
    let args = match params.get("arguments") {
        None | Some(Value::Null) => &empty,
        Some(v) => v,
    };
    let (text, is_error) = match command(name, args) {
        Err(m) => {
            let verb = match name.strip_prefix("jurygraph_") {
                Some(v) if tools().iter().any(|t| t["name"] == name) => v.replace('_', "-"),
                _ => "mcp".to_string(),
            };
            (
                Envelope::err(&verb, &AppError::InvalidArguments(m)).to_json(),
                true,
            )
        }
        Ok(cmd) => {
            let env = envelope_for(&Cli {
                root: PathBuf::from(root),
                pretty: false,
                cmd,
            });
            (env.to_json(), !env.ok)
        }
    };
    json!({ "content": [{"type":"text","text":text}], "isError": is_error })
}

fn err(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc":"2.0", "id": id, "error": {"code":code,"message":message} })
}

/// Handle one line; None means no reply (notification).
fn handle(root: &Path, line: &str) -> Option<Value> {
    let v: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return Some(err(Value::Null, -32700, "parse error")),
    };
    let id = v.get("id").cloned();
    let method = v.get("method").and_then(Value::as_str);
    if v.get("jsonrpc").and_then(Value::as_str) != Some("2.0") || method.is_none() || !v.is_object()
    {
        return Some(err(id.unwrap_or(Value::Null), -32600, "invalid request"));
    }
    let id = id?; // notification: no reply
    let method = method.unwrap();
    let params = v.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => json!({ "protocolVersion": PROTOCOL, "capabilities": {"tools": {}},
                                "serverInfo": {"name":"jurygraph","version":env!("CARGO_PKG_VERSION")} }),
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tools() }),
        "tools/call" => call(root, &params),
        _ => return Some(err(id, -32601, "method not found")),
    };
    Some(json!({ "jsonrpc":"2.0", "id": id, "result": result }))
}

pub fn serve<R: BufRead, W: Write>(root: &Path, mut input: R, mut out: W) {
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match input.read_until(b'\n', &mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let line = String::from_utf8_lossy(&buf);
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = handle(root, &line) {
            if writeln!(out, "{reply}").and_then(|_| out.flush()).is_err() {
                break;
            }
        }
    }
}
