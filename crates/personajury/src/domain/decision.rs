//! Decision juries (D37): the questions file, and the classes that keep a question with a person. Pure.
//! A decision subject is a `questions.md` (`## Q<n> <question>`, then `- option` lines) plus the point-of-view files it
//! is judged from. The classes are keyword lists, fixed and documented: a question that falls in one is never
//! decided by a persona quorum, whatever the jury picked.

/// The attachment label a decision subject's questions live under.
pub const QUESTIONS_LABEL: &str = "questions.md";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// `Q1`, `Q2`, ...
    pub id: String,
    /// the class the author declared (`[class: <class>]`, C3 of the v0.6-C review): a jury class or a human one
    pub class: String,
    pub text: String,
    pub options: Vec<String>,
}

/// The classes a jury may decide on its own (the stem table still reads their text).
pub const JURY_CLASSES: [&str; 4] = ["taste", "product", "copy", "ops"];
/// The classes only a person decides: the human classes of `personajury.toml` and the three a decision adds.
pub const QUESTION_HUMAN_CLASSES: [&str; 10] =
    ["money", "permission", "deploy", "secret", "licence", "public-claim", "brand", "legal", "spending", "irreversible"];
/// The classes a split between strata is never left with a jury on (C4).
pub const SENSITIVE_CLASSES: [&str; 5] = ["money", "spending", "trust", "legal", "public-claim"];

/// Phrases that tell a juror which option the author prefers (`brief --neutral`). Lower-case; matched against the lower-cased text.
const LEANING: [&str; 10] = [
    "(recommended)", "[recommended]", "recommended:", "we recommend", "we prefer", "i prefer", "our preference", "preferred option", "(preferred)", "(default)",
];

/// The lines of a questions file that carry a leaned-on option, as `Q2 option "..." (we prefer)`. Question text and options are read;
/// an empty list means the ballot is neutral. Order: question order, text before options.
pub fn leaning_hits(questions: &[Question]) -> Vec<String> {
    let mut hits = Vec::new();
    for q in questions {
        let mut check = |what: &str, line: &str| {
            let low = line.to_lowercase();
            if let Some(m) = LEANING.iter().find(|m| low.contains(**m)) {
                hits.push(format!("{} {what} \"{}\" ({})", q.id, line.trim(), m.trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']' || c == ':')));
            }
        };
        check("question", &q.text);
        for o in &q.options {
            check("option", o);
        }
    }
    hits
}

/// Why a questions file is refused: the class is missing, or any other break of the grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionsError {
    ClassMissing(String),
    Invalid(String),
}

/// Whether `line` mentions a question id (`Q` then a digit) anywhere: such a line is a question or an error, never prose.
fn mentions_q(line: &str) -> bool {
    let b = line.as_bytes();
    b.windows(2).any(|w| (w[0] == b'Q' || w[0] == b'q') && w[1].is_ascii_digit())
}

/// Parse a questions file (C2, strict). The grammar, and nothing else:
/// - a question is exactly `## Q<n> [class: <class>] <text>`, `n` canonical (1, 2, 3 ... in order, no leading zero);
/// - an option is `- <option>` and belongs to the question above it;
/// - blank lines are allowed anywhere.
///
/// Every other non-blank line is refused with its line number, so nothing is ever silently dropped.
pub fn parse_questions(md: &str) -> Result<Vec<Question>, QuestionsError> {
    let inv = |n: usize, why: String| QuestionsError::Invalid(format!("line {n}: {why}"));
    let mut out: Vec<(usize, Question)> = Vec::new();
    for (i, line) in md.lines().enumerate() {
        let n = i + 1;
        let l = line.trim_end();
        if l.trim().is_empty() {
            continue;
        }
        if let Some(rest) = l.strip_prefix("## ") {
            let (id, after) = rest.split_once(' ').unwrap_or((rest, ""));
            let digits = id.strip_prefix('Q').filter(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()));
            let Some(digits) = digits else {
                return Err(inv(n, format!("\"{l}\" is not \"## Q<n> [class: <class>] <question>\"")));
            };
            let want = out.len() + 1;
            if digits != want.to_string() {
                return Err(match out.iter().find(|(_, q)| q.id == id) {
                    Some((at, _)) => inv(n, format!("{id} is listed twice (first on line {at})")),
                    None => inv(n, format!("{id} is not Q{want}: questions are numbered 1, 2, 3 ... with no leading zero")),
                });
            }
            let after = after.trim_start();
            let Some(tag) = after.strip_prefix("[class:") else {
                return Err(QuestionsError::ClassMissing(format!(
                    "line {n}: {id} declares no class; write \"## {id} [class: <class>] <question>\" with one of {}, {}",
                    JURY_CLASSES.join(", "),
                    QUESTION_HUMAN_CLASSES.join(", ")
                )));
            };
            let Some((class, text)) = tag.split_once(']') else {
                return Err(inv(n, format!("{id}: the class tag has no closing \"]\"")));
            };
            let class = class.trim().to_lowercase();
            let class = if class == "license" { "licence".to_string() } else { class };
            if !JURY_CLASSES.contains(&class.as_str()) && !QUESTION_HUMAN_CLASSES.contains(&class.as_str()) {
                return Err(inv(n, format!("{id}: the class \"{class}\" is not one of {}, {}", JURY_CLASSES.join(", "), QUESTION_HUMAN_CLASSES.join(", "))));
            }
            if text.trim().is_empty() {
                return Err(inv(n, format!("{id} has no question text")));
            }
            out.push((n, Question { id: id.to_string(), class, text: text.trim().to_string(), options: vec![] }));
        } else if let Some(o) = l.strip_prefix("- ") {
            let Some((_, q)) = out.last_mut() else {
                return Err(inv(n, format!("the option \"{}\" comes before the first question", o.trim())));
            };
            let o = o.trim();
            if o.is_empty() {
                return Err(inv(n, format!("{} has an empty option", q.id)));
            }
            if q.options.iter().any(|x| x == o) {
                return Err(inv(n, format!("{} lists the option \"{o}\" twice", q.id)));
            }
            q.options.push(o.to_string());
        } else if l.trim_start().starts_with('#') && mentions_q(l) {
            return Err(inv(n, format!("\"{l}\": a question is a \"## Q<n> [class: <class>] <question>\" heading, at level two")));
        } else {
            return Err(inv(n, format!("\"{l}\" is neither a \"## Q<n> [class: <class>] <question>\" heading nor a \"- option\" line")));
        }
    }
    if out.is_empty() {
        return Err(QuestionsError::Invalid("no question: write \"## Q1 [class: <class>] <question>\" and one \"- option\" line per option".into()));
    }
    if let Some((n, q)) = out.iter().find(|(_, q)| q.options.len() < 2) {
        return Err(inv(*n, format!("{} has {} option(s); a question needs at least two", q.id, q.options.len())));
    }
    Ok(out.into_iter().map(|(_, q)| q).collect())
}

/// The stem table (C3, D37). A word of the text matches a stem when it starts with it, so plurals and other endings
/// match ("prices", "charges", "migrations"); a word that only contains it does not ("relationship" never meets "ship").
/// The table leans wide on purpose: a false hit costs a person a minute, a miss lets a jury decide what is the owner's.
/// `email` counts only beside "all", "every..." or "customer..."; `notify all` is "notif..." beside "all" or "every...";
/// another stem of two words (`turn off`, `shut down`) is two consecutive words, each starting with its part.
pub const STEMS: [(&str, &[&str]); 5] = [
    ("money", &["pric", "charg", "fee", "bill", "refund", "pay", "spend", "budget", "cost", "discount", "invoice", "dollar", "purchas", "salar", "payroll", "revenue", "hire", "hiring", "vendor"]),
    ("legal", &["legal", "contract", "licen", "terms", "privacy", "gdpr", "consent", "liabil", "complian", "trademark", "patent"]),
    ("irreversible", &["delet", "drop", "purge", "migrat", "irreversib", "sunset", "deprecat", "turn off", "shut down"]),
    ("outward", &["deploy", "release", "ship", "publish", "announce", "press", "public", "email", "notify all"]),
    ("secret-permission", &["secret", "key", "token", "password", "permission", "access", "role", "admin", "credential"]),
];
/// What a split between strata is a split on, beyond the stem table: trust (C4). Read the same way, by word start.
pub const TRUST_STEMS: [&str; 6] = ["trust", "secur", "safe", "confiden", "audit", "consent"];

fn words(text: &str) -> Vec<String> {
    text.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_string).collect()
}
fn stem_hits(ws: &[String], stem: &str) -> bool {
    let any_start = |p: &str| ws.iter().any(|w| w.starts_with(p));
    let to_all = ws.iter().any(|w| w == "all" || w.starts_with("every"));
    match stem {
        "email" => any_start("email") && (to_all || any_start("customer")),
        "notify all" => any_start("notif") && to_all,
        _ => match stem.split_once(' ') {
            Some((a, b)) => ws.windows(2).any(|w| w[0].starts_with(a) && w[1].starts_with(b)),
            None => any_start(stem),
        },
    }
}

/// Every (group, stem) of the table that the text hits, in table order; a currency sign counts as money, stem `$`.
pub fn stem_matches(text: &str) -> Vec<(&'static str, &'static str)> {
    let ws = words(text);
    let mut out = Vec::new();
    for (group, stems) in STEMS {
        for stem in stems {
            if stem_hits(&ws, stem) {
                out.push((group, *stem));
            }
        }
    }
    if text.contains(['$', '\u{20ac}', '\u{a3}']) {
        out.insert(0, ("money", "$"));
    }
    out
}
fn question_text(q: &Question) -> String {
    format!("{} {}", q.text, q.options.join(" "))
}
/// The stems a question's text and options hit.
pub fn question_stems(q: &Question) -> Vec<(&'static str, &'static str)> {
    stem_matches(&question_text(q))
}
/// Whether the declared class is one only a person decides.
pub fn is_human_class(class: &str) -> bool {
    QUESTION_HUMAN_CLASSES.contains(&class)
}
/// The sensitive classes a question is in, declared or matched, in the fixed order of `SENSITIVE_CLASSES`: a split
/// between strata on any of them stays with a person (C4).
pub fn sensitive_classes(q: &Question) -> Vec<&'static str> {
    let groups: Vec<&str> = question_stems(q).into_iter().map(|(g, _)| g).collect();
    let ws = words(&question_text(q));
    SENSITIVE_CLASSES
        .into_iter()
        .filter(|c| {
            q.class == *c
                || match *c {
                    "money" | "spending" => groups.contains(&"money"),
                    "legal" => groups.contains(&"legal"),
                    "public-claim" => groups.contains(&"outward"),
                    "trust" => TRUST_STEMS.iter().any(|t| stem_hits(&ws, t)) || groups.contains(&"secret-permission"),
                    _ => false,
                }
        })
        .collect()
}

#[cfg(test)]
mod leaning_tests {
    use super::*;
    fn q(id: &str, text: &str, options: &[&str]) -> Question {
        Question { id: id.into(), class: "product".into(), text: text.into(), options: options.iter().map(|o| o.to_string()).collect() }
    }
    #[test]
    fn a_marked_option_is_a_hit_and_a_plain_ballot_is_not() {
        let plain = [q("Q1", "Which order?", &["Install first", "Account first"])];
        assert!(leaning_hits(&plain).is_empty());
        let marked = [q("Q1", "Which order?", &["Install first (Recommended)", "Account first"]), q("Q2", "Which price? We prefer the lower one", &["9", "12"])];
        let hits = leaning_hits(&marked);
        assert_eq!(hits.len(), 2, "{hits:?}");
        assert!(hits[0].starts_with("Q1 option") && hits[0].contains("recommended"), "{hits:?}");
        assert!(hits[1].starts_with("Q2 question") && hits[1].contains("we prefer"), "{hits:?}");
    }
    #[test]
    fn the_word_recommend_alone_is_not_a_hit() {
        let ok = [q("Q1", "Would you recommend this to a friend?", &["yes", "no"])];
        assert!(leaning_hits(&ok).is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn q(text: &str, options: &[&str]) -> Question {
        Question { id: "Q1".into(), class: "product".into(), text: text.into(), options: options.iter().map(|s| s.to_string()).collect() }
    }
    fn invalid(md: &str) -> String {
        match parse_questions(md).unwrap_err() {
            QuestionsError::Invalid(w) => w,
            other => panic!("{other:?}"),
        }
    }
    #[test]
    fn parses_numbered_questions_with_a_class_and_options() {
        let qs = parse_questions("## Q1 [class: taste] One?\n- a\n- b\n\n## Q2 [class: Money] Two?\n- c\n- d\n- e\n").unwrap();
        assert_eq!(qs.len(), 2);
        assert_eq!((qs[1].id.as_str(), qs[1].class.as_str(), qs[1].options.len()), ("Q2", "money", 3));
        assert_eq!(qs[1].text, "Two?");
    }
    #[test]
    fn refuses_what_the_jury_could_not_answer() {
        assert!(invalid("nothing").contains("line 1"));
        assert!(invalid("## Q1 [class: taste] x\n- a\n").contains("at least two"));
        assert!(invalid("## Q1 [class: taste] x\n- a\n- a\n").contains("twice"));
        assert!(invalid("## First?\n- a\n- b\n").contains("not \"## Q<n>"));
        assert!(invalid("## Q1 [class: taste x\n- a\n- b\n").contains("closing"));
        assert!(matches!(parse_questions("## Q1 x\n- a\n- b\n"), Err(QuestionsError::ClassMissing(_))));
    }
    /// Every stem of the table, each through a plural or another ending of the word.
    #[test]
    fn every_stem_matches_its_word_and_its_plural() {
        let cases: &[(&str, &str, &str)] = &[
            ("money", "pric", "prices"), ("money", "pric", "pricing"), ("money", "charg", "charges"), ("money", "fee", "fees"),
            ("money", "bill", "bills"), ("money", "bill", "billing"), ("money", "refund", "refunds"), ("money", "pay", "payments"),
            ("money", "spend", "spends"), ("money", "budget", "budgets"), ("money", "cost", "costs"), ("money", "discount", "discounts"),
            ("money", "invoice", "invoices"), ("money", "dollar", "dollars"), ("money", "purchas", "purchases"), ("money", "salar", "salaries"),
            ("money", "payroll", "payrolls"), ("money", "revenue", "revenues"), ("money", "hire", "hires"), ("money", "hiring", "hiring"),
            ("money", "vendor", "vendors"),
            ("legal", "legal", "legal"), ("legal", "legal", "legalities"), ("legal", "contract", "contracts"), ("legal", "licen", "licences"),
            ("legal", "licen", "licenses"), ("legal", "terms", "terms"), ("legal", "privacy", "privacy"), ("legal", "gdpr", "gdpr"),
            ("legal", "consent", "consents"), ("legal", "liabil", "liabilities"), ("legal", "complian", "compliance"),
            ("legal", "trademark", "trademarks"), ("legal", "patent", "patents"),
            ("irreversible", "delet", "deletes"), ("irreversible", "delet", "deletion"), ("irreversible", "drop", "drops"),
            ("irreversible", "purge", "purges"), ("irreversible", "migrat", "migrations"), ("irreversible", "irreversib", "irreversible"),
            ("irreversible", "sunset", "sunsets"), ("irreversible", "deprecat", "deprecations"),
            ("irreversible", "turn off", "turning off"), ("irreversible", "shut down", "shutting down"),
            ("outward", "deploy", "deploys"), ("outward", "release", "releases"), ("outward", "ship", "ships"), ("outward", "publish", "publishes"),
            ("outward", "announce", "announcements"), ("outward", "press", "presses"), ("outward", "public", "publicly"),
            ("secret-permission", "secret", "secrets"), ("secret-permission", "key", "keys"), ("secret-permission", "token", "tokens"),
            ("secret-permission", "password", "passwords"), ("secret-permission", "permission", "permissions"),
            ("secret-permission", "access", "accesses"), ("secret-permission", "role", "roles"), ("secret-permission", "admin", "admins"),
            ("secret-permission", "credential", "credentials"),
        ];
        for (group, stem, word) in cases {
            let hits = stem_matches(&format!("Should we change the {word} now?"));
            assert!(hits.contains(&(*group, *stem)), "{word} -> {stem}: {hits:?}");
        }
        // every stem of the table is exercised above, or by the two context rules below
        for (group, stems) in STEMS {
            for stem in stems {
                assert!(cases.iter().any(|(g, s, _)| g == &group && s == stem) || ["email", "notify all"].contains(stem), "untested stem {stem}");
            }
        }
        assert!(stem_matches("Should we email every customer?").contains(&("outward", "email")));
        assert!(stem_matches("Do we send emails to all customers?").contains(&("outward", "email")));
        assert!(!stem_matches("Should the email field be optional?").iter().any(|(_, s)| *s == "email"));
        assert!(stem_matches("Should we notify all users?").contains(&("outward", "notify all")));
        assert!(stem_matches("Notifications for everyone?").contains(&("outward", "notify all")));
        assert!(!stem_matches("Should the notification be blue?").iter().any(|(_, s)| *s == "notify all"));
        assert_eq!(stem_matches("Ten ($10) or twenty?")[0], ("money", "$"));
        assert!(stem_matches("Should we turn off the old API for all users?").contains(&("irreversible", "turn off")));
        assert!(!stem_matches("Which turn of phrase?").iter().any(|(_, s)| *s == "turn off"));
    }
    #[test]
    fn a_stem_meets_a_word_start_only() {
        // "ship" never meets "relationship", "cost" never meets "accost", "role" never meets "parole"
        assert!(stem_matches("Is the relationship worth the accost on parole?").is_empty());
        assert!(question_stems(&q("Which order should the checklist follow?", &["Install first", "Account first"])).is_empty());
    }
    #[test]
    fn sensitive_classes_are_declared_or_matched() {
        assert_eq!(sensitive_classes(&q("Should the app keep a security log of every sign-in?", &["yes", "no"])), vec!["trust"]);
        assert_eq!(sensitive_classes(&q("What should it cost?", &["9", "12"])), vec!["money", "spending"]);
        assert!(sensitive_classes(&q("Which order?", &["a", "b"])).is_empty());
        let mut d = q("Which order?", &["a", "b"]);
        d.class = "public-claim".into();
        assert_eq!(sensitive_classes(&d), vec!["public-claim"]);
    }
}
