//! `should-run`: the trigger classifier over a diff (report §10). Rules are data in jurygraph.toml.
use super::model::{Classification, Quorum, RuleHit, TriggerConfig, HUMAN_CLASSES};
use super::refuse::Refusal;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::BTreeSet;

fn quorum_name(q: Quorum) -> &'static str {
    match q {
        Quorum::None => "none",
        Quorum::Low => "low",
        Quorum::High => "high",
        Quorum::Human => "human",
    }
}
fn build_set(paths: &[String]) -> Result<GlobSet, String> {
    let mut b = GlobSetBuilder::new();
    for p in paths {
        b.add(Glob::new(p).map_err(|e| format!("bad glob {p:?}: {e}"))?);
    }
    b.build().map_err(|e| e.to_string())
}

/// A class name as the human-class lookup reads it (I7): trimmed, lower-case, spaces and underscores as `-`, and the
/// US `license` as `licence`. "Money", " public_claim " and "license" are human classes.
pub fn normalise_class(class: &str) -> String {
    let c: String = class
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch == ' ' || ch == '_' { '-' } else { ch })
        .collect();
    if c == "license" {
        "licence".into()
    } else {
        c
    }
}

/// Parse and refuse a human class under a persona quorum, whatever its spelling.
pub fn parse_config(s: &str) -> Result<TriggerConfig, Refusal> {
    let cfg: TriggerConfig =
        toml::from_str(s).map_err(|e| Refusal::InvalidConfig(e.to_string()))?;
    for r in &cfg.rules {
        if HUMAN_CLASSES.contains(&normalise_class(&r.class).as_str()) && r.quorum != Quorum::Human
        {
            return Err(Refusal::HumanClassUnderPersonaQuorum {
                class: r.class.clone(),
                quorum: quorum_name(r.quorum).into(),
            });
        }
        if r.paths.is_empty() {
            return Err(Refusal::InvalidConfig(format!(
                "rule {}: paths is empty",
                r.class
            )));
        }
        build_set(&r.paths)
            .map_err(|e| Refusal::InvalidConfig(format!("rule {}: {e}", r.class)))?;
        if matches!(r.quorum, Quorum::Low | Quorum::High) {
            if r.panel.as_deref().is_none_or(|p| p.trim().is_empty()) {
                return Err(Refusal::RuleWithoutPanel {
                    class: r.class.clone(),
                });
            }
            if r.lenses.is_empty() {
                return Err(Refusal::InvalidConfig(format!(
                    "rule {}: lenses is empty; a {} quorum runs at least one lens",
                    r.class,
                    quorum_name(r.quorum)
                )));
            }
        }
    }
    Ok(cfg)
}
fn bad_diff(line: &str, why: &str) -> Refusal {
    Refusal::InvalidDiff(format!("{why}: {line}"))
}

/// Read one C-quoted string (git's `core.quotePath` form) at the start of `s`: the unquoted text and what follows the
/// closing quote. Octal escapes are bytes; the result must be UTF-8.
fn unquote_at(s: &str) -> Option<(String, &str)> {
    let body = s.strip_prefix('"')?;
    let b = body.as_bytes();
    let mut out: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' => return Some((String::from_utf8(out).ok()?, &body[i + 1..])),
            b'\\' => {
                let c = *b.get(i + 1)?;
                match c {
                    b'0'..=b'7' => {
                        let oct = body.get(i + 1..i + 4)?;
                        out.push(u8::from_str_radix(oct, 8).ok()?);
                        i += 4;
                        continue;
                    }
                    b'n' => out.push(b'\n'),
                    b't' => out.push(b'\t'),
                    b'r' => out.push(b'\r'),
                    b'a' => out.push(0x07),
                    b'b' => out.push(0x08),
                    b'f' => out.push(0x0c),
                    b'v' => out.push(0x0b),
                    b'\\' | b'"' => out.push(c),
                    _ => return None,
                }
                i += 2;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    None
}

/// A path after `+++ `, `--- `, `rename from ` or `rename to `: quoted or plain (a plain one ends at a tab).
fn path_field(rest: &str, line: &str) -> Result<String, Refusal> {
    if rest.starts_with('"') {
        match unquote_at(rest) {
            Some((p, tail)) if tail.is_empty() || tail.starts_with('\t') => Ok(p),
            _ => Err(bad_diff(line, "a quoted path that does not parse")),
        }
    } else {
        Ok(rest.split('\t').next().unwrap_or(rest).to_string())
    }
}

/// The b-side path of a `diff --git <a> <b>` header, either side quoted or plain.
fn header_path(rest: &str, line: &str) -> Result<String, Refusal> {
    let refuse = || bad_diff(line, "a diff --git header with no a/ and b/ path");
    let b_side = if rest.starts_with('"') {
        let (a, tail) = unquote_at(rest).ok_or_else(refuse)?;
        if !a.starts_with("a/") {
            return Err(refuse());
        }
        let tail = tail.strip_prefix(' ').ok_or_else(refuse)?;
        if tail.starts_with('"') {
            match unquote_at(tail) {
                Some((b, "")) => b,
                _ => return Err(refuse()),
            }
        } else {
            tail.to_string()
        }
    } else if rest.ends_with('"') {
        let i = rest.rfind(" \"b/").ok_or_else(refuse)?;
        match unquote_at(&rest[i + 1..]) {
            Some((b, "")) if rest.starts_with("a/") => b,
            _ => return Err(refuse()),
        }
    } else {
        if !rest.starts_with("a/") {
            return Err(refuse());
        }
        rest.rsplit_once(" b/")
            .map(|(_, y)| format!("b/{y}"))
            .ok_or_else(refuse)?
    };
    match b_side.strip_prefix("b/") {
        Some(p) if !p.is_empty() => Ok(p.to_string()),
        _ => Err(refuse()),
    }
}

/// The paths a unified diff touches: the b-side of every `diff --git` header and `+++`, the a-side of `---`, both
/// sides of a rename; git's C-quoted paths are unquoted (I8). Sorted, unique, `/dev/null` skipped. A `diff --git`
/// header that yields no path refuses the diff (`invalid_diff`): nothing is dropped silently.
pub fn paths_from_diff(diff: &str) -> Result<Vec<String>, Refusal> {
    let mut out = BTreeSet::new();
    for line in diff.lines() {
        let found = if let Some(rest) = line.strip_prefix("diff --git ") {
            Some(header_path(rest, line)?)
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            path_field(rest, line)?
                .strip_prefix("b/")
                .map(str::to_string)
        } else if let Some(rest) = line.strip_prefix("--- ") {
            path_field(rest, line)?
                .strip_prefix("a/")
                .map(str::to_string)
        } else if let Some(rest) = line
            .strip_prefix("rename from ")
            .or_else(|| line.strip_prefix("rename to "))
        {
            Some(path_field(rest, line)?)
        } else {
            None
        };
        if let Some(p) = found {
            if !p.is_empty() && p != "/dev/null" {
                out.insert(p);
            }
        }
    }
    Ok(out.into_iter().collect())
}
pub fn classify(cfg: &TriggerConfig, paths: &[String]) -> Classification {
    let all: BTreeSet<&String> = paths.iter().collect();
    let mut hits = Vec::new();
    let mut matched: BTreeSet<&String> = BTreeSet::new();
    for r in &cfg.rules {
        // a set that fails to build matches nothing; parse_config already refuses such a rule
        let Ok(set) = build_set(&r.paths) else {
            continue;
        };
        let ps: Vec<String> = all
            .iter()
            .filter(|p| set.is_match(p.as_str()))
            .map(|p| (*p).clone())
            .collect();
        if ps.is_empty() {
            continue;
        }
        matched.extend(all.iter().filter(|p| set.is_match(p.as_str())));
        hits.push(RuleHit {
            class: r.class.clone(),
            quorum: r.quorum,
            paths: ps,
        });
    }
    let quorum = hits.iter().map(|h| h.quorum).max().unwrap_or(Quorum::None);
    let unmatched = all
        .iter()
        .filter(|p| !matched.contains(**p))
        .map(|p| (*p).clone())
        .collect();
    Classification {
        quorum,
        hits,
        unmatched,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{Quorum, HUMAN_CLASSES};

    const CFG: &str = r#"
[[rule]]
class = "docs"
paths = ["docs/**", "*.md"]
quorum = "low"
panel = "public"
lenses = ["recall"]

[[rule]]
class = "ui"
paths = ["src/ui/**"]
quorum = "high"
panel = "public"
lenses = ["critical"]

[[rule]]
class = "money"
paths = ["src/billing/**"]
quorum = "human"
"#;
    fn paths(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn money_under_low_refused() {
        let s = "[[rule]]\nclass = \"money\"\npaths = [\"a\"]\nquorum = \"low\"\n";
        let e = parse_config(s).unwrap_err();
        assert_eq!(e.code(), "human_class_under_persona_quorum");
        assert!(e.to_string().contains("low"));
    }
    #[test]
    fn every_human_class_is_refused_under_each_persona_quorum() {
        for class in HUMAN_CLASSES {
            for q in ["none", "low", "high"] {
                let s =
                    format!("[[rule]]\nclass = \"{class}\"\npaths = [\"a\"]\nquorum = \"{q}\"\n");
                assert_eq!(
                    parse_config(&s).unwrap_err().code(),
                    "human_class_under_persona_quorum",
                    "{class} {q}"
                );
            }
        }
    }
    #[test]
    fn human_quorum_on_any_class_is_allowed() {
        for class in HUMAN_CLASSES.iter().chain(["docs", "ui"].iter()) {
            let s = format!("[[rule]]\nclass = \"{class}\"\npaths = [\"a\"]\nquorum = \"human\"\n");
            assert!(parse_config(&s).is_ok(), "{class}");
        }
    }
    #[test]
    fn invalid_config_cases() {
        for s in [
            "[[rule]]\nclass = \"x\"\npaths = [\"a\"]\nquorum = \"mega\"\n",
            "[[rule]]\nclass = \"x\"\npaths = []\nquorum = \"low\"\n",
            "[[rule]]\nclass = \"x\"\npaths = [\"[a\"]\nquorum = \"low\"\n",
            "not toml = = =",
        ] {
            assert_eq!(parse_config(s).unwrap_err().code(), "invalid_config", "{s}");
        }
    }
    #[test]
    fn a_low_or_high_rule_needs_a_panel() {
        for q in ["low", "high"] {
            let s = format!(
                "[[rule]]\nclass = \"copy\"\npaths = [\"a\"]\nquorum = \"{q}\"\nlenses = [\"recall\"]\n"
            );
            let e = parse_config(&s).unwrap_err();
            assert_eq!(e.code(), "rule_without_panel", "{q}");
            assert!(e.to_string().contains("copy"));
        }
        for q in ["none", "human"] {
            let s = format!("[[rule]]\nclass = \"copy\"\npaths = [\"a\"]\nquorum = \"{q}\"\n");
            assert!(parse_config(&s).is_ok(), "{q} needs no panel");
        }
    }
    #[test]
    fn a_low_or_high_rule_needs_lenses() {
        for q in ["low", "high"] {
            let s = format!(
                "[[rule]]\nclass = \"copy\"\npaths = [\"a\"]\nquorum = \"{q}\"\npanel = \"p\"\n"
            );
            assert_eq!(parse_config(&s).unwrap_err().code(), "invalid_config", "{q}");
        }
    }
    #[test]
    fn rule_detail_default_compact() {
        use crate::domain::model::Detail;
        let cfg = parse_config(CFG).unwrap();
        assert!(cfg.rules.iter().all(|r| r.detail == Detail::Compact));
        let s = "[[rule]]\nclass = \"ui\"\npaths = [\"a\"]\nquorum = \"none\"\ndetail = \"full\"\n";
        assert_eq!(parse_config(s).unwrap().rules[0].detail, Detail::Full);
        let s = "[[rule]]\nclass = \"ui\"\npaths = [\"a\"]\nquorum = \"high\"\ndetail = \"huge\"\n";
        assert_eq!(parse_config(s).unwrap_err().code(), "invalid_config");
    }
    #[test]
    fn paths_from_diff_reads_renames() {
        let d = "diff --git a/old/name.rs b/new/name.rs\nsimilarity index 90%\nrename from old/name.rs\nrename to new/name.rs\n\
diff --git a/x.md b/x.md\n--- a/x.md\n+++ b/x.md\n@@ -1 +1 @@\n-a\n+b\n\
diff --git a/gone.txt b/gone.txt\ndeleted file mode 100644\n--- a/gone.txt\n+++ /dev/null\n\
diff --git a/new.txt b/new.txt\nnew file mode 100644\n--- /dev/null\n+++ b/new.txt\n";
        assert_eq!(
            paths_from_diff(d).unwrap(),
            vec!["gone.txt", "new.txt", "new/name.rs", "old/name.rs", "x.md"]
        );
    }
    #[test]
    fn quoted_paths_are_unquoted() {
        let d = "diff --git \"a/brand/logo-\\303\\251.svg\" \"b/brand/logo-\\303\\251.svg\"\n\
--- \"a/brand/logo-\\303\\251.svg\"\n+++ \"b/brand/logo-\\303\\251.svg\"\n\
diff --git a/x y.md \"b/x\\ty.md\"\nrename from \"say \\\"hi\\\".md\"\nrename to b.md\n";
        assert_eq!(
            paths_from_diff(d).unwrap(),
            vec!["b.md", "brand/logo-é.svg", "say \"hi\".md", "x\ty.md"]
        );
    }
    #[test]
    fn unreadable_header_refuses_the_diff() {
        for d in [
            "diff --git what is this\n",
            "diff --git \"a/unterminated\n",
            "diff --git \"a/x\\9\" \"b/x\"\n",
            "diff --git a/x \"b/\\377\"\n",
        ] {
            assert_eq!(
                paths_from_diff(d).unwrap_err().code(),
                "invalid_diff",
                "{d}"
            );
        }
    }
    #[test]
    fn class_spellings_are_normalised() {
        for (raw, want) in [
            ("license", "licence"),
            (" Money ", "money"),
            ("public_claim", "public-claim"),
            ("Public Claim", "public-claim"),
            ("LICENCE", "licence"),
            ("docs", "docs"),
        ] {
            assert_eq!(normalise_class(raw), want);
        }
    }
    #[test]
    fn classify_takes_highest_quorum() {
        let cfg = parse_config(CFG).unwrap();
        let c = classify(&cfg, &paths(&["src/ui/b.rs", "README.md", "src/ui/a.rs"]));
        assert_eq!(c.quorum, Quorum::High);
        assert_eq!(c.hits.len(), 2);
        assert_eq!(c.hits[0].class, "docs");
        assert_eq!(c.hits[1].paths, vec!["src/ui/a.rs", "src/ui/b.rs"]);
        assert!(c.unmatched.is_empty());
        let c = classify(&cfg, &paths(&["src/billing/x.rs", "src/ui/a.rs"]));
        assert_eq!(c.quorum, Quorum::Human);
    }
    #[test]
    fn unmatched_paths_are_none() {
        let cfg = parse_config(CFG).unwrap();
        let c = classify(&cfg, &paths(&["z.rs", "a.rs"]));
        assert_eq!(c.quorum, Quorum::None);
        assert!(c.hits.is_empty());
        assert_eq!(c.unmatched, vec!["a.rs", "z.rs"]);
        let c = classify(&cfg, &paths(&["z.rs", "docs/a.txt"]));
        assert_eq!(c.quorum, Quorum::Low);
        assert_eq!(c.unmatched, vec!["z.rs"]);
    }
}
