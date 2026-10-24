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

/// `R` or `r` followed by anything but a letter, or by `C`/`c` and a digit: an attempt at a cell (`R1C2`, `R01C1`, `R+1C1`,
/// `r9c9`, `RC1`), refused unless canonical; not an anchor like `Rationale`.
fn cell_shaped(loc: &str) -> bool {
    let Some(r) = loc.strip_prefix('R').or_else(|| loc.strip_prefix('r')) else { return false };
    let mut c = r.chars();
    match c.next() {
        Some('C' | 'c') => c.next().is_some_and(|d| d.is_ascii_digit()),
        Some(x) => !x.is_alphabetic(),
        None => false,
    }
}
/// `R<row>C<col>` in canonical form (1-based, the header row is row 1, no sign, no leading zero) inside the CSV's records.
fn check_cell(c: &str, label: &str, locator: &str, rows: &[usize]) -> Result<(), Refusal> {
    let canon = |d: &str| -> Option<usize> {
        let n = d.parse::<usize>().ok().filter(|_| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))?;
        (d == n.to_string()).then_some(n)
    };
    let (row, col) = locator
        .strip_prefix('R')
        .and_then(|r| r.split_once('C'))
        .and_then(|(r, c)| Some((canon(r)?, canon(c)?)))
        .ok_or_else(|| unresolved(c, "a CSV cell locator reads R<row>C<col> (1-based, digits, no sign, no leading zero)"))?;
    if row < 1 || row > rows.len() {
        return Err(unresolved(c, format!("{label} has {} rows (records, the header is row 1); R{row} is outside", rows.len())));
    }
    if col < 1 || col > rows[row - 1] {
        return Err(unresolved(c, format!("{label} row {row} has {} fields; C{col} is outside", rows[row - 1])));
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
            "file" if view.csv.is_some() && cell_shaped(locator) => {
                check_cell(c, label, locator, view.csv.as_deref().unwrap_or(&[]))?
            }
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
    fn csv_ev() -> RunEvidence {
        // name,price / "Team, annual",12 / "two\nlines",7,extra   -> 3 records of 2, 2, 3 fields
        let sheet = AttachmentView {
            hash: "H6".into(),
            media: "file".into(),
            lines: 4,
            text: true,
            csv: Some(crate::domain::csv::shape("name,price\n\"Team, annual\",12\n\"two\nlines\",7,extra\n")),
            ..Default::default()
        };
        RunEvidence { attachments: BTreeMap::from([("prices.csv".into(), sheet)]), comparators: BTreeMap::new() }
    }
    #[test]
    fn a_csv_cell_resolves_in_range() {
        let e = csv_ev();
        assert_eq!(resolve("prices.csv#R1C1", &e).unwrap(), "H6#R1C1");
        assert_eq!(resolve("prices.csv#R2C2", &e).unwrap(), "H6#R2C2");
        // a quoted comma and an embedded newline do not add fields or rows
        assert_eq!(resolve("prices.csv#R3C3", &e).unwrap(), "H6#R3C3");
        // line locators keep working
        assert_eq!(resolve("prices.csv#L4", &e).unwrap(), "H6#L4");
    }
    #[test]
    fn a_csv_cell_out_of_range_is_unresolved() {
        let e = csv_ev();
        for c in ["prices.csv#R4C1", "prices.csv#R2C3", "prices.csv#R1C9", "prices.csv#R0C1", "prices.csv#R1C0"] {
            assert_eq!(resolve(c, &e).unwrap_err().code(), "citation_unresolved", "{c}");
        }
        let m = resolve("prices.csv#R4C1", &e).unwrap_err().to_string();
        assert!(m.contains("3 rows"), "{m}");
    }
    #[test]
    fn a_csv_cell_must_be_canonical() {
        let e = csv_ev();
        for c in ["prices.csv#R01C1", "prices.csv#R+1C1", "prices.csv#R1C01", "prices.csv#R1", "prices.csv#R1C", "prices.csv#R-1C1", "prices.csv#R1c1"] {
            assert_eq!(resolve(c, &e).unwrap_err().code(), "citation_unresolved", "{c}");
        }
        // an anchor that merely starts with R stays an anchor
        assert_eq!(resolve("prices.csv#Rationale", &e).unwrap(), "H6#Rationale");
    }
    #[test]
    fn a_cell_locator_on_a_file_that_is_not_a_csv_is_left_alone() {
        assert_eq!(resolve("doc#R9C9", &ev()).unwrap(), "H4#R9C9");
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
