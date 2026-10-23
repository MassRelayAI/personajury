//! Citations: `<label>[#<locator>]` on the subject, `cmp:<name>/<label>[#<locator>]` on a comparator.
use super::hash::sha256_hex;
use super::model::{AttachmentView, RunEvidence};
use super::refuse::Refusal;

fn unresolved(c: &str, why: impl Into<String>) -> Refusal {
    Refusal::CitationUnresolved {
        citation: c.to_string(),
        why: why.into(),
    }
}

/// `L` followed by anything but a letter: an attempt at a line (`L3`, `L+1`, `L01`, `L 1`), not an anchor like `Latest`.
fn line_shaped(loc: &str) -> bool {
    loc.strip_prefix('L')
        .and_then(|r| r.chars().next())
        .is_some_and(|c| !c.is_alphabetic())
}
/// `L<n>` in canonical form (no sign, no leading zero, so one line has one citation hash) inside `1..=lines`.
fn check_line(c: &str, label: &str, locator: &str, lines: usize, what: &str) -> Result<(), Refusal> {
    let n = locator
        .strip_prefix('L')
        .filter(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|d| d.parse::<usize>().ok())
        .filter(|n| locator == format!("L{n}"))
        .ok_or_else(|| unresolved(c, format!("a {what} locator reads L<n> (digits, no sign, no leading zero)")))?;
    if n < 1 || n > lines {
        return Err(unresolved(
            c,
            format!("{label} has {lines} lines; L{n} is outside"),
        ));
    }
    Ok(())
}

/// Resolve to `<attachment hash>#<locator>` (locator may be empty), or refuse with `citation_unresolved`.
pub fn resolve(c: &str, ev: &RunEvidence) -> Result<String, Refusal> {
    let (view, label, locator): (&AttachmentView, &str, &str) =
        if let Some(rest) = c.strip_prefix("cmp:") {
            let (name, target) = rest.split_once('/').ok_or_else(|| {
                unresolved(
                    c,
                    "a comparator citation reads cmp:<name>/<label>[#<locator>]",
                )
            })?;
            let (label, loc) = target.split_once('#').unwrap_or((target, ""));
            let atts = ev
                .comparators
                .get(name)
                .ok_or_else(|| unresolved(c, format!("the run holds no comparator {name}")))?;
            let view = atts.get(label).ok_or_else(|| {
                unresolved(c, format!("comparator {name} has no attachment {label}"))
            })?;
            (view, label, loc)
        } else {
            let (label, loc) = c.split_once('#').unwrap_or((c, ""));
            let view = ev
                .attachments
                .get(label)
                .ok_or_else(|| unresolved(c, format!("the run holds no attachment {label}")))?;
            (view, label, loc)
        };
    if !locator.is_empty() {
        match view.media.as_str() {
            "aria-json" => {
                if !view.aria_ids.contains(locator) {
                    return Err(unresolved(c, format!("{label} has no ARIA node {locator}")));
                }
            }
            // a diff takes only line locators; a text file takes anchors as v0.1 did, and checks the `L<n>` ones
            "diff" => check_line(c, label, locator, view.lines, "diff")?,
            "file" if view.text && line_shaped(locator) => {
                check_line(c, label, locator, view.lines, "text file")?
            }
            _ => {}
        }
    }
    Ok(format!("{}#{}", view.hash, locator))
}
/// sha256 of the resolved citations sorted and joined by `\n`.
pub fn citation_hash(resolved: &[String]) -> String {
    let mut v: Vec<&str> = resolved.iter().map(String::as_str).collect();
    v.sort();
    sha256_hex(v.join("\n").as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::AttachmentView;
    use std::collections::{BTreeMap, BTreeSet};

    fn ev() -> RunEvidence {
        let aria = AttachmentView {
            hash: "H1".into(),
            media: "aria-json".into(),
            aria_ids: BTreeSet::from(["n1".to_string(), "n2".to_string()]),
            lines: 0,
            ..Default::default()
        };
        let diff = AttachmentView {
            hash: "H2".into(),
            media: "diff".into(),
            aria_ids: BTreeSet::new(),
            lines: 10,
            ..Default::default()
        };
        let img = AttachmentView {
            hash: "H3".into(),
            media: "image".into(),
            aria_ids: BTreeSet::new(),
            lines: 0,
            ..Default::default()
        };
        let cmp = AttachmentView {
            hash: "H9".into(),
            media: "aria-json".into(),
            aria_ids: BTreeSet::from(["c1".to_string()]),
            lines: 0,
            ..Default::default()
        };
        let doc = AttachmentView {
            hash: "H4".into(),
            media: "file".into(),
            lines: 5,
            text: true,
            ..Default::default()
        };
        let blob = AttachmentView {
            hash: "H5".into(),
            media: "file".into(),
            ..Default::default()
        };
        RunEvidence {
            attachments: BTreeMap::from([
                ("doc".into(), doc),
                ("blob".into(), blob),
                ("page".into(), aria),
                ("patch".into(), diff),
                ("shot".into(), img),
            ]),
            comparators: BTreeMap::from([(
                "rival".to_string(),
                BTreeMap::from([("page".to_string(), cmp)]),
            )]),
        }
    }
    #[test]
    fn aria_node_resolves() {
        assert_eq!(resolve("page#n2", &ev()).unwrap(), "H1#n2");
        assert_eq!(resolve("page", &ev()).unwrap(), "H1#");
    }
    #[test]
    fn unknown_aria_node_unresolved() {
        let e = resolve("page#nope", &ev()).unwrap_err();
        assert_eq!(e.code(), "citation_unresolved");
    }
    #[test]
    fn unknown_label_unresolved() {
        assert_eq!(
            resolve("ghost#n1", &ev()).unwrap_err().code(),
            "citation_unresolved"
        );
    }
    #[test]
    fn diff_line_resolves_and_past_end_unresolved() {
        assert_eq!(resolve("patch#L10", &ev()).unwrap(), "H2#L10");
        assert_eq!(resolve("patch#L1", &ev()).unwrap(), "H2#L1");
    }
    #[test]
    fn diff_line_past_end_unresolved() {
        for c in ["patch#L11", "patch#L0", "patch#10", "patch#Lx"] {
            assert_eq!(
                resolve(c, &ev()).unwrap_err().code(),
                "citation_unresolved",
                "{c}"
            );
        }
    }
    #[test]
    fn text_file_line_resolves_and_past_end_unresolved() {
        assert_eq!(resolve("doc#L1", &ev()).unwrap(), "H4#L1");
        assert_eq!(resolve("doc#L5", &ev()).unwrap(), "H4#L5");
        assert_eq!(resolve("doc", &ev()).unwrap(), "H4#");
        for c in ["doc#L6", "doc#L0"] {
            assert_eq!(
                resolve(c, &ev()).unwrap_err().code(),
                "citation_unresolved",
                "{c}"
            );
        }
        let e = resolve("doc#L6", &ev()).unwrap_err();
        assert!(e.to_string().contains("5 lines"), "{e}");
    }
    #[test]
    fn only_line_locators_of_a_text_file_are_checked_and_they_are_canonical() {
        for c in ["doc#intro", "doc#Latest", "doc#install"] {
            assert_eq!(resolve(c, &ev()).unwrap(), format!("H4#{}", &c[4..]), "{c}");
        }
        for c in ["doc#L+1", "doc#L01", "doc#L 1", "doc#L-1", "doc#L1x", "patch#L+1", "patch#L01"] {
            assert_eq!(resolve(c, &ev()).unwrap_err().code(), "citation_unresolved", "{c}");
        }
    }
    #[test]
    fn a_file_that_is_not_text_keeps_its_locator_unchecked() {
        assert_eq!(resolve("blob#L99", &ev()).unwrap(), "H5#L99");
    }
    #[test]
    fn image_locator_is_unchecked() {
        assert_eq!(resolve("shot#top left", &ev()).unwrap(), "H3#top left");
    }
    #[test]
    fn comparator_citation_resolves() {
        assert_eq!(resolve("cmp:rival/page#c1", &ev()).unwrap(), "H9#c1");
        assert_eq!(
            resolve("cmp:rival/page#zz", &ev()).unwrap_err().code(),
            "citation_unresolved"
        );
        assert_eq!(
            resolve("cmp:other/page", &ev()).unwrap_err().code(),
            "citation_unresolved"
        );
        assert_eq!(
            resolve("cmp:rival", &ev()).unwrap_err().code(),
            "citation_unresolved"
        );
    }
    #[test]
    fn citation_hash_is_order_free_and_empty_is_sha_of_empty() {
        let a = citation_hash(&["b#1".to_string(), "a#2".to_string()]);
        let b = citation_hash(&["a#2".to_string(), "b#1".to_string()]);
        assert_eq!(a, b);
        assert_eq!(
            citation_hash(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
