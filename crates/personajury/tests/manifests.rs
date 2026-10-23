//! D16: the surfaces' manifests agree. One name, one version (the workspace's), one command (`personajury mcp`) across the
//! plugin manifests, `mcp.json`, `server.json` and the `.mcpb` manifest; the `.mcpb` arguments are run, so their order is
//! pinned by the binary itself; the bundle script is run on a stand-in binary.
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}
fn json_of(rel: &str) -> Value {
    serde_json::from_str(&read(rel)).unwrap_or_else(|e| panic!("{rel} is not JSON: {e}"))
}
/// `[workspace.package] version` of the root Cargo.toml.
fn cargo_version() -> String {
    let t: toml::Value = toml::from_str(&read("Cargo.toml")).unwrap();
    t["workspace"]["package"]["version"].as_str().unwrap().to_string()
}
const PLATFORMS: [&str; 4] = ["linux-x64", "darwin-arm64", "darwin-x64", "win32-x64"];

#[test]
fn every_manifest_carries_the_cargo_version() {
    let v = cargo_version();
    let seen = [
        (".claude-plugin/plugin.json", json_of(".claude-plugin/plugin.json")["version"].clone()),
        (".claude-plugin/marketplace.json", json_of(".claude-plugin/marketplace.json")["plugins"][0]["version"].clone()),
        (".codex-plugin/plugin.json", json_of(".codex-plugin/plugin.json")["version"].clone()),
        (".cursor-plugin/plugin.json", json_of(".cursor-plugin/plugin.json")["version"].clone()),
        ("gemini-extension.json", json_of("gemini-extension.json")["version"].clone()),
        ("server.json", json_of("server.json")["version"].clone()),
        ("packaging/mcpb/manifest.json", json_of("packaging/mcpb/manifest.json")["version"].clone()),
    ];
    for (file, got) in seen {
        assert_eq!(got, json!(v), "{file} has version {got}, Cargo.toml has {v}");
    }
}

#[test]
fn every_manifest_names_personajury() {
    for f in [
        ".claude-plugin/plugin.json",
        ".codex-plugin/plugin.json",
        ".cursor-plugin/plugin.json",
        "gemini-extension.json",
        "packaging/mcpb/manifest.json",
    ] {
        assert_eq!(json_of(f)["name"], "personajury", "{f}");
    }
    assert_eq!(json_of(".claude-plugin/marketplace.json")["plugins"][0]["name"], "personajury");
    let name = json_of("server.json")["name"].as_str().unwrap().to_string();
    assert!(name.ends_with("/personajury"), "{name}");
}

#[test]
fn one_command_personajury_mcp_across_every_surface() {
    let want = json!({"command": "personajury", "args": ["mcp"]});
    assert_eq!(json_of("mcp.json")["mcpServers"]["personajury"], want);
    assert_eq!(json_of(".claude-plugin/plugin.json")["mcpServers"]["personajury"], want);
    assert_eq!(json_of("gemini-extension.json")["mcpServers"]["personajury"], want);
    // Codex and Cursor point at the one file; the file is `mcp.json`, never `.mcp.json` (Claude Code would load that one)
    assert_eq!(json_of(".codex-plugin/plugin.json")["mcpServers"], "./mcp.json");
    assert_eq!(json_of(".cursor-plugin/plugin.json")["mcpServers"], "mcp.json");
    assert!(!repo().join(".mcp.json").exists());
}

#[test]
fn the_registry_entry_lists_one_mcpb_package_per_bundle() {
    let s = json_of("server.json");
    let v = cargo_version();
    let pk = s["packages"].as_array().expect("server.json lists packages");
    assert_eq!(pk.len(), PLATFORMS.len());
    for (p, plat) in pk.iter().zip(PLATFORMS) {
        assert_eq!(p["registryType"], "mcpb");
        assert_eq!(p["transport"]["type"], "stdio");
        let id = p["identifier"].as_str().unwrap();
        assert_eq!(
            id,
            format!("https://github.com/massrelay-ai/personajury/releases/download/v{v}/personajury-{plat}.mcpb")
        );
        assert!(id.contains("mcp"), "the registry needs `mcp` in the URL");
        let sha = p["fileSha256"].as_str().unwrap();
        assert!(sha.len() == 64 && sha.chars().all(|c| c.is_ascii_hexdigit()), "{sha}");
    }
}

#[test]
fn the_mcpb_manifest_has_the_shape_and_its_arguments_run() {
    let m = json_of("packaging/mcpb/manifest.json");
    assert_eq!(m["manifest_version"], "0.4");
    assert_eq!(m["server"]["type"], "binary");
    assert_eq!(m["server"]["entry_point"], "bin/personajury");
    assert_eq!(m["server"]["mcp_config"]["command"], "${__dirname}/bin/personajury");
    assert_eq!(m["user_config"]["root"]["type"], "directory");
    assert_eq!(m["user_config"]["root"]["required"], true);
    let args: Vec<String> = m["server"]["mcp_config"]["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect();
    assert_eq!(args, ["--root", "${user_config.root}", "mcp"]);
    // run them: clap's global --root precedes the verb, and the server answers an initialize
    let ws = tempfile::tempdir().unwrap();
    let args: Vec<String> = args.iter().map(|a| a.replace("${user_config.root}", ws.path().to_str().unwrap())).collect();
    let mut ch = Command::new(assert_cmd::cargo::cargo_bin("personajury"))
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = ch.stdin.take().unwrap();
    writeln!(stdin, "{}", json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}})).unwrap();
    drop(stdin);
    let first = BufReader::new(ch.stdout.take().unwrap()).lines().next().unwrap().unwrap();
    let r: Value = serde_json::from_str(&first).unwrap();
    assert_eq!(r["result"]["serverInfo"]["name"], "personajury", "{r}");
    ch.wait().unwrap();
}

#[test]
fn the_bundle_script_zips_the_manifest_for_the_platform_and_the_binary() {
    let work = tempfile::tempdir().unwrap();
    let fake = work.path().join("personajury-fake");
    std::fs::write(&fake, b"not a real binary").unwrap();
    for (plat, os, exe) in [("linux-x64", "linux", "personajury"), ("win32-x64", "win32", "personajury.exe")] {
        let out = Command::new("bash")
            .arg(repo().join("scripts/build-mcpb.sh"))
            .args([plat, fake.to_str().unwrap(), work.path().join("dist").to_str().unwrap()])
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let sha = String::from_utf8(out.stdout).unwrap().trim().to_string();
        assert_eq!(sha.len(), 64, "the script prints the sha256: {sha:?}");
        let bundle = work.path().join(format!("dist/personajury-{plat}.mcpb"));
        assert!(bundle.is_file());
        let listed = Command::new("unzip").args(["-Z1", bundle.to_str().unwrap()]).output().unwrap();
        let names = String::from_utf8(listed.stdout).unwrap();
        assert!(names.lines().any(|l| l == "manifest.json") && names.lines().any(|l| l == format!("bin/{exe}")), "{names}");
        let m = Command::new("unzip").args(["-p", bundle.to_str().unwrap(), "manifest.json"]).output().unwrap();
        let m: Value = serde_json::from_slice(&m.stdout).unwrap();
        assert_eq!(m["compatibility"]["platforms"], json!([os]), "{m}");
        assert_eq!(m["server"]["mcp_config"]["command"], format!("${{__dirname}}/bin/{exe}"), "{m}");
        let sums = std::fs::read_to_string(work.path().join(format!("dist/personajury-{plat}.mcpb.sha256"))).unwrap();
        assert!(sums.starts_with(&sha), "{sums}");
    }
    let bad = Command::new("bash").arg(repo().join("scripts/build-mcpb.sh")).args(["plan9", fake.to_str().unwrap()]).output().unwrap();
    assert!(!bad.status.success());
}
