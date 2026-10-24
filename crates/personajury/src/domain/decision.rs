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
    pub text: String,
    pub options: Vec<String>,
}

/// Parse a questions file. `Err` names the question and what is wrong with it.
pub fn parse_questions(md: &str) -> Result<Vec<Question>, String> {
    let mut out: Vec<Question> = Vec::new();
    for line in md.lines() {
        let l = line.trim_end();
        if let Some(rest) = l.strip_prefix("## ") {
            let (id, text) = rest.split_once(' ').unwrap_or((rest, ""));
            let digits = id.strip_prefix('Q').filter(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()));
            if digits.is_none() {
                return Err(format!("the heading \"{l}\" is not \"## Q<n> <question>\""));
            }
            if out.iter().any(|q| q.id == id) {
                return Err(format!("{id} is listed twice"));
            }
            if text.trim().is_empty() {
                return Err(format!("{id} has no question text"));
            }
            out.push(Question { id: id.to_string(), text: text.trim().to_string(), options: vec![] });
        } else if let Some(o) = l.strip_prefix("- ") {
            if let Some(q) = out.last_mut() {
                let o = o.trim();
                if o.is_empty() {
                    continue;
                }
                if q.options.iter().any(|x| x == o) {
                    return Err(format!("{} lists the option \"{o}\" twice", q.id));
                }
                q.options.push(o.to_string());
            }
        }
    }
    if out.is_empty() {
        return Err("no question: write \"## Q1 <question>\" and one \"- option\" line per option".into());
    }
    if let Some(q) = out.iter().find(|q| q.options.len() < 2) {
        return Err(format!("{} has {} option(s); a question needs at least two", q.id, q.options.len()));
    }
    Ok(out)
}

const CLASSES: [(&str, &[&str]); 5] = [
    (
        "public price",
        &["price", "prices", "pricing", "per seat", "per month", "a month", "each month", "discount", "refund", "free trial", "tier", "tiers"],
    ),
    (
        "public words",
        &["public", "announce", "announcement", "press", "tagline", "headline", "slogan", "blog post", "marketing copy"],
    ),
    ("legal", &["legal", "licence", "license", "terms of", "contract", "privacy policy", "gdpr", "compliance", "liability", "trademark", "patent"]),
    ("spending", &["spend", "spending", "budget", "hire", "hiring", "buy", "purchase", "invoice", "payroll", "salary", "vendor", "outside auditor", "pay for"]),
    (
        "irreversible",
        &["irreversible", "delete", "deletion", "migrate", "migration", "shut down", "sunset", "deprecate", "cannot be undone", "email customers", "release", "publish"],
    ),
];
const MONEY: [&str; 15] = ["price", "prices", "pricing", "cost", "costs", "pay", "paid", "budget", "fee", "fees", "spend", "refund", "money", "revenue", "dollars"];
const TRUST: [&str; 10] = ["trust", "security", "privacy", "safe", "safety", "secure", "confidential", "consent", "audit", "permission"];

/// Whole words (or whole phrases) of the lower-cased text: "tier" never meets "frontier", "buy" never meets "buyer".
fn has(text: &str, words: &[&str]) -> bool {
    let t = text.to_lowercase();
    let t: String = t.chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect();
    let padded = format!(" {} ", t.split_whitespace().collect::<Vec<_>>().join(" "));
    words.iter().any(|w| padded.contains(&format!(" {w} ")))
}

/// The human classes a question falls in, from its text and its options, in the fixed order of the table.
pub fn human_classes(q: &Question) -> Vec<&'static str> {
    let text = format!("{} {}", q.text, q.options.join(" "));
    CLASSES.iter().filter(|(_, words)| has(&text, words)).map(|(c, _)| *c).collect()
}
/// Whether the question is about money, and whether it is about trust: what a split on it is a split on.
pub fn split_topics(q: &Question) -> Vec<&'static str> {
    let text = format!("{} {}", q.text, q.options.join(" "));
    let mut v = Vec::new();
    if has(&text, &MONEY) || text.contains('$') {
        v.push("money");
    }
    if has(&text, &TRUST) {
        v.push("trust");
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    fn q(text: &str, options: &[&str]) -> Question {
        Question { id: "Q1".into(), text: text.into(), options: options.iter().map(|s| s.to_string()).collect() }
    }
    #[test]
    fn parses_numbered_questions_with_options() {
        let qs = parse_questions("# t\n\n## Q1 One?\n- a\n- b\n\n## Q12 Two?\n- c\n- d\n- e\n").unwrap();
        assert_eq!(qs.len(), 2);
        assert_eq!((qs[1].id.as_str(), qs[1].options.len()), ("Q12", 3));
    }
    #[test]
    fn refuses_what_the_jury_could_not_answer() {
        assert!(parse_questions("nothing").unwrap_err().contains("no question"));
        assert!(parse_questions("## Q1 x\n- a\n").unwrap_err().contains("at least two"));
        assert!(parse_questions("## Q1 x\n- a\n- a\n").unwrap_err().contains("twice"));
        assert!(parse_questions("## First?\n- a\n- b\n").unwrap_err().contains("not \"## Q<n>"));
    }
    #[test]
    fn classes_by_word_start() {
        assert_eq!(human_classes(&q("What should the plan cost per seat?", &["9", "12"])), vec!["public price"]);
        assert_eq!(human_classes(&q("Should we hire an auditor?", &["yes", "no"])), vec!["spending"]);
        assert!(human_classes(&q("Which order should the checklist follow?", &["a", "b"])).is_empty());
        // "tier" must not meet "frontier"; "buy" must not meet "buyer"
        assert!(human_classes(&q("Is the frontier ready for the buyer?", &["a", "b"])).is_empty());
    }
}
