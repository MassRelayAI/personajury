//! adapted from comparegraph crates/comparegraph/src/adapters/mcp.rs
//! MCP stdio server: JSON-RPC 2.0, one object per line. Eight tools run the same verbs as the CLI and return the same envelope.
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
            "personajury_brief",
            "Compose one deterministic, hashed brief per persona of a panel for each lens and create the run(s). The panel is a saved one (panel), an ad-hoc query (query, not saved) or a rule's (rule); a stake per persona comes from the panel's terms (\"3 builder=primary\") unless stakes overrides it. With rule, the rule's panel, lenses, spend and detail are the defaults; you may override any of them, and every value that differs, or any brief above 12 seats, needs reason (refusal override_without_reason) and is recorded on the run. Seats are capped at 12 unless allow_large (then 1000, and any brief above 12 seats needs reason, rule or not). The answer carries waves (the persona ids in waves of wave seats, default 30: run one wave, ingest it, then the next), estimate {briefs, tokens} (the cost, before anyone spends it), the draw seed, and large:true above 12 seats. Returns each run id, each persona's brief (its hash and byte size) and the spend. The calling agent then gives each brief to one subagent.",
            schema(
                json!({
                    "subject": {"type":"string"},
                    "panel": {"type":"string","description":"a saved panel"},
                    "query": {"type":"string","description":"an ad-hoc panel, not saved: \"3 builder=primary, 2 others=bystander\""},
                    "lens": {"description":"a lens name, or an array of them (one run each); default: the rule's",
                             "oneOf": [{"type":"string"},{"type":"array","items":{"type":"string"}}]},
                    "rule": {"type":"string","description":"a rule class of personajury.toml: its panel, lenses, spend and detail are the defaults"},
                    "stakes": {"description":"a stake for a panel persona, over the panel's own term stake: an object persona -> primary|secondary|bystander, or an array of \"persona=stake\"",
                               "oneOf": [
                                   {"type":"object","additionalProperties":{"type":"string","enum":["primary","secondary","bystander"]}},
                                   {"type":"array","items":{"type":"string"}}]},
                    "model": {"type":"string","description":"the model the agent will run the personas on; the run is pinned to it"},
                    "spend": {"type":"integer","minimum":1,"description":"take the first N personas of the panel, primary then secondary then bystander; default all"},
                    "comparators": {"type":"array","items":{"type":"string"},"description":"comparator snapshots this run holds"},
                    "detail": detail_prop(),
                    "draw": {"type":"string","description":"the draw's seed as 16 hex digits; default: from the subject's name and the panel's"},
                    "allow_large": {"type":"boolean","description":"allow more than 12 seats (the most is 1000)"},
                    "reason": {"type":"string","description":"why this brief departs from its rule, or runs above 12 seats; recorded on the run"},
                    "by": {"type":"string","description":"who departed from the rule; default agent"},
                    "effort": {"type":"string","enum":["low","medium","high","max"],"description":"how much this run may spend, over personajury.toml's effort (default medium under a rule): low is 3 seats and the rule's first lens, medium up to 7, high up to 12, max the rule's panel (above 12 needs allow_large and reason). A cheap jury on a risky change advises, never applies. Recorded on the run."},
                    "seats": {"type":"string","enum":["auto"],"description":"auto: seat every persona whose serves meets the subject's audience (capped by effort) plus exactly one control persona that does not, instead of a panel; the answer lists seats [{persona, why}] and the run records them. Refusals: audience_unknown, no_matching_personas."},
                    "traits": {"type":"string","description":"render the personas' traits as \"you tend to\" lines: a trait set's name, all, or off (default: the rule's trait_set, else off). Under a rule, a value that differs from the rule's is an override and needs reason. Traits are told, never seen."},
                    "independence": {"type":"string","enum":["independent","sequential"],"description":"independent (default): one context per brief. sequential: one context will read all the briefs; the run is labelled so and apply-check never answers apply for it"},
                    "wave": {"type":"integer","minimum":1,"description":"seats per wave in the answer's waves (default 30): run one wave, ingest it, then the next"} }),
                &["subject", "model"],
            ),
        ),
        tool(
            "personajury_verdict",
            "Ingest one persona's verdict for a run: an object with the persona, its batch of lines (dimension, about, score, anchor_met, evidence kind, citations, blockers) and picks. Returns the stored lines. Append-only. Refusals, by error code: seen_without_citation, citation_unresolved, comparator_not_snapshotted, score_out_of_range, unknown_dimension, unknown_about, bad_anchor, bottom_score_without_blocker, evidence_not_allowed, seen_compare_without_comparator, duplicate_dimension, missing_dimension, dismissed_point_repeated, subject_moved, pick_without_reason, pick_unknown_option, pick_missing, invalid_usage, not_found, invalid_arguments; the message names the line. The verdict may carry usage {input_tokens, output_tokens} (the tokens you spent on it, non-negative integers): matrix and doctor show it beside the estimate, labelled as reported by the agent.",
            schema(
                json!({ "run": {"type":"integer","minimum":1,"description":"the run id a brief returned"},
                        "verdict": {"type":"object","description":"the persona's verdict JSON, as the brief asked for it"} }),
                &["run", "verdict"],
            ),
        ),
        tool(
            "personajury_matrix",
            "Scores per stratum for one or more runs: every score, the median, min and max within a stratum; never a mean, never averaged across strata. Returns a compact envelope; table=true adds the markdown table as data.table.",
            schema(
                json!({ "runs": {"type":"array","items":{"type":"integer","minimum":1},"minItems":1},
                        "slice": {"type":"string","description":"a trait id: also group each stratum's scores by that trait's bands or options (data.slices: groups, cells, thin, unset, separates), counted per stratum, never averaged"},
                        "table": {"type":"boolean"}, "history": {"type":"boolean","description":"include superseded batches"},
                        "detail": detail_prop() }),
                &["runs"],
            ),
        ),
        tool(
            "personajury_graph",
            "Draw the jury of one or more runs of one subject as a single static SVG and return its text (data.svg): personas in three stratum columns, a column per lens of the dimensions they scored (edges coloured by score, the number on hover; a median badge per stratum, never one number), and the findings coloured by decision (open, fixed, dismissed, row). Several runs draw each finding's life as a strip of dots. Deterministic: the same store gives the same bytes. Reads only.",
            schema(
                json!({ "runs": {"type":"array","items":{"type":"integer","minimum":1},"minItems":1,"description":"the run ids; several runs of one subject draw the rounds"} }),
                &["runs"],
            ),
        ),
        tool(
            "personajury_delta",
            "What moved between two runs on the same subject and lens: every cell's move, blockers per stratum, the cells the later run lacks (incomplete), and landed. Returns a compact envelope; refuses runs that differ in subject, lens or lens version.",
            schema(
                json!({ "a": {"type":"integer","minimum":1,"description":"the earlier run id"},
                        "b": {"type":"integer","minimum":1,"description":"the later run id"} }),
                &["a", "b"],
            ),
        ),
        tool(
            "personajury_apply_check",
            "The verdict to obey last, over the runs of one subject: apply, escalate, human or advise, with the reasons as data (human_class; escalate: primary_blocker, split, check_failed, delta_not_landed, human_blocker; advise: incomplete_jury, quorum_unknown, rule_below_sources, overridden_down, effort_below_quorum, independence_sequential, stale_run, delta_unavailable). apply only when every condition is proven: anything unknown, missing or lowered answers advise or worse. human: the change falls under a human class (named by rule, or found in the subject's recorded sources): a person decides, the jury only advises. escalate: run the next quorum up (next_quorum: low to high, high to human). advise: the jury was incomplete, cheap, overridden down, stale or weak; it may advise but never apply. apply: nothing stands in the way. It applies nothing itself and reads only.",
            schema(
                json!({ "runs": {"type":"array","items":{"type":"integer","minimum":1},"minItems":1,"description":"the run ids of one subject; the latest run of each lens is judged, earlier ones are the rounds before it"},
                        "rule": {"type":"string","description":"the rule class of personajury.toml this change falls under: its quorum is the current level"},
                        "calibration": {"type":"string","description":"path of a file written by `calibrate --out`; calibrated:false turns apply into advise (uncalibrated)"} }),
                &["runs"],
            ),
        ),
        tool(
            "personajury_summary",
            "The answer display for the person: plain sentences (the apply-check answer in words, who won with counts per stratum, the weakest dimension, the top blockers in the jurors' own words), a numbers table \"for checking\" (medians as primary·secondary·bystander, never pooled), then a fixed Next sentence. Returns the text in data.text; paste it as is. No bars. With `md` or `html` it also writes that file, at a relative path inside the project root (else invalid_report_dir).",
            schema(
                json!({ "runs": {"type":"array","items":{"type":"integer","minimum":1},"minItems":1,"description":"the run ids of one subject; two rounds of a lens show the change"},
                        "slice": {"type":"string","description":"a trait id: after the sentences, one sentence per stratum on how the jurors of each band of that trait scored the weakest dimension (counts, never a claim about people)"},
                        "width": {"type":"integer","minimum":30,"maximum":200,"description":"characters per line, default 80"},
                        "md": {"type":"string","description":"write the markdown summary here: a relative path inside the project root"},
                        "html": {"type":"string","description":"write the self-contained html here: a relative path inside the project root"} }),
                &["runs"],
            ),
        ),
        tool(
            "personajury_should_run",
            "Classify a diff or a list of paths against personajury.toml: none, low, high or human, and the spend. Returns a compact envelope; reads only.",
            schema(
                json!({ "diff": {"type":"string","description":"path of a unified diff"},
                        "paths": {"type":"array","items":{"type":"string"}},
                        "config": {"type":"string","description":"path of the personajury.toml; default <root>/personajury.toml"},
                        "subject": {"type":"string","description":"classify the recorded source paths of this subject instead of a diff"} }),
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
        None | Some(Value::Null) => return Ok(vec![]),
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
    Ok(list)
}

fn runs_arg(a: &Value) -> Result<Vec<i64>, String> {
    match a.get("runs") {
        Some(Value::Array(v)) if !v.is_empty() => v
            .iter()
            .map(|x| id_of(x, "runs[]"))
            .collect::<Result<Vec<_>, _>>(),
        Some(Value::Array(_)) => Err("runs must name at least one run".into()),
        None | Some(Value::Null) => Err("missing required argument runs".into()),
        Some(_) => Err("runs must be an array of run ids".into()),
    }
}

fn command(name: &str, a: &Value) -> Result<Cmd, String> {
    if !a.is_object() {
        return Err("arguments must be an object".into());
    }
    let known: &[&str] = match name {
        "personajury_brief" => &[
            "subject",
            "panel",
            "query",
            "lens",
            "rule",
            "stakes",
            "model",
            "spend",
            "comparators",
            "detail",
            "draw",
            "allow_large",
            "reason",
            "by",
            "wave",
            "effort",
            "independence",
            "traits",
            "seats",
        ],
        "personajury_verdict" => &["run", "verdict"],
        "personajury_matrix" => &["runs", "table", "history", "detail", "slice"],
        "personajury_delta" => &["a", "b"],
        "personajury_graph" => &["runs"],
        "personajury_apply_check" => &["runs", "rule", "calibration"],
        "personajury_summary" => &["runs", "width", "md", "html", "slice"],
        "personajury_should_run" => &["diff", "paths", "config", "subject"],
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
        "personajury_brief" => Cmd::Brief {
            subject: req(a, "subject")?,
            panel: str_arg(a, "panel")?,
            query: str_arg(a, "query")?,
            lens: match a.get("lens") {
                None | Some(Value::Null) => vec![],
                Some(Value::String(s)) => vec![s.clone()],
                Some(Value::Array(_)) => strings_arg(a, "lens")?,
                Some(_) => return Err("lens must be a string or an array of strings".into()),
            },
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
            detail: str_arg(a, "detail")?,
            rule: str_arg(a, "rule")?,
            draw: str_arg(a, "draw")?,
            allow_large: bool_arg(a, "allow_large")?,
            reason: str_arg(a, "reason")?,
            by: str_arg(a, "by")?,
            effort: str_arg(a, "effort")?,
            independence: str_arg(a, "independence")?,
            traits: str_arg(a, "traits")?,
            seats: str_arg(a, "seats")?,
            wave: match a.get("wave") {
                None | Some(Value::Null) => None,
                Some(v) => match v.as_u64() {
                    Some(n) if n >= 1 => Some(n as usize),
                    _ => return Err("wave must be an integer >= 1".into()),
                },
            },
        },
        "personajury_verdict" => {
            let run = id_arg(a, "run")?;
            match a.get("verdict") {
                Some(v @ Value::Object(_)) => Cmd::Verdict {
                    run,
                    file: None,
                    json: Some(v.to_string()),
                    by_human: None,
                },
                None | Some(Value::Null) => return Err("missing required argument verdict".into()),
                Some(_) => return Err("verdict must be an object".into()),
            }
        }
        "personajury_matrix" => {
            let runs = runs_arg(a)?;
            Cmd::Matrix {
                runs,
                across_strata: false,
                history: bool_arg(a, "history")?,
                table: bool_arg(a, "table")?,
                detail: detail()?,
                slice: str_arg(a, "slice")?,
            }
        }
        "personajury_graph" => Cmd::Graph {
            runs: runs_arg(a)?,
            svg: true,
            out: None,
        },
        "personajury_apply_check" => Cmd::ApplyCheck {
            runs: runs_arg(a)?,
            latest: false,
            subject: None,
            rule: str_arg(a, "rule")?,
            calibration: str_arg(a, "calibration")?.map(PathBuf::from),
        },
        "personajury_summary" => Cmd::Summary {
            runs: runs_arg(a)?,
            html: str_arg(a, "html")?.map(PathBuf::from),
            md: str_arg(a, "md")?.map(PathBuf::from),
            save: false,
            confined: true,
            slice: str_arg(a, "slice")?,
            width: match a.get("width") {
                None | Some(Value::Null) => None,
                Some(v) => match v.as_u64() {
                    Some(n) => Some(n as usize),
                    _ => return Err("width must be an integer".into()),
                },
            },
        },
        "personajury_delta" => Cmd::Delta {
            a: id_arg(a, "a")?,
            b: id_arg(a, "b")?,
        },
        _ => Cmd::ShouldRun {
            diff: str_arg(a, "diff")?.map(PathBuf::from),
            paths: strings_arg(a, "paths")?,
            config: str_arg(a, "config")?.map(PathBuf::from),
            subject: str_arg(a, "subject")?,
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
            let verb = match name.strip_prefix("personajury_") {
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
                                "serverInfo": {"name":"personajury","version":env!("CARGO_PKG_VERSION")} }),
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
