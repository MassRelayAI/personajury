//! Content hashes: sha256, lowercase hex. Same input, same hash, on every platform.
use super::model::{Attachment, SubjectKind};
use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
/// Keys sorted at every depth, no whitespace.
pub fn canonical_json(v: &serde_json::Value) -> String {
    let mut out = String::new();
    write_canonical(v, &mut out);
    out
}
fn write_canonical(v: &serde_json::Value, out: &mut String) {
    use serde_json::Value;
    match v {
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(x, out);
            }
            out.push(']');
        }
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(k).expect("string serialises"));
                out.push(':');
                write_canonical(&m[k], out);
            }
            out.push('}');
        }
        other => out.push_str(&serde_json::to_string(other).expect("scalar serialises")),
    }
}
/// sha256 of `kind\n` followed by `label\thash\n` for each attachment sorted by label.
pub fn subject_hash(kind: SubjectKind, atts: &[Attachment]) -> String {
    let mut sorted: Vec<&Attachment> = atts.iter().collect();
    sorted.sort_by(|a, b| a.label.cmp(&b.label));
    let mut s = format!("{}\n", kind.name());
    for a in sorted {
        s.push_str(&format!("{}\t{}\n", a.label, a.hash));
    }
    sha256_hex(s.as_bytes())
}
/// sha256 of the query, a newline, then the persona ids in the given (spend) order joined by `\n`.
pub fn panel_hash(query: &str, ids: &[String]) -> String {
    sha256_hex(format!("{}\n{}", query, ids.join("\n")).as_bytes())
}
/// First 16 hex of sha256(subject \t persona \t dimension \t about \t text \t citation_hash); `about` is the option the
/// line scored, the empty string for the subject as a whole (I9).
pub fn finding_id(
    subject: &str,
    persona: &str,
    dimension: &str,
    about: Option<&str>,
    text: &str,
    citation_hash: &str,
) -> String {
    let about = about.unwrap_or("");
    sha256_hex(
        format!("{subject}\t{persona}\t{dimension}\t{about}\t{text}\t{citation_hash}").as_bytes(),
    )[..16]
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sha256_of_empty_is_known() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
    #[test]
    fn canonical_json_sorts_keys() {
        let v = json!({"b": 1, "a": {"d": [2, {"z": null, "y": "x"}], "c": true}});
        assert_eq!(
            canonical_json(&v),
            r#"{"a":{"c":true,"d":[2,{"y":"x","z":null}]},"b":1}"#
        );
    }
    fn att(label: &str, hash: &str) -> Attachment {
        Attachment {
            label: label.into(),
            media: "file".into(),
            hash: hash.into(),
            source_path: label.into(),
        }
    }
    #[test]
    fn subject_hash_ignores_attachment_order() {
        let a = [att("a", "h1"), att("b", "h2")];
        let b = [att("b", "h2"), att("a", "h1")];
        assert_eq!(
            subject_hash(SubjectKind::Page, &a),
            subject_hash(SubjectKind::Page, &b)
        );
        assert_eq!(
            subject_hash(SubjectKind::Page, &a),
            sha256_hex(b"page\na\th1\nb\th2\n")
        );
        assert_ne!(
            subject_hash(SubjectKind::Page, &a),
            subject_hash(SubjectKind::Diff, &a)
        );
    }
    #[test]
    fn panel_hash_depends_on_query_and_ids() {
        let ids = vec!["a".to_string(), "b".to_string()];
        assert_eq!(panel_hash("2 x", &ids), panel_hash("2 x", &ids));
        assert_ne!(panel_hash("2 x", &ids), panel_hash("3 x", &ids));
        assert_ne!(panel_hash("2 x", &ids), panel_hash("2 x", &ids[..1]));
    }
    #[test]
    fn finding_id_is_stable() {
        let a = finding_id("s", "p", "d", None, "t", "c");
        assert_eq!(a.len(), 16);
        assert_eq!(a, finding_id("s", "p", "d", None, "t", "c"));
        assert_eq!(a, sha256_hex(b"s\tp\td\t\tt\tc")[..16]);
        assert_ne!(a, finding_id("s", "p", "d", None, "t2", "c"));
        // the option is part of the point (I9)
        let on_a = finding_id("s", "p", "d", Some("a.png"), "t", "c");
        assert_eq!(on_a, sha256_hex(b"s\tp\td\ta.png\tt\tc")[..16]);
        assert_ne!(on_a, a);
        assert_ne!(on_a, finding_id("s", "p", "d", Some("cmp:x"), "t", "c"));
    }
}
