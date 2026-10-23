//! Panel queries: "5 companies, 5 builders, 3 personal, 2 others" against persona tags.
use super::model::{Persona, Stake, Term};
use super::rng::SplitMix64;
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
                    "query term {:?} is not \"<positive integer> <tag>[=<stake>]\"",
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
            let (tag_raw, stake) = match parts[1].split_once('=') {
                None => (parts[1], None),
                Some((t, s)) => {
                    let stake = Stake::parse(&s.to_lowercase()).ok_or_else(bad)?;
                    (t, Some(stake))
                }
            };
            if tag_raw.is_empty() {
                return Err(bad());
            }
            let mut tag = tag_raw.to_lowercase();
            if tag == "other" {
                tag = "others".into();
            }
            Ok(Term { count, tag, stake })
        })
        .collect()
}
/// The seats of each term, in query order, personas inside a term by id, each persona once; `others` = none of the
/// query's other tags. With a seed, a term whose pool is bigger than its seats is drawn by a seeded shuffle of the
/// pool (sorted by id, then Fisher-Yates); a pool no bigger than the seats is never shuffled.
pub fn draw_terms(terms: &[Term], personas: &[Persona], seed: Option<[u8; 8]>) -> Vec<Vec<String>> {
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
        let mut pool: Vec<&Persona> = sorted
            .iter()
            .copied()
            .filter(|p| !taken.contains(p.id.as_str()))
            .filter(|p| {
                if t.tag == "others" {
                    !p.tags.iter().any(|x| named.contains(x.as_str()))
                } else {
                    p.tags.contains(&t.tag)
                }
            })
            .collect();
        // one stream per term (seeded from the run's seed and the tag): adding to another term's pool moves nothing here
        if let Some(seed) = seed {
            if t.count > 0 && pool.len() > t.count {
                SplitMix64::from_bytes(super::hash::term_seed(seed, &t.tag)).shuffle(&mut pool);
            }
        }
        let seats: Vec<String> = pool.iter().take(t.count).map(|p| p.id.clone()).collect();
        for p in pool.iter().take(t.count) {
            taken.insert(p.id.as_str());
        }
        out.push(seats);
    }
    out
}
/// The terms as one normalised string (`3 builder=primary, 2 others`): what an ad-hoc query is seeded and hashed by.
pub fn normalise(terms: &[Term]) -> String {
    terms
        .iter()
        .map(|t| match t.stake {
            Some(s) => format!("{} {}={}", t.count, t.tag, s.name()),
            None => format!("{} {}", t.count, t.tag),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
/// `draw_terms` flattened, each seat with the stake its term gives it.
pub fn draw(terms: &[Term], personas: &[Persona], seed: Option<[u8; 8]>) -> Vec<(String, Option<Stake>)> {
    draw_terms(terms, personas, seed)
        .into_iter()
        .zip(terms)
        .flat_map(|(ids, t)| ids.into_iter().map(move |i| (i, t.stake)))
        .collect()
}
/// `draw` with no seed: the id order, the stakes dropped.
pub fn resolve(terms: &[Term], personas: &[Persona]) -> Vec<String> {
    draw(terms, personas, None).into_iter().map(|(i, _)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{Origin, Stake};

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
                    tag: "companies".into(),
                    stake: None
                },
                Term {
                    count: 2,
                    tag: "builders".into(),
                    stake: None
                },
                Term {
                    count: 1,
                    tag: "others".into(),
                    stake: None
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

    #[test]
    fn parse_reads_a_stake_per_term() {
        let t = parse("3 builder=primary, 1 newcomer=Primary, 2 approver=secondary, 1 others=bystander, 2 plain")
            .unwrap();
        let got: Vec<(&str, Option<Stake>)> = t.iter().map(|x| (x.tag.as_str(), x.stake)).collect();
        assert_eq!(
            got,
            vec![
                ("builder", Some(Stake::Primary)),
                ("newcomer", Some(Stake::Primary)),
                ("approver", Some(Stake::Secondary)),
                ("others", Some(Stake::Bystander)),
                ("plain", None),
            ]
        );
        for bad in ["2 builder=", "2 builder=boss", "2 =primary", "2 a=primary=x"] {
            assert_eq!(parse(bad).unwrap_err().code(), "invalid_query", "{bad:?}");
        }
    }
    fn pool(n: usize, tag: &str) -> Vec<Persona> {
        (0..n).map(|i| p(&format!("{tag}-{i:02}"), &[tag])).collect()
    }
    const SEED_A: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    const SEED_B: [u8; 8] = [8, 7, 6, 5, 4, 3, 2, 1];
    #[test]
    fn draw_without_a_seed_is_the_id_order() {
        let ps = pool(6, "others");
        let t = parse("3 others").unwrap();
        let ids: Vec<String> = draw(&t, &ps, None).into_iter().map(|(i, _)| i).collect();
        assert_eq!(ids, vec!["others-00", "others-01", "others-02"]);
        assert_eq!(resolve(&t, &ps), ids);
    }
    #[test]
    fn draw_takes_distinct_seats_from_the_pool_and_is_reproducible() {
        let ps = pool(9, "others");
        let t = parse("4 others=bystander").unwrap();
        let a = draw(&t, &ps, Some(SEED_A));
        assert_eq!(a.len(), 4);
        let mut ids: Vec<&String> = a.iter().map(|(i, _)| i).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 4, "a persona sits once");
        assert!(a.iter().all(|(_, s)| *s == Some(Stake::Bystander)));
        assert_eq!(a, draw(&t, &ps, Some(SEED_A)), "the same seed gives the same draw");
        assert_ne!(a, draw(&t, &ps, Some(SEED_B)), "another seed draws other seats");
    }
    #[test]
    fn draw_is_a_permutation_when_seats_cover_the_pool_size_minus_none() {
        // a draw of every seat but one never repeats and never invents a persona
        let ps = pool(6, "others");
        let t = parse("5 others").unwrap();
        let mut ids: Vec<String> = draw(&t, &ps, Some(SEED_A)).into_iter().map(|(i, _)| i).collect();
        ids.sort();
        let all: Vec<String> = ps.iter().map(|x| x.id.clone()).collect();
        assert!(ids.iter().all(|i| all.contains(i)));
        ids.dedup();
        assert_eq!(ids.len(), 5);
    }
    #[test]
    fn a_pool_equal_to_the_seats_is_not_shuffled() {
        let ps = pool(3, "builder");
        let t = parse("3 builder").unwrap();
        let ids: Vec<String> = draw(&t, &ps, Some(SEED_A)).into_iter().map(|(i, _)| i).collect();
        assert_eq!(ids, vec!["builder-00", "builder-01", "builder-02"]);
        // a pool smaller than the seats gives what there is
        let t = parse("5 builder").unwrap();
        assert_eq!(draw(&t, &ps, Some(SEED_A)).len(), 3);
    }
    #[test]
    fn a_persona_added_to_the_pool_changes_the_draw() {
        let mut ps = pool(8, "others");
        let t = parse("3 others").unwrap();
        let before = draw(&t, &ps, Some(SEED_A));
        ps.push(p("others-99", &["others"]));
        let after = draw(&t, &ps, Some(SEED_A));
        assert_ne!(before, after);
    }
    #[test]
    fn a_term_has_its_own_stream_adding_to_one_term_leaves_the_others() {
        let mut ps = pool(1, "builder");
        ps.extend(pool(8, "others"));
        let t = parse("1 builder=primary, 3 others=bystander").unwrap();
        let before = draw_terms(&t, &ps, Some(SEED_A));
        // the builder pool grows from equal-to-seats (not shuffled) to bigger (shuffled)
        ps.extend((0..4).map(|i| p(&format!("builder-x{i}"), &["builder"])));
        let after = draw_terms(&t, &ps, Some(SEED_A));
        assert_eq!(before[1], after[1], "the others keep their seats");
    }
    #[test]
    fn terms_are_drawn_in_order_and_each_persona_once() {
        let mut ps = pool(5, "builder");
        ps.extend(pool(5, "others"));
        let t = parse("2 builder=primary, 3 others=bystander").unwrap();
        let per = draw_terms(&t, &ps, Some(SEED_A));
        assert_eq!(per.len(), 2);
        assert_eq!((per[0].len(), per[1].len()), (2, 3));
        assert!(per[0].iter().all(|i| i.starts_with("builder")));
        assert!(per[1].iter().all(|i| i.starts_with("others")));
    }
}
