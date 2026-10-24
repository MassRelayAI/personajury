//! adapted from comparegraph crates/comparegraph/tests/mcp.rs
//! The MCP server: protocol, seven tools, and the envelope passed through unchanged. Verb behaviour is not tested here.
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn bin() -> String {
    assert_cmd::cargo::cargo_bin("personajury")
        .to_string_lossy()
        .into_owned()
}
/// Feed raw lines, close stdin, return (exit success, stdout lines parsed).
fn session(ws: &std::path::Path, lines: &[String]) -> (bool, Vec<Value>) {
    let mut ch = Command::new(bin())
        .arg("--root")
        .arg(ws)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = ch.stdin.take().unwrap();
    for l in lines {
        writeln!(stdin, "{l}").unwrap();
    }
    drop(stdin);
    let rd = BufReader::new(ch.stdout.take().unwrap());
    let out: Vec<Value> = rd
        .lines()
        .map(|l| serde_json::from_str(&l.unwrap()).expect("stdout line is JSON"))
        .collect();
    (ch.wait().unwrap().success(), out)
}
fn req(id: i64, method: &str, params: Value) -> String {
    json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}).to_string()
}
fn call(id: i64, name: &str, args: Value) -> String {
    req(id, "tools/call", json!({"name": name, "arguments": args}))
}
fn tool_env(out: &Value) -> Value {
    serde_json::from_str(out["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}
fn tools_list() -> Vec<Value> {
    let ws = tempfile::tempdir().unwrap();
    let (_, out) = session(ws.path(), &[req(1, "tools/list", json!({}))]);
    out[0]["result"]["tools"].as_array().unwrap().clone()
}

#[test]
fn initialize_and_list_the_tools() {
    let ws = tempfile::tempdir().unwrap();
    let (ok, out) = session(
        ws.path(),
        &[
            req(1, "initialize", json!({})),
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string(),
            req(2, "tools/list", json!({})),
            req(3, "ping", json!({})),
        ],
    );
    assert!(ok);
    assert_eq!(out.len(), 3, "the notification gets no reply");
    assert_eq!(out[0]["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(out[0]["result"]["serverInfo"]["name"], "personajury");
    assert!(out[0]["result"]["capabilities"]["tools"].is_object());
    let tools = out[1]["result"]["tools"].as_array().unwrap();
    let mut names: Vec<_> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "personajury_apply_check",
            "personajury_brief",
            "personajury_deliberate",
            "personajury_delta",
            "personajury_graph",
            "personajury_matrix",
            "personajury_should_run",
            "personajury_summary",
            "personajury_trait_list",
            "personajury_trait_observed",
            "personajury_trait_suggest_check",
            "personajury_verdict"
        ]
    );
    for t in tools {
        assert_eq!(t["inputSchema"]["type"], "object");
        assert_eq!(t["inputSchema"]["additionalProperties"], false);
        assert!(
            t["inputSchema"]["properties"].is_object() && t["inputSchema"]["required"].is_array()
        );
        assert!(!t["description"].as_str().unwrap().is_empty());
    }
    assert_eq!(out[2]["id"], 3);
}

#[test]
fn the_trait_list_tool_answers_the_catalog_and_takes_no_arguments_but_a_set_flag() {
    let ws = tempfile::tempdir().unwrap();
    let _ = std::process::Command::new(assert_cmd::cargo::cargo_bin("personajury"))
        .args(["--root", ws.path().to_str().unwrap(), "trait", "install"])
        .output()
        .unwrap();
    let (_, out) = session(ws.path(), &[call(1, "personajury_trait_list", json!({}))]);
    let text = out[0]["result"]["content"][0]["text"].as_str().unwrap();
    let env: Value = serde_json::from_str(text).unwrap();
    assert_eq!(env["ok"], true, "{env}");
    assert_eq!(env["data"]["traits"].as_array().unwrap().len(), 49);
    let (_, out) = session(ws.path(), &[call(1, "personajury_trait_list", json!({"sets": true}))]);
    let env: Value = serde_json::from_str(out[0]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(env["data"]["sets"].as_array().unwrap().len(), 8);
}

/// A workspace with the catalog and one persona whose prose can be quoted.
fn prose_ws() -> tempfile::TempDir {
    let ws = tempfile::tempdir().unwrap();
    let cli = |args: &[&str]| {
        std::process::Command::new(assert_cmd::cargo::cargo_bin("personajury")).arg("--root").arg(ws.path()).args(args).output().unwrap()
    };
    cli(&["trait", "install"]);
    let f = ws.path().join("p.json");
    std::fs::write(
        &f,
        json!({"id": "ravi", "name": "Ravi", "tags": ["builder"], "role": "An engineer who reads the pricing page first", "context": "c",
               "daily_tasks": "d", "fears": "f", "never_reads": "n", "decides_by": "b", "stake_rules": "s"})
        .to_string(),
    )
    .unwrap();
    cli(&["persona", "add", "--file", f.to_str().unwrap()]);
    ws
}

#[test]
fn the_trait_observed_tool_reads_and_takes_a_persona_and_runs() {
    let ws = prose_ws();
    let t = tools_list().into_iter().find(|t| t["name"] == "personajury_trait_observed").unwrap();
    assert_eq!(t["inputSchema"]["required"], json!(["persona"]));
    let (_, out) = session(ws.path(), &[call(1, "personajury_trait_observed", json!({"persona": "ravi"}))]);
    let env = tool_env(&out[0]);
    assert_eq!(env["ok"], true, "{env}");
    assert_eq!(env["data"]["persona"], "ravi");
    assert_eq!(env["data"]["traits"], json!([]));
    let (_, out) = session(ws.path(), &[call(1, "personajury_trait_observed", json!({"persona": ["ravi"], "runs": [1]}))]);
    assert_eq!(tool_env(&out[0])["ok"], false, "run 1 does not exist");
    let (_, out) = session(ws.path(), &[call(1, "personajury_trait_observed", json!({"persona": "ravi", "bogus": 1}))]);
    assert!(out[0]["error"].is_object() || out[0]["result"]["isError"] == true, "{}", out[0]);
}

#[test]
fn the_trait_suggest_check_tool_answers_the_merged_file_and_writes_nothing() {
    let ws = prose_ws();
    let t = tools_list().into_iter().find(|t| t["name"] == "personajury_trait_suggest_check").unwrap();
    assert_eq!(t["inputSchema"]["required"], json!(["suggestion"]));
    let sug = json!({"persona": "ravi", "traits": {"consumer.price_consciousness": 5},
                     "evidence": {"consumer.price_consciousness": "reads the pricing page first"}});
    let (_, out) = session(ws.path(), &[call(1, "personajury_trait_suggest_check", json!({"suggestion": sug}))]);
    let env = tool_env(&out[0]);
    assert_eq!(env["ok"], true, "{env}");
    assert_eq!(env["data"]["persona_file"]["traits"]["consumer.price_consciousness"], 5);
    let bad = json!({"persona": "ravi", "traits": {"consumer.price_consciousness": 5}, "evidence": {"consumer.price_consciousness": "made up out of thin air"}});
    let (_, out) = session(ws.path(), &[call(1, "personajury_trait_suggest_check", json!({"suggestion": bad}))]);
    assert_eq!(tool_env(&out[0])["error"]["code"], "quote_not_found");
    assert!(!ws.path().join(".personajury/generate").exists(), "nothing was written");
}

#[test]
fn unknown_method_and_parse_error() {
    let ws = tempfile::tempdir().unwrap();
    let (ok, out) = session(
        ws.path(),
        &[
            req(1, "nope", json!({})),
            "{not json".into(),
            req(2, "ping", json!({})),
        ],
    );
    assert!(ok);
    assert_eq!(out[0]["error"]["code"], -32601);
    assert_eq!(out[1]["error"]["code"], -32700);
    assert_eq!(out[1]["id"], Value::Null);
    assert_eq!(out[2]["id"], 2, "the server survives a bad line");
}

#[test]
fn matrix_tool_has_no_across_strata() {
    let t = tools_list()
        .into_iter()
        .find(|t| t["name"] == "personajury_matrix")
        .unwrap();
    let props = t["inputSchema"]["properties"].as_object().unwrap();
    assert!(!props.contains_key("across_strata"));
    assert_eq!(t["inputSchema"]["additionalProperties"], false);
    for k in ["runs", "table", "history", "detail"] {
        assert!(props.contains_key(k), "{k}");
    }
    assert_eq!(props["runs"]["minItems"], 1);
    // sending it anyway is refused as an unknown argument, not passed to the engine
    let ws = tempfile::tempdir().unwrap();
    let (_, out) = session(
        ws.path(),
        &[call(
            1,
            "personajury_matrix",
            json!({"runs":[1],"across_strata":true}),
        )],
    );
    assert_eq!(out[0]["result"]["isError"], true);
    let env = tool_env(&out[0]);
    assert_eq!(env["error"]["code"], "invalid_arguments");
    assert!(env["error"]["message"]
        .as_str()
        .unwrap()
        .contains("across_strata"));
}

#[test]
fn verdict_tool_passes_the_envelope() {
    // A real refusal through the tool: verdict on run 1 in an empty workspace is the engine's own error envelope.
    let ws = tempfile::tempdir().unwrap();
    let (_, out) = session(
        ws.path(),
        &[call(
            1,
            "personajury_verdict",
            json!({"run": 1, "verdict": {"persona": "p", "lines": []}}),
        )],
    );
    let env = tool_env(&out[0]);
    assert_eq!(env["verb"], "verdict");
    assert_eq!(env["ok"], false);
    assert_eq!(out[0]["result"]["isError"], true);
    assert_eq!(env["error"]["code"], "not_found");
    assert_eq!(env["schema"], 1);
    assert_eq!(env["evidence"]["model_calls"], false);
}

#[test]
fn verdict_tool_checks_its_arguments() {
    let ws = tempfile::tempdir().unwrap();
    let (_, out) = session(
        ws.path(),
        &[
            call(1, "personajury_verdict", json!({"run": 1})),
            call(2, "personajury_verdict", json!({"run": "1", "verdict": {}})),
            call(3, "personajury_verdict", json!({"run": 1, "verdict": "text"})),
        ],
    );
    for o in &out {
        assert_eq!(o["result"]["isError"], true);
        assert_eq!(tool_env(o)["error"]["code"], "invalid_arguments");
    }
}

#[test]
fn should_run_tool() {
    let ws = tempfile::tempdir().unwrap();
    let (_, out) = session(
        ws.path(),
        &[
            call(
                1,
                "personajury_should_run",
                json!({"paths": ["src/a.rs", "web/b.tsx"]}),
            ),
            call(2, "personajury_should_run", json!({"bogus": 1})),
            call(3, "personajury_should_run", json!({"paths": "src/a.rs"})),
        ],
    );
    let env = tool_env(&out[0]);
    assert_eq!(env["verb"], "should-run");
    assert_eq!(
        out[0]["result"]["isError"],
        !env["ok"].as_bool().unwrap(),
        "isError mirrors the envelope"
    );
    assert_eq!(tool_env(&out[1])["error"]["code"], "invalid_arguments");
    assert_eq!(tool_env(&out[2])["error"]["code"], "invalid_arguments");
}

#[test]
fn brief_delta_and_matrix_tools_reach_their_verbs() {
    let ws = tempfile::tempdir().unwrap();
    let (_, out) = session(
        ws.path(),
        &[
            call(
                1,
                "personajury_brief",
                json!({"subject":"s","panel":"p","lens":"critical","stakes":{"a":"primary","b":"bystander"},"model":"m","comparators":["c"],"detail":"full","spend":2}),
            ),
            call(
                2,
                "personajury_brief",
                json!({"subject":"s","panel":"p","lens":"critical","stakes":["a=primary","b=secondary"],"model":"m"}),
            ),
            call(3, "personajury_delta", json!({"a": 1, "b": 2})),
            call(
                4,
                "personajury_matrix",
                json!({"runs":[1,2],"table":true,"history":true,"detail":"compact"}),
            ),
        ],
    );
    let verbs: Vec<_> = out
        .iter()
        .map(|o| tool_env(o)["verb"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(verbs, ["brief", "brief", "delta", "matrix"]);
    for o in &out {
        let env = tool_env(o);
        assert_eq!(o["result"]["isError"], !env["ok"].as_bool().unwrap());
        assert_ne!(env["error"]["code"], "invalid_arguments", "{env}");
    }
}

#[test]
fn bad_stakes_detail_and_runs_are_invalid_arguments() {
    let ws = tempfile::tempdir().unwrap();
    let (_, out) = session(
        ws.path(),
        &[
            call(
                1,
                "personajury_brief",
                json!({"subject":"s","panel":"p","lens":7,"model":"m"}),
            ),
            call(
                2,
                "personajury_brief",
                json!({"subject":"s","panel":"p","lens":"l","model":"m","stakes":{"a":"loud"}}),
            ),
            call(
                3,
                "personajury_brief",
                json!({"subject":"s","panel":"p","lens":"l","model":"m","stakes":{"a":"primary"},"detail":"huge"}),
            ),
            call(4, "personajury_matrix", json!({"runs": []})),
            call(5, "personajury_matrix", json!({})),
            call(6, "personajury_delta", json!({"a": 1})),
            call(7, "personajury_nope", json!({})),
            call(8, "personajury_delta", json!([1, 2])),
        ],
    );
    for o in &out {
        assert_eq!(o["result"]["isError"], true, "{o}");
        assert_eq!(tool_env(o)["error"]["code"], "invalid_arguments", "{o}");
    }
}

#[test]
fn apply_check_over_mcp_answers_the_same_envelope() {
    let ws = tempfile::tempdir().unwrap();
    let (ok, out) = session(
        ws.path(),
        &[call(1, "personajury_apply_check", json!({"runs": [1]})), call(2, "personajury_apply_check", json!({}))],
    );
    assert!(ok);
    // no store yet: the verb's own refusal, as an envelope with isError
    let env: Value = serde_json::from_str(out[0]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(env["verb"], "apply-check", "{env}");
    assert_eq!(out[0]["result"]["isError"], true);
    let env: Value = serde_json::from_str(out[1]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "invalid_arguments", "{env}");
}
