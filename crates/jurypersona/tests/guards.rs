//! R16: the engine never fetches the network and never reads a key from the environment. The scanners are functions so
//! that a unit test can show each goes red on a planted offence.
use std::path::{Path, PathBuf};

const NETWORK_CRATES: [&str; 10] = [
    "reqwest",
    "hyper",
    "ureq",
    "curl",
    "isahc",
    "surf",
    "h2",
    "rustls",
    "native-tls",
    "openssl",
];

/// The network crates a Cargo.lock names.
fn network_crates_in_lock(lock: &str) -> Vec<String> {
    NETWORK_CRATES
        .iter()
        .filter(|c| lock.lines().any(|l| l.trim() == format!("name = \"{c}\"")))
        .map(|c| c.to_string())
        .collect()
}

const SECRET_WORDS: [&str; 5] = ["KEY", "TOKEN", "SECRET", "PASSWORD", "API"];

/// Every `env::var(` / `env::var_os(` call in `src` whose first string argument names a secret.
fn secret_env_reads(src: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (n, line) in src.lines().enumerate() {
        for pat in ["env::var(", "env::var_os("] {
            let mut rest = line;
            while let Some(i) = rest.find(pat) {
                rest = &rest[i + pat.len()..];
                let arg = rest.trim_start();
                if let Some(s) = arg.strip_prefix('"') {
                    let name = s.split('"').next().unwrap_or("").to_uppercase();
                    if SECRET_WORDS.iter().any(|w| name.contains(w)) {
                        found.push(format!("line {}: {}", n + 1, line.trim()));
                    }
                }
            }
        }
    }
    found
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn no_network_crate_in_lock() {
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let text = std::fs::read_to_string(&lock).expect("Cargo.lock at the workspace root");
    assert!(
        text.contains("name = \"jurypersona\""),
        "scanned the right lock"
    );
    assert_eq!(network_crates_in_lock(&text), Vec::<String>::new());
}

#[test]
fn network_scanner_catches_a_planted_crate() {
    let planted =
        "[[package]]\nname = \"serde\"\n\n[[package]]\nname = \"reqwest\"\nversion = \"0.12.0\"\n";
    assert_eq!(network_crates_in_lock(planted), vec!["reqwest"]);
    assert!(network_crates_in_lock("[[package]]\nname = \"serde_json\"\n").is_empty());
}

#[test]
fn no_secret_env_read_in_src() {
    let mut files = Vec::new();
    rs_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(files.len() > 10, "walked src/: {} files", files.len());
    let mut bad = Vec::new();
    for f in files {
        for hit in secret_env_reads(&std::fs::read_to_string(&f).unwrap()) {
            bad.push(format!("{}: {hit}", f.display()));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn env_scanner_catches_a_planted_read() {
    let planted = "fn k() -> String {\n    std::env::var(\"OPENAI_API_KEY\").unwrap()\n}\n";
    assert_eq!(secret_env_reads(planted).len(), 1);
    assert_eq!(
        secret_env_reads("let t = env::var_os( \"gh_token\");").len(),
        1
    );
    // harmless reads and mentions in prose are not flagged
    assert!(secret_env_reads("let h = std::env::var(\"HOME\");").is_empty());
    assert!(secret_env_reads("// never call env::var for a key").is_empty());
}

const NETWORK_STD: [&str; 3] = ["std::net", "TcpStream", "UdpSocket"];

/// Every line of `src` that names the standard library's network API (no crate needed for a socket).
fn std_network_uses(src: &str) -> Vec<String> {
    src.lines()
        .enumerate()
        .filter(|(_, l)| NETWORK_STD.iter().any(|w| l.contains(w)))
        .map(|(n, l)| format!("line {}: {}", n + 1, l.trim()))
        .collect()
}

#[test]
fn no_std_network_in_src() {
    let mut files = Vec::new();
    rs_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(files.len() > 10, "walked src/: {} files", files.len());
    let mut bad = Vec::new();
    for f in files {
        for hit in std_network_uses(&std::fs::read_to_string(&f).unwrap()) {
            bad.push(format!("{}: {hit}", f.display()));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn std_network_scanner_catches_a_planted_socket() {
    assert_eq!(
        std_network_uses("use std::net::TcpStream;\nfn f() {}\n").len(),
        1
    );
    assert_eq!(
        std_network_uses("let s = UdpSocket::bind(\"0.0.0.0:0\");").len(),
        1
    );
    assert!(std_network_uses("let x = 1;").is_empty());
}

/// A `bind(` or `listen(` call: the engine is a one-shot CLI and a stdio server, it never accepts a connection.
fn listener_calls(src: &str) -> Vec<String> {
    src.lines()
        .enumerate()
        .filter(|(_, l)| {
            let code = l.split("//").next().unwrap_or("");
            code.contains("bind(") || code.contains("listen(")
        })
        .map(|(n, l)| format!("line {}: {}", n + 1, l.trim()))
        .collect()
}

#[test]
fn no_listener_in_src() {
    let mut files = Vec::new();
    rs_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    assert!(files.len() > 10, "walked src/: {} files", files.len());
    let mut bad = Vec::new();
    for f in files {
        for hit in listener_calls(&std::fs::read_to_string(&f).unwrap()) {
            bad.push(format!("{}: {hit}", f.display()));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn listener_scanner_catches_a_planted_bind() {
    assert_eq!(listener_calls("let l = TcpListener::bind(\"127.0.0.1:0\");").len(), 1);
    assert_eq!(listener_calls("sock.listen(16);").len(), 1);
    // prose after a line comment is not code
    assert!(listener_calls("let x = 1; // never bind( a port").is_empty());
    assert!(listener_calls("let x = 1;").is_empty());
}
