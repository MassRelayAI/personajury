//! `trait observed` (D35): what a persona did in its accepted initial verdicts, as counts, and what that bears on the
//! traits it declares. Pure. A bearing, not a measurement: nothing here is a score, a share or a mean, and a contradicted
//! trait is a question for the author, not a verdict on the persona.
use super::model::{Evidence, ScoreRow, Stake};
use super::traits::{TraitDef, TraitValue};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const SIGNALS: [&str; 5] = ["severity", "blockers", "evidence", "pick", "bottom"];

/// One row of `traits/observe-map.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub signal: String,
    #[serde(default = "star")]
    pub lens: String,
    #[serde(default = "star")]
    pub dimension: String,
    pub direction: String,
    #[serde(rename = "trait")]
    pub trait_id: String,
    pub pole: String,
}
fn star() -> String {
    "*".into()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleFile {
    #[serde(default, rename = "rule")]
    rules: Vec<Rule>,
}

/// The directions a signal can bear, the first being the one a persona's counts must exceed the second to be `consistent`.
fn directions(signal: &str) -> &'static [&'static str] {
    match signal {
        "severity" => &["below", "above"],
        "blockers" | "bottom" => &["more", "fewer"],
        "evidence" => &["recalled_over_seen", "seen_over_recalled"],
        "pick" => &["against_majority", "with_majority"],
        _ => &[],
    }
}

/// Parse a mapping table (the shipped one, or a project's rows). A row names a known signal and one of its directions; a
/// lens or dimension belongs to `severity` only.
pub fn parse_rules(text: &str) -> Result<Vec<Rule>, String> {
    let f: RuleFile = toml::from_str(text).map_err(|e| e.to_string())?;
    for r in &f.rules {
        if !SIGNALS.contains(&r.signal.as_str()) {
            return Err(format!("rule {}: signal {:?} is not one of {}", r.id, r.signal, SIGNALS.join(", ")));
        }
        if !directions(&r.signal).contains(&r.direction.as_str()) {
            return Err(format!("rule {}: direction {:?} does not fit signal {}; the words are {}", r.id, r.direction, r.signal, directions(&r.signal).join(", ")));
        }
        if r.signal != "severity" && (r.lens != "*" || r.dimension != "*") {
            return Err(format!("rule {}: lens and dimension belong to the severity signal only", r.id));
        }
        if r.id.trim().is_empty() || r.trait_id.trim().is_empty() || r.pole.trim().is_empty() {
            return Err(format!("rule {}: id, trait and pole are required", r.id));
        }
    }
    Ok(f.rules)
}

/// One run's accepted initial rows of every juror, with what the signals need beside them.
#[derive(Clone)]
pub struct RunData {
    pub run: i64,
    pub stakes: BTreeMap<String, Stake>,
    pub rows: Vec<ScoreRow>,
    /// the lowest score of the run's lens
    pub scale_min: i64,
    /// (persona, choice) of picks among options; a decision's picks per question are not among them
    pub picks: Vec<(String, String)>,
    pub subject: String,
}

/// The signals of one persona over some runs. Every figure is a count.
#[derive(Debug, Default)]
pub struct Signals {
    pub runs: Vec<i64>,
    /// (lens, dimension) to (below, at, above)
    pub severity: BTreeMap<(String, String), (usize, usize, usize)>,
    /// per run: (run, the persona's blocker count, the stratum's median, word)
    pub blockers: Vec<(i64, usize, usize, &'static str)>,
    pub seen: usize,
    pub told: usize,
    pub recalled: usize,
    /// (subject, comparator, attachment, with, against)
    pub pick: (usize, usize, usize, usize, usize),
    pub bottom: (usize, usize, usize),
}

fn lower_middle(v: &mut [usize]) -> usize {
    v.sort_unstable();
    v[(v.len() - 1) / 2]
}
fn lower_middle_i(v: &mut [i64]) -> i64 {
    v.sort_unstable();
    v[(v.len() - 1) / 2]
}
fn word(mine: usize, median: usize) -> &'static str {
    match mine.cmp(&median) {
        std::cmp::Ordering::Greater => "more",
        std::cmp::Ordering::Equal => "same",
        std::cmp::Ordering::Less => "fewer",
    }
}

/// Count `persona`'s behaviour across `runs`. A comparison against the stratum needs two other jurors in it (a median of
/// one or two is the persona itself or nearly), so a thinner stratum adds nothing to severity, blockers, pick or bottom;
/// the evidence counts are the persona's own and need no peer.
pub fn observe(persona: &str, runs: &[RunData]) -> Signals {
    let mut s = Signals::default();
    for r in runs {
        let Some(&stake) = r.stakes.get(persona) else { continue };
        let mine: Vec<&ScoreRow> = r.rows.iter().filter(|x| x.persona == persona).collect();
        if mine.is_empty() {
            continue;
        }
        s.runs.push(r.run);
        for row in &mine {
            match row.evidence {
                Evidence::Seen => s.seen += 1,
                Evidence::Told => s.told += 1,
                Evidence::Recalled => s.recalled += 1,
            }
        }
        let peers: Vec<&String> = {
            let mut v: Vec<&String> = r
                .rows
                .iter()
                .filter(|x| r.stakes.get(&x.persona) == Some(&stake))
                .map(|x| &x.persona)
                .collect();
            v.sort();
            v.dedup();
            v
        };
        if peers.len() < 3 {
            continue;
        }
        // severity: the persona's score in each cell against the stratum's lower middle of the cell
        let mut cells: BTreeMap<(&str, &str, Option<&str>), BTreeMap<&str, i64>> = BTreeMap::new();
        for x in r.rows.iter().filter(|x| r.stakes.get(&x.persona) == Some(&stake)) {
            cells.entry((&x.lens, &x.dimension, x.about.as_deref())).or_default().insert(&x.persona, x.score);
        }
        for ((lens, dim, _), by) in &cells {
            let Some(&mine) = by.get(persona) else { continue };
            if by.len() < 3 {
                continue;
            }
            let mut vals: Vec<i64> = by.values().copied().collect();
            let median = lower_middle_i(&mut vals);
            let e = s.severity.entry((lens.to_string(), dim.to_string())).or_default();
            match mine.cmp(&median) {
                std::cmp::Ordering::Less => e.0 += 1,
                std::cmp::Ordering::Equal => e.1 += 1,
                std::cmp::Ordering::Greater => e.2 += 1,
            }
        }
        // blockers and bottom scores per juror of the stratum
        let count = |who: &str, f: &dyn Fn(&ScoreRow) -> usize| r.rows.iter().filter(|x| x.persona == who).map(f).sum::<usize>();
        let blockers: Vec<usize> = peers.iter().map(|p| count(p, &|x| x.blockers.len())).collect();
        let (b_mine, b_med) = (count(persona, &|x| x.blockers.len()), lower_middle(&mut blockers.clone()));
        let w = word(b_mine, b_med);
        s.blockers.push((r.run, b_mine, b_med, w));
        let bottoms: Vec<usize> = peers.iter().map(|p| count(p, &|x| usize::from(x.score == r.scale_min))).collect();
        let (t_mine, t_med) = (count(persona, &|x| usize::from(x.score == r.scale_min)), lower_middle(&mut bottoms.clone()));
        match word(t_mine, t_med) {
            "more" => s.bottom.0 += 1,
            "same" => s.bottom.1 += 1,
            _ => s.bottom.2 += 1,
        }
        // picks: what it chose, and whether the stratum's most common choice was the same
        let in_stratum: Vec<&(String, String)> = r.picks.iter().filter(|(p, _)| r.stakes.get(p) == Some(&stake)).collect();
        if let Some((_, choice)) = in_stratum.iter().find(|(p, _)| p == persona) {
            if choice.starts_with("cmp:") {
                s.pick.1 += 1;
            } else if *choice == r.subject {
                s.pick.0 += 1;
            } else {
                s.pick.2 += 1;
            }
            if in_stratum.len() >= 3 {
                let mut tally: BTreeMap<&str, usize> = BTreeMap::new();
                for (_, c) in &in_stratum {
                    *tally.entry(c).or_default() += 1;
                }
                let top = tally.values().copied().max().unwrap_or(0);
                let leaders: Vec<&&str> = tally.iter().filter(|(_, n)| **n == top).map(|(c, _)| c).collect();
                if leaders.len() == 1 {
                    if **leaders[0] == **choice {
                        s.pick.3 += 1;
                    } else {
                        s.pick.4 += 1;
                    }
                }
            }
        }
    }
    s
}

impl Signals {
    /// `punished`: the (lens, dimension) with the most `below`, ties by id order; none when nothing was below.
    pub fn punished(&self) -> Option<(&str, &str)> {
        let mut best: Option<(&(String, String), usize)> = None;
        for (k, v) in &self.severity {
            if v.0 > 0 && best.is_none_or(|(_, n)| v.0 > n) {
                best = Some((k, v.0));
            }
        }
        best.map(|(k, _)| (k.0.as_str(), k.1.as_str()))
    }
    pub fn blocker_words(&self) -> (usize, usize, usize) {
        let n = |w: &str| self.blockers.iter().filter(|b| b.3 == w).count();
        (n("more"), n("same"), n("fewer"))
    }
    pub fn to_json(&self) -> Value {
        let (more, same, fewer) = self.blocker_words();
        json!({
            "severity": self.severity.iter().map(|((l, d), (b, a, u))| json!({"lens": l, "dimension": d, "below": b, "at": a, "above": u})).collect::<Vec<_>>(),
            "blockers": {
                "more": more, "same": same, "fewer": fewer,
                "runs": self.blockers.iter().map(|(run, n, med, w)| json!({"run": run, "count": n, "stratum_median": med, "word": w})).collect::<Vec<_>>(),
            },
            "punished": self.punished().map(|(l, d)| json!({"lens": l, "dimension": d})),
            "evidence": {"seen": self.seen, "told": self.told, "recalled": self.recalled},
            "pick": {"subject": self.pick.0, "comparator": self.pick.1, "attachment": self.pick.2,
                     "with_majority": self.pick.3, "against_majority": self.pick.4},
            "bottom": {"more": self.bottom.0, "same": self.bottom.1, "fewer": self.bottom.2},
        })
    }
}

/// The word of one rule: its direction's count against the opposite one's.
fn rule_word(consistent_n: usize, opposite_n: usize) -> &'static str {
    match consistent_n.cmp(&opposite_n) {
        std::cmp::Ordering::Greater => "consistent",
        std::cmp::Ordering::Less => "contradicted",
        std::cmp::Ordering::Equal => "no_signal",
    }
}

/// (the rule's own word, the counts it was read from)
fn read_rule(rule: &Rule, s: &Signals) -> (&'static str, Value) {
    match rule.signal.as_str() {
        "severity" => {
            let (mut b, mut a, mut u) = (0, 0, 0);
            for ((l, d), (x, y, z)) in &s.severity {
                if (rule.lens == "*" || rule.lens == *l) && (rule.dimension == "*" || rule.dimension == *d) {
                    b += x;
                    a += y;
                    u += z;
                }
            }
            let w = if rule.direction == "below" { rule_word(b, u) } else { rule_word(u, b) };
            (w, json!({"below": b, "at": a, "above": u}))
        }
        "blockers" | "bottom" => {
            let (more, same, fewer) = if rule.signal == "blockers" { s.blocker_words() } else { s.bottom };
            let w = if rule.direction == "more" { rule_word(more, fewer) } else { rule_word(fewer, more) };
            (w, json!({"more": more, "same": same, "fewer": fewer}))
        }
        "evidence" => {
            let w = if rule.direction == "recalled_over_seen" { rule_word(s.recalled, s.seen) } else { rule_word(s.seen, s.recalled) };
            (w, json!({"seen": s.seen, "told": s.told, "recalled": s.recalled}))
        }
        _ => {
            let (with, against) = (s.pick.3, s.pick.4);
            let w = if rule.direction == "against_majority" { rule_word(against, with) } else { rule_word(with, against) };
            (w, json!({"with_majority": with, "against_majority": against}))
        }
    }
}

/// The rules that apply to a declared value: for a band 4 or 5 the `high` pole, 1 or 2 the `low` pole, 3 none; for a
/// category the rules whose pole is the declared option (or one of them); for words none.
pub fn applicable<'a>(def: Option<&TraitDef>, v: &TraitValue, rules: &'a [Rule]) -> Vec<&'a Rule> {
    let Some(def) = def else { return vec![] };
    let poles: Vec<String> = match (def.shape.as_str(), v) {
        ("band", TraitValue::Int(n)) => match n {
            4 | 5 => vec!["high".into()],
            1 | 2 => vec!["low".into()],
            _ => vec![],
        },
        ("category", TraitValue::Str(s)) => vec![s.clone()],
        ("category", TraitValue::List(l)) => l.clone(),
        _ => vec![],
    };
    rules.iter().filter(|r| r.trait_id == def.id && poles.contains(&r.pole)).collect()
}

/// The verdict per declared trait: the rules that apply, each with its counts and word, and the trait's own word.
pub fn verdicts(declared: &BTreeMap<String, TraitValue>, defs: &BTreeMap<String, TraitDef>, rules: &[Rule], s: &Signals) -> Vec<Value> {
    declared
        .iter()
        .map(|(id, v)| {
            let read: Vec<(&Rule, &'static str, Value)> =
                applicable(defs.get(id), v, rules).into_iter().map(|r| { let (w, c) = read_rule(r, s); (r, w, c) }).collect();
            let any = |w: &str| read.iter().any(|(_, x, _)| *x == w);
            let overall = match (any("consistent"), any("contradicted")) {
                (true, true) => "mixed",
                (true, false) => "consistent",
                (false, true) => "contradicted",
                _ => "no_signal",
            };
            json!({
                "trait": id, "declared": v, "verdict": overall,
                "rules": read.iter().map(|(r, w, c)| json!({"id": r.id, "verdict": w, "counts": c})).collect::<Vec<_>>(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_table_parses_and_has_eighteen_rows() {
        let rules = parse_rules(include_str!("../../../../traits/observe-map.toml")).unwrap();
        assert_eq!(rules.len(), 18);
    }
    #[test]
    fn a_direction_must_fit_the_signal() {
        let bad = "[[rule]]\nid=\"x\"\nsignal=\"blockers\"\ndirection=\"below\"\ntrait=\"a.b\"\npole=\"high\"\n";
        assert!(parse_rules(bad).unwrap_err().contains("does not fit"));
        let lens_on_blockers = "[[rule]]\nid=\"x\"\nsignal=\"blockers\"\nlens=\"compare\"\ndirection=\"more\"\ntrait=\"a.b\"\npole=\"high\"\n";
        assert!(parse_rules(lens_on_blockers).is_err());
    }

    fn defs() -> BTreeMap<String, TraitDef> {
        ["trust.marketing_skepticism", "choice.decision_style", "cognition.need_for_cognition", "social.need_for_uniqueness", "social.influence_susceptibility"]
            .iter()
            .map(|id| (id.to_string(), TraitDef::from_toml(super::super::catalog::shipped_text(id).unwrap()).unwrap()))
            .collect()
    }
    fn rules() -> Vec<Rule> {
        parse_rules(super::super::catalog::OBSERVE_MAP).unwrap()
    }
    fn word(declared: (&str, TraitValue), s: &Signals) -> String {
        let m: BTreeMap<String, TraitValue> = [(declared.0.to_string(), declared.1)].into();
        verdicts(&m, &defs(), &rules(), s)[0]["verdict"].as_str().unwrap().to_string()
    }
    fn signals() -> Signals {
        Signals::default()
    }

    #[test]
    fn a_rule_that_agrees_and_a_rule_that_disagrees_make_the_trait_mixed() {
        let mut s = signals();
        s.severity.insert(("compare".into(), "trust".into()), (0, 0, 2)); // above: contradicts trust-severity-skeptic
        s.blockers = vec![(1, 4, 0, "more"), (2, 4, 0, "more")]; // more: agrees with blockers-skeptic
        assert_eq!(word(("trust.marketing_skepticism", TraitValue::Int(5)), &s), "mixed");
        s.blockers.clear();
        assert_eq!(word(("trust.marketing_skepticism", TraitValue::Int(5)), &s), "contradicted");
        s.severity.insert(("compare".into(), "trust".into()), (2, 0, 0));
        assert_eq!(word(("trust.marketing_skepticism", TraitValue::Int(4)), &s), "consistent");
        // the low pole has no rule for this trait, and a 3 predicts nothing
        assert_eq!(word(("trust.marketing_skepticism", TraitValue::Int(2)), &s), "no_signal");
        assert_eq!(word(("trust.marketing_skepticism", TraitValue::Int(3)), &s), "no_signal");
        // a tie is no signal
        s.severity.insert(("compare".into(), "trust".into()), (1, 0, 1));
        assert_eq!(word(("trust.marketing_skepticism", TraitValue::Int(5)), &s), "no_signal");
    }
    #[test]
    fn evidence_bears_on_need_for_cognition_at_either_pole_and_on_a_category_option() {
        let mut s = signals();
        s.recalled = 3;
        s.seen = 1;
        assert_eq!(word(("cognition.need_for_cognition", TraitValue::Int(1)), &s), "consistent");
        assert_eq!(word(("cognition.need_for_cognition", TraitValue::Int(5)), &s), "contradicted");
        s.recalled = 0;
        s.seen = 4;
        assert_eq!(word(("choice.decision_style", TraitValue::Str("rational".into())), &s), "consistent");
        assert_eq!(word(("choice.decision_style", TraitValue::Str("intuitive".into())), &s), "no_signal", "no rule for that option");
        s.seen = 0;
        assert_eq!(word(("choice.decision_style", TraitValue::Str("rational".into())), &s), "no_signal", "no lines, no signal");
    }
    #[test]
    fn picks_bear_on_uniqueness_and_on_susceptibility() {
        let mut s = signals();
        s.pick = (0, 0, 3, 0, 3); // against the majority three times
        assert_eq!(word(("social.need_for_uniqueness", TraitValue::Int(5)), &s), "consistent");
        assert_eq!(word(("social.influence_susceptibility", TraitValue::Int(5)), &s), "contradicted");
        s.pick = (0, 0, 3, 2, 1);
        assert_eq!(word(("social.need_for_uniqueness", TraitValue::Int(4)), &s), "contradicted");
        assert_eq!(word(("social.influence_susceptibility", TraitValue::Int(4)), &s), "consistent");
    }
    #[test]
    fn punished_is_the_cell_with_most_below_and_the_first_in_id_order_on_a_tie() {
        let mut s = signals();
        assert_eq!(s.punished(), None);
        s.severity.insert(("critical".into(), "works".into()), (1, 0, 0));
        s.severity.insert(("compare".into(), "price".into()), (2, 0, 0));
        s.severity.insert(("compare".into(), "clarity".into()), (2, 1, 0));
        assert_eq!(s.punished(), Some(("compare", "clarity")));
    }
}
