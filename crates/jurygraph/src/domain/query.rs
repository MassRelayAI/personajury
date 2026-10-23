//! Panel queries: "5 companies, 5 builders, 3 personal, 2 others" against persona tags.
use super::model::{Persona, Term};
use super::refuse::Refusal;
use std::collections::BTreeSet;

pub fn parse(q: &str) -> Result<Vec<Term>, Refusal> {
    let q = q.trim();
    if q.is_empty() {
        return Err(Refusal::InvalidQuery(
            "the panel query is empty; write e.g. \"5 builders, 2 others\"".into(),
        ));
    }
    q.split(',')
        .map(|raw| {
            let bad = || {
                Refusal::InvalidQuery(format!(
                    "query term {:?} is not \"<positive integer> <tag>\"",
                    raw.trim()
                ))
            };
            let parts: Vec<&str> = raw.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(bad());
            }
            let count: usize = parts[0].parse().map_err(|_| bad())?;
            if count == 0 {
                return Err(bad());
            }
            let mut tag = parts[1].to_lowercase();
            if tag == "other" {
                tag = "others".into();
            }
            Ok(Term { count, tag })
        })
        .collect()
}
/// Terms in query order, personas inside a term by id, each persona once; `others` = none of the query's other tags.
pub fn resolve(terms: &[Term], personas: &[Persona]) -> Vec<String> {
    let named: BTreeSet<&str> = terms
        .iter()
        .map(|t| t.tag.as_str())
        .filter(|t| *t != "others")
        .collect();
    let mut sorted: Vec<&Persona> = personas.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    let mut taken: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    for t in terms {
        let mut n = 0;
        for p in &sorted {
            if n == t.count {
                break;
            }
            if taken.contains(p.id.as_str()) {
                continue;
            }
            let matches = if t.tag == "others" {
                !p.tags.iter().any(|x| named.contains(x.as_str()))
            } else {
                p.tags.contains(&t.tag)
            };
            if matches {
                taken.insert(p.id.as_str());
                out.push(p.id.clone());
                n += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::Origin;

    fn p(id: &str, tags: &[&str]) -> Persona {
        Persona {
            id: id.into(),
            version: 1,
            name: id.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            stake_rules: String::new(),
            profile_json: "{}".into(),
            origin: Origin::Hand,
            hash: String::new(),
        }
    }
    #[test]
    fn parse_query_reads_counts_and_tags() {
        let t = parse(" 5 Companies, 2 builders ,1 other").unwrap();
        assert_eq!(
            t,
            vec![
                Term {
                    count: 5,
                    tag: "companies".into()
                },
                Term {
                    count: 2,
                    tag: "builders".into()
                },
                Term {
                    count: 1,
                    tag: "others".into()
                }
            ]
        );
    }
    #[test]
    fn parse_refuses_bad_terms() {
        for q in [
            "",
            "  ",
            "x builders",
            "0 builders",
            "3",
            "3 a b",
            "2 a,,1 b",
            "-1 a",
        ] {
            assert_eq!(parse(q).unwrap_err().code(), "invalid_query", "{q:?}");
        }
        assert!(parse("2 a, nope").unwrap_err().to_string().contains("nope"));
    }
    #[test]
    fn resolve_takes_each_persona_once() {
        let ps = [
            p("c", &["builders", "companies"]),
            p("a", &["builders"]),
            p("b", &["companies"]),
        ];
        let t = parse("2 companies, 2 builders").unwrap();
        // companies by id: b, c ; builders by id: a (c already taken)
        assert_eq!(resolve(&t, &ps), vec!["b", "c", "a"]);
    }
    #[test]
    fn others_means_none_of_the_named_tags() {
        let ps = [
            p("a", &["builders"]),
            p("b", &["x"]),
            p("c", &[]),
            p("d", &["companies"]),
        ];
        let t = parse("1 builders, 1 companies, 5 others").unwrap();
        assert_eq!(resolve(&t, &ps), vec!["a", "d", "b", "c"]);
    }
}
