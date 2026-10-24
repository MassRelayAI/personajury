//! Traits as data (D29): a trait file, a trait set, the values a persona holds, and the checks that guard them. Pure.
//! A trait is a tendency or a standing fact that makes one reader decide differently from another, never a type or a
//! diagnosis. It is an input, never a score: nothing here averages, and nothing turns a trait into a probability.
use super::hash::{canonical_json, sha256_hex};
use super::refuse::Refusal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The fourteen families the catalog ships; any other family is a project's own (`project.<slug>`).
pub const FAMILIES: [&str; 14] = [
    "personality", "risk", "trust", "cognition", "choice", "adoption", "consumer", "persuasion", "buying", "situation",
    "expertise", "aesthetics", "social", "access",
];
/// The decisions a trait may be known to move.
pub const MOVES: [&str; 6] = ["buy", "trust_a_claim", "adopt", "read_docs", "pick_name_or_design", "approve_vendor"];
pub const EVIDENCE: [&str; 4] = ["strong", "moderate", "weak", "project"];
/// The most traits a persona holds, a set lists, and a cover names (D29, D30).
pub const MAX_PERSONA_TRAITS: usize = 8;
pub const MAX_SET_TRAITS: usize = 8;
pub const MAX_COVER: usize = 4;

const TYPOLOGY: &str = "typology without predictive validity";
const COUNTRY: &str = "a country dimension, not a mechanism";
const ATTRIBUTE: &str = "an attribute of the persona, not a sensitivity of the juror";
/// Ids (or last segments of ids) no trait may carry, and why (D29): typologies and attributes of the persona. A protected
/// attribute is no longer refused: it is labelled (D39).
pub const REFUSED: [(&str, &str); 16] = [
    ("mbti", TYPOLOGY),
    ("myers_briggs", TYPOLOGY),
    ("enneagram", TYPOLOGY),
    ("disc", TYPOLOGY),
    ("learning_style", TYPOLOGY),
    ("generation", TYPOLOGY),
    ("astrology", TYPOLOGY),
    ("zodiac", TYPOLOGY),
    ("blood_type", TYPOLOGY),
    ("appearance", ATTRIBUTE),
    ("attractiveness", ATTRIBUTE),
    ("beauty", ATTRIBUTE),
    ("fame", ATTRIBUTE),
    ("celebrity", ATTRIBUTE),
    ("outcast", ATTRIBUTE),
    ("outsider", ATTRIBUTE),
];

/// How a stem of the label table meets a word (D39).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    /// anywhere inside a word: `bisexual`, `unhealthy`, `nondisabled`
    Within,
    /// at the start of a word: `colorful`, not `international`
    Start,
    /// the whole word only, for a short or ambiguous stem: `trans`, not `transform`; `alter`, not `alternative`
    Word,
}
use Fit::{Start, Within, Word};
/// The categories a label names, in the order an answer lists them (D39).
pub const SENSITIVE: [&str; 12] = [
    "age", "gender", "ethnicity", "religion", "nationality", "health", "disability", "sexuality", "politics", "marital",
    "pregnancy", "genetics",
];
/// The sensitive-attribute label (D39, the detection of D29 as ruled after the v0.6-B review), the one table: every text of
/// every trait (id, family, title, caveats, source, each anchor, each option's id, title and sentence), every free-text
/// value and a persona's prose are read against it, and each hit names one of `SENSITIVE`. Nothing is refused: personas
/// are simulations, and a trait or persona that names a sensitive attribute is accepted and labelled `sensitive: [...]`,
/// and a run that seats it records `uses_sensitive`. Texts and stems are normalised first (`normalise`), and each text is
/// read twice: word by word, and with the separators `_ - .` and space removed between words, so `gen_der` is `gender`. It
/// errs wide: `colorful` and `unions` are labelled too.
pub const SENSITIVE_STEMS: [(&str, &str, Fit); 64] = [
    ("age", "age", Start),
    ("gender", "gender", Within),
    ("sex", "gender", Start),
    ("sexual", "sexuality", Within),
    ("orientation", "sexuality", Start),
    ("race", "ethnicity", Start),
    ("racial", "ethnicity", Within),
    ("ethnic", "ethnicity", Within),
    ("religion", "religion", Within),
    ("religio", "religion", Within),
    ("faith", "religion", Within),
    ("caste", "ethnicity", Start),
    ("nationality", "nationality", Start),
    ("national", "nationality", Start),
    ("citizenship", "nationality", Within),
    ("country", "nationality", Start),
    ("pregnan", "pregnancy", Within),
    ("marital", "marital", Within),
    ("disab", "disability", Within),
    ("politic", "politics", Within),
    ("union", "politics", Start),
    ("health", "health", Within),
    ("genetic", "genetics", Within),
    ("skin", "ethnicity", Start),
    ("color", "ethnicity", Start),
    ("colour", "ethnicity", Start),
    ("tribe", "ethnicity", Start),
    ("tribal", "ethnicity", Within),
    // added after the v0.6-B review: gender identity, age, religion and their words in other languages
    ("trans", "gender", Word),
    ("transgender", "gender", Within),
    ("nonbinary", "gender", Within),
    ("elderly", "age", Within),
    ("senior citizen", "age", Within),
    ("pensioner", "age", Within),
    ("muslim", "religion", Within),
    ("christian", "religion", Within),
    ("jewish", "religion", Within),
    ("hindu", "religion", Within),
    ("buddhist", "religion", Within),
    ("sikh", "religion", Word),
    ("islam", "religion", Within),
    ("catholic", "religion", Within),
    ("atheis", "religion", Within),
    ("etni", "ethnicity", Within),
    ("\u{e9}tni", "ethnicity", Within),
    ("ethnie", "ethnicity", Within),
    ("raza", "ethnicity", Word),
    ("rasse", "ethnicity", Word),
    ("religi", "religion", Within),
    ("nacional", "nationality", Start),
    ("nationalit", "nationality", Within),
    ("g\u{e9}nero", "gender", Word),
    ("genero", "gender", Word),
    ("geschlecht", "gender", Within),
    ("edad", "age", Word),
    ("alter", "age", Word),
    ("schwanger", "pregnancy", Within),
    ("behinder", "disability", Within),
    ("handicap", "disability", Within),
    // the review's synonyms
    ("female", "gender", Within),
    ("male", "gender", Word),
    ("woman", "gender", Word),
    ("women", "gender", Word),
    ("immigra", "nationality", Within),
];
/// Words that start with a stem and name no protected attribute: `agent` and `agency` are not `age`.
const STEM_EXCEPTIONS: [(&str, [&str; 3]); 1] = [("age", ["agent", "agenc", "agend"])];
/// Short words of ordinary prose that never begin a joined split (the second check): "a general" is not "age...", "be
/// hindered" is not "behinder".
const JOIN_STOPS: [&str; 24] = [
    "a", "an", "the", "of", "to", "in", "is", "be", "by", "as", "at", "on", "or", "it", "we", "us", "no", "so", "do", "my",
    "if", "up", "and", "for",
];
/// Families no trait may belong to (`project.<f>` included): a culture, the country dimension (D29). A demographic family
/// is a simulation's own and is labelled, not refused (D39).
pub const REFUSED_FAMILIES: [&str; 1] = ["culture"];

/// Latin look-alikes of Cyrillic and Greek capitals, before lower-casing (a Greek capital eta is an H, its small letter is not).
const CAPITAL_LOOKALIKES: [(char, &str); 30] = [
    ('\u{0391}', "a"), ('\u{0392}', "b"), ('\u{0395}', "e"), ('\u{0396}', "z"), ('\u{0397}', "h"), ('\u{0399}', "i"),
    ('\u{039A}', "k"), ('\u{039C}', "m"), ('\u{039D}', "n"), ('\u{039F}', "o"), ('\u{03A1}', "p"), ('\u{03A4}', "t"),
    ('\u{03A5}', "y"), ('\u{03A7}', "x"), ('\u{0410}', "a"), ('\u{0412}', "b"), ('\u{0415}', "e"), ('\u{041A}', "k"),
    ('\u{041C}', "m"), ('\u{041D}', "h"), ('\u{041E}', "o"), ('\u{0420}', "p"), ('\u{0421}', "c"), ('\u{0422}', "t"),
    ('\u{0425}', "x"), ('\u{0405}', "s"), ('\u{0406}', "i"), ('\u{0408}', "j"), ('\u{04C0}', "l"), ('\u{0423}', "y"),
];
/// Latin look-alikes of Cyrillic and Greek small letters, and the Latin letters with a mark or a ligature folded to plain
/// letters (the part of NFKC and of the confusables list the screen needs; no crate).
const SMALL_FOLDS: [(char, &str); 118] = [
    // Cyrillic
    ('\u{0430}', "a"), ('\u{0432}', "b"), ('\u{0435}', "e"), ('\u{0451}', "e"), ('\u{043A}', "k"), ('\u{043C}', "m"),
    ('\u{043D}', "h"), ('\u{043E}', "o"), ('\u{0440}', "p"), ('\u{0441}', "c"), ('\u{0442}', "t"), ('\u{0443}', "y"),
    ('\u{0445}', "x"), ('\u{0455}', "s"), ('\u{0456}', "i"), ('\u{0457}', "i"), ('\u{0458}', "j"), ('\u{0501}', "d"),
    ('\u{04CF}', "l"), ('\u{04BB}', "h"), ('\u{051B}', "q"), ('\u{051D}', "w"),
    // Greek
    ('\u{03B1}', "a"), ('\u{03B5}', "e"), ('\u{03B9}', "i"), ('\u{03BA}', "k"), ('\u{03BD}', "v"), ('\u{03BF}', "o"),
    ('\u{03C1}', "p"), ('\u{03C4}', "t"), ('\u{03C5}', "u"), ('\u{03C7}', "x"), ('\u{03F2}', "c"), ('\u{03AF}', "i"),
    ('\u{03CC}', "o"), ('\u{03AD}', "e"), ('\u{03AC}', "a"),
    // Latin with marks
    ('\u{e0}', "a"), ('\u{e1}', "a"), ('\u{e2}', "a"), ('\u{e3}', "a"), ('\u{e4}', "a"), ('\u{e5}', "a"), ('\u{101}', "a"),
    ('\u{103}', "a"), ('\u{105}', "a"), ('\u{e7}', "c"), ('\u{107}', "c"), ('\u{109}', "c"), ('\u{10b}', "c"), ('\u{10d}', "c"),
    ('\u{10f}', "d"), ('\u{111}', "d"), ('\u{e8}', "e"), ('\u{e9}', "e"), ('\u{ea}', "e"), ('\u{eb}', "e"), ('\u{113}', "e"),
    ('\u{115}', "e"), ('\u{117}', "e"), ('\u{119}', "e"), ('\u{11b}', "e"), ('\u{ec}', "i"), ('\u{ed}', "i"), ('\u{ee}', "i"),
    ('\u{ef}', "i"), ('\u{129}', "i"), ('\u{12b}', "i"), ('\u{12d}', "i"), ('\u{12f}', "i"), ('\u{131}', "i"), ('\u{f1}', "n"),
    ('\u{144}', "n"), ('\u{146}', "n"), ('\u{148}', "n"), ('\u{f2}', "o"), ('\u{f3}', "o"), ('\u{f4}', "o"), ('\u{f5}', "o"),
    ('\u{f6}', "o"), ('\u{f8}', "o"), ('\u{14d}', "o"), ('\u{151}', "o"), ('\u{f9}', "u"), ('\u{fa}', "u"), ('\u{fb}', "u"),
    ('\u{fc}', "u"), ('\u{169}', "u"), ('\u{16b}', "u"), ('\u{16f}', "u"), ('\u{171}', "u"), ('\u{173}', "u"), ('\u{fd}', "y"),
    ('\u{ff}', "y"), ('\u{15b}', "s"), ('\u{15f}', "s"), ('\u{161}', "s"), ('\u{219}', "s"), ('\u{163}', "t"), ('\u{165}', "t"),
    ('\u{21b}', "t"), ('\u{17a}', "z"), ('\u{17c}', "z"), ('\u{17e}', "z"), ('\u{11f}', "g"), ('\u{123}', "g"), ('\u{261}', "g"),
    ('\u{142}', "l"), ('\u{13e}', "l"), ('\u{159}', "r"), ('\u{df}', "ss"), ('\u{e6}', "ae"), ('\u{153}', "oe"),
    // ligatures (NFKC)
    ('\u{fb00}', "ff"), ('\u{fb01}', "fi"), ('\u{fb02}', "fl"), ('\u{fb03}', "ffi"), ('\u{fb04}', "ffl"),
];
/// Characters that print nothing: zero-width spaces and joiners, the soft hyphen, the word joiner, the byte-order mark and
/// the direction marks.
fn is_invisible(c: char) -> bool {
    matches!(c, '\u{00AD}' | '\u{180E}' | '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}')
}
/// Combining marks (a `g` followed by an accent is a `g`).
fn is_combining(c: char) -> bool {
    matches!(c, '\u{0300}'..='\u{036F}' | '\u{1AB0}'..='\u{1AFF}' | '\u{1DC0}'..='\u{1DFF}' | '\u{20D0}'..='\u{20FF}' | '\u{FE20}'..='\u{FE2F}')
}
/// The NFKC compatibility forms the screen folds: fullwidth ASCII and the mathematical alphanumeric letters.
fn compat(c: char) -> char {
    let n = c as u32;
    match n {
        0xFF01..=0xFF5E => char::from_u32(n - 0xFF01 + 0x21).unwrap_or(c),
        0x1D400..=0x1D6A3 => {
            let k = (n - 0x1D400) % 52;
            char::from_u32(if k < 26 { 0x41 + k } else { 0x61 + k - 26 }).unwrap_or(c)
        }
        _ => c,
    }
}
/// The text the screen reads: the NFKC compatibility forms folded, the look-alikes of Cyrillic and Greek mapped to Latin,
/// lower case, invisible characters and combining marks removed, marked Latin letters folded to plain ones. A small,
/// documented subset of Unicode NFKC and of the confusables list, without a crate (D29 as ruled).
pub fn normalise(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars().map(compat) {
        if is_invisible(c) || is_combining(c) {
            continue;
        }
        if let Some((_, s)) = CAPITAL_LOOKALIKES.iter().find(|(k, _)| *k == c) {
            out.push_str(s);
            continue;
        }
        for l in c.to_lowercase() {
            if is_combining(l) {
                continue;
            }
            match SMALL_FOLDS.iter().find(|(k, _)| *k == l) {
                Some((_, s)) => out.push_str(s),
                None => out.push(l),
            }
        }
    }
    out
}
fn is_sep(c: char) -> bool {
    matches!(c, '_' | '-' | '.') || c.is_whitespace()
}
/// A stem as the screen matches it: normalised, its separators removed (`senior citizen` is `seniorcitizen`).
fn stem_key(stem: &str) -> String {
    normalise(stem).chars().filter(|c| !is_sep(*c)).collect()
}
fn excepted(stem: &str, tail: &str) -> bool {
    STEM_EXCEPTIONS.iter().any(|(s, words)| *s == stem && words.iter().any(|w| tail.starts_with(w)))
}
/// Whether a stem meets a word read whole (the first check).
fn in_word(word: &str, stem: &str, fit: Fit) -> bool {
    let k = stem_key(stem);
    match fit {
        Within => word.contains(&k),
        Start => word.starts_with(&k) && !excepted(stem, word),
        Word => word == k,
    }
}
/// Whether a stem is made by two or more words once the separators between them are removed (the second check): it starts
/// where a word starts and runs into the next word; a whole-word stem ends where a word ends. A short prose word
/// (`JOIN_STOPS`) never starts one.
fn in_join(words: &[&str], stem: &str, fit: Fit) -> bool {
    let k = stem_key(stem);
    (0..words.len().saturating_sub(1)).any(|i| {
        if JOIN_STOPS.contains(&words[i]) || k.len() <= words[i].len() {
            return false;
        }
        let tail: String = words[i..].concat();
        if !tail.starts_with(&k) {
            return false;
        }
        match fit {
            Word => {
                let mut at = 0;
                words[i..].iter().any(|w| {
                    at += w.len();
                    at == k.len()
                })
            }
            Start => !excepted(stem, &tail),
            Within => true,
        }
    })
}
/// The categories of `SENSITIVE` that one text names, in that order: normalised, then read word by word and with the
/// separators removed.
pub fn sensitive_in(text: &str) -> Vec<&'static str> {
    let n = normalise(text);
    let words: Vec<&str> = n.split(|c: char| !c.is_alphabetic()).filter(|w| !w.is_empty()).collect();
    let segments: Vec<Vec<&str>> = n
        .split(|c: char| !(c.is_alphabetic() || is_sep(c)))
        .map(|seg| seg.split(is_sep).filter(|w| !w.is_empty()).collect())
        .collect();
    let mut hit: Vec<&'static str> = SENSITIVE_STEMS
        .iter()
        .filter(|(stem, _, fit)| {
            words.iter().any(|w| in_word(w, stem, *fit)) || segments.iter().any(|ws| in_join(ws, stem, *fit))
        })
        .map(|(_, what, _)| *what)
        .collect();
    order(&mut hit);
    hit
}
/// Categories in the order of `SENSITIVE`, once each.
pub fn order(cats: &mut Vec<&'static str>) {
    cats.sort_by_key(|c| SENSITIVE.iter().position(|s| s == c).unwrap_or(usize::MAX));
    cats.dedup();
}
/// The categories several texts name, in order.
pub fn sensitive_of<'a>(texts: impl IntoIterator<Item = &'a str>) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = texts.into_iter().flat_map(sensitive_in).collect();
    order(&mut out);
    out
}
/// The screen of a trait id: a refused family (its first segment, D29). A sensitive word in the id is labelled, not refused.
pub fn screen_id(id: &str) -> Result<(), Refusal> {
    let first = id.split('.').next().unwrap_or(id);
    if REFUSED_FAMILIES.contains(&first) {
        return Err(Refusal::RefusedTraitId { id: id.to_string(), why: COUNTRY.to_string() });
    }
    Ok(())
}
/// Every text of a trait, each with where it stands: id, family, title, caveats, source, each anchor, each option's id,
/// title and sentence.
pub fn trait_texts(t: &TraitDef) -> Vec<(String, String)> {
    let mut out = vec![
        ("the id".to_string(), t.id.clone()),
        ("the family".to_string(), t.family.clone()),
        ("the title".to_string(), t.title.clone()),
        ("the caveats".to_string(), t.caveats.clone()),
        ("the source".to_string(), t.source.clone()),
    ];
    for (k, s) in &t.anchors {
        out.push((format!("anchor {k}"), s.clone()));
    }
    for o in &t.options {
        out.push((format!("option {}", o.id), o.id.clone()));
        out.push((format!("option {} title", o.id), o.title.clone()));
        out.push((format!("option {} sentence", o.id), o.sentence.clone()));
    }
    out
}
/// The refusal a trait still meets (D29): a refused family (`culture`, as `project.culture` too). Nothing else of a trait's
/// words is refused; `trait_sensitive` labels them (D39).
pub fn screen_trait(t: &TraitDef) -> Result<(), Refusal> {
    let family = t.family.strip_prefix("project.").unwrap_or(&t.family);
    if REFUSED_FAMILIES.contains(&family) || REFUSED_FAMILIES.contains(&t.id.split('.').next().unwrap_or("")) {
        return Err(Refusal::RefusedTraitId { id: t.id.clone(), why: COUNTRY.to_string() });
    }
    Ok(())
}
/// The sensitive categories a trait names (D39), over every text. `shipped` is the catalog's own trait of the same id, when
/// there is one: a text the catalog ships unchanged carries the catalog's reviewed label (`shipped_label`), an edited one is
/// read again.
pub fn trait_sensitive(t: &TraitDef, shipped: Option<&TraitDef>) -> Vec<&'static str> {
    let reviewed = shipped.map(trait_texts).unwrap_or_default();
    let mut out: Vec<&'static str> = shipped.map(|s| shipped_label(&s.id).to_vec()).unwrap_or_default();
    for (place, text) in trait_texts(t) {
        if !reviewed.iter().any(|(p, x)| *p == place && *x == text) {
            out.extend(sensitive_in(&text));
        }
    }
    order(&mut out);
    out
}
/// The labels of the shipped traits, reviewed by hand (D39): a shipped trait is labelled only where it truly names a
/// sensitive attribute, not where a stem falls on a word by chance ("in good faith", "colour vision").
pub const SHIPPED_LABELS: [(&str, &[&str]); 1] = [("risk.health_safety", &["health"])];
pub fn shipped_label(id: &str) -> &'static [&'static str] {
    SHIPPED_LABELS.iter().find(|(i, _)| *i == id).map(|(_, l)| *l).unwrap_or(&[])
}
/// The whole check of a traits map a persona or a person holds (`persona add`, `trait suggest --check`, `verdict --by-human
/// --traits`): the refused list and the ceiling, then each trait the store holds, then the value.
pub fn check_held(map: &BTreeMap<String, TraitValue>, defs: &BTreeMap<String, TraitDef>) -> Result<(), Refusal> {
    check_map_shape(map)?;
    for (id, v) in map {
        let def = defs.get(id).ok_or_else(|| Refusal::UnknownTrait { id: id.clone() })?;
        screen_trait(def)?;
        def.check_value(v)?;
    }
    Ok(())
}
/// The sensitive categories a held map names (D39): each trait's own label (`trait_sensitive`), and each free-text value.
pub fn held_sensitive(map: &BTreeMap<String, TraitValue>, defs: &BTreeMap<String, TraitDef>) -> Vec<&'static str> {
    let mut out = Vec::new();
    for (id, v) in map {
        if let Some(def) = defs.get(id) {
            let shipped = super::catalog::shipped_text(id).and_then(|t| TraitDef::from_toml(t).ok());
            out.extend(trait_sensitive(def, shipped.as_ref()));
        }
        if let TraitValue::Str(s) = v {
            if defs.get(id).is_some_and(|d| d.shape == "words") {
                out.extend(sensitive_in(s));
            }
        }
    }
    order(&mut out);
    out
}
/// The most characters a free-text trait value holds, whatever the trait's `max_chars` says (D31 as ruled).
pub const MAX_TRAIT_TEXT: usize = 80;
/// A free-text value (`situation.job`): one line of at most 80 characters, no digit, not a self-description ("You are",
/// "I am") (`trait_text_refused`). A sensitive word in it is labelled, not refused (D39).
pub fn check_text(id: &str, s: &str, max: usize) -> Result<(), Refusal> {
    let one_line = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let lower = one_line.to_lowercase();
    let n = one_line.chars().count();
    let why = if one_line.is_empty() {
        Some("empty".to_string())
    } else if n > max.min(MAX_TRAIT_TEXT) {
        Some(format!("{n} characters; at most {}", max.min(MAX_TRAIT_TEXT)))
    } else if one_line.chars().any(|c| c.is_ascii_digit()) {
        Some("holds a digit; describe the work, not a number about the person".to_string())
    } else if lower.starts_with("you are") || lower.starts_with("i am") || lower.starts_with("i'm") {
        Some("starts as a self-description (\"You are\", \"I am\"); describe the work".to_string())
    } else {
        None
    };
    match why {
        Some(why) => Err(Refusal::TraitTextRefused { id: id.to_string(), why }),
        None => Ok(()),
    }
}
/// Why `id` may not be a trait id, when its last segment (or the whole id) is on the refused list.
pub fn refused_why(id: &str) -> Option<&'static str> {
    let last = id.rsplit('.').next().unwrap_or(id);
    REFUSED.iter().find(|(w, _)| *w == last || *w == id).map(|(_, why)| *why)
}
/// The refusal for `id` when it is on the list.
pub fn check_not_refused(id: &str) -> Result<(), Refusal> {
    match refused_why(id) {
        Some(why) => Err(Refusal::RefusedTraitId { id: id.to_string(), why: why.to_string() }),
        None => Ok(()),
    }
}

/// What a persona holds for a trait: a band (integer), a category option or a words value (string), or a list of
/// options (a multi category). Read from JSON by hand: `arbitrary_precision` does not deserialise untagged numbers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum TraitValue {
    Int(i64),
    Str(String),
    List(Vec<String>),
}
impl<'de> Deserialize<'de> for TraitValue {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let v = serde_json::Value::deserialize(d)?;
        match &v {
            serde_json::Value::Number(n) => match n.as_i64() {
                Some(i) => Ok(TraitValue::Int(i)),
                None => Err(D::Error::custom(format!("a trait value must be an integer, a string or a list of strings, not {n}"))),
            },
            serde_json::Value::String(s) => Ok(TraitValue::Str(s.clone())),
            serde_json::Value::Array(a) => a
                .iter()
                .map(|x| x.as_str().map(str::to_string))
                .collect::<Option<Vec<_>>>()
                .map(TraitValue::List)
                .ok_or_else(|| D::Error::custom("a list trait value holds strings only")),
            other => Err(D::Error::custom(format!(
                "a trait value must be an integer, a string or a list of strings, not {other}"
            ))),
        }
    }
}
impl TraitValue {
    /// The value as the refusals print it.
    pub fn show(&self) -> String {
        match self {
            TraitValue::Int(i) => i.to_string(),
            TraitValue::Str(s) => s.clone(),
            TraitValue::List(l) => format!("[{}]", l.join(", ")),
        }
    }
    /// A multi-category list in one order, once each (a value is a set; its hash must not depend on the order given).
    pub fn canonical(&self) -> TraitValue {
        match self {
            TraitValue::List(l) => {
                let mut l = l.clone();
                l.sort();
                l.dedup();
                TraitValue::List(l)
            }
            other => other.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitOption {
    pub id: String,
    pub title: String,
    pub sentence: String,
}

/// A trait file as `traits/<family>/<trait>.toml` reads and as `trait.body_json` stores it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitDef {
    pub id: String,
    pub version: i64,
    pub family: String,
    pub title: String,
    /// `band` | `category` | `words`
    pub shape: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<[i64; 2]>,
    /// a category a persona may hold several options of (`access.need`)
    #[serde(default, skip_serializing_if = "is_false")]
    pub multi: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<usize>,
    #[serde(default)]
    pub moves: Vec<String>,
    pub evidence: String,
    pub source: String,
    /// must be present and `false`: the catalog describes constructs, it never copies questionnaire items
    #[serde(default)]
    pub items_copied: Option<bool>,
    #[serde(default)]
    pub correlates_with: Vec<String>,
    #[serde(default)]
    pub caveats: String,
    #[serde(default)]
    pub anchors: BTreeMap<String, String>,
    #[serde(default, rename = "option")]
    pub options: Vec<TraitOption>,
}
fn is_false(b: &bool) -> bool {
    !*b
}

fn invalid(why: impl Into<String>) -> Refusal {
    Refusal::InvalidTraitFile { why: why.into() }
}
fn id_ok(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}
/// `^[a-z0-9_]+\.[a-z0-9_]+$`
pub fn trait_id_ok(id: &str) -> bool {
    matches!(id.split_once('.'), Some((a, b)) if id_ok(a) && id_ok(b))
}
/// A sentence an anchor or an option holds: a predicate that follows "You tend to", 40 to 400 characters, lower case at
/// the start, no "I " start, and no "agree" or "disagree" (that is a questionnaire item, not a tendency).
pub fn check_sentence(where_: &str, s: &str) -> Result<(), Refusal> {
    let n = s.chars().count();
    if !(40..=400).contains(&n) {
        return Err(invalid(format!("{where_}: {n} characters; a sentence is 40 to 400")));
    }
    let first = s.chars().next().unwrap_or(' ');
    if !first.is_lowercase() {
        return Err(invalid(format!("{where_}: starts with an upper-case letter or a symbol; it follows \"You tend to\", so start in lower case")));
    }
    if s.starts_with("I ") {
        return Err(invalid(format!("{where_}: starts with \"I \"; write what you tend to do, not an item")));
    }
    if s.to_lowercase().contains("agree") {
        return Err(invalid(format!("{where_}: contains \"agree\" or \"disagree\"; that is a questionnaire item, not a tendency")));
    }
    Ok(())
}

impl TraitDef {
    pub fn from_toml(s: &str) -> Result<TraitDef, Refusal> {
        // the id is checked against the refused list before anything else, so the refusal names the reason
        if let Ok(v) = toml::from_str::<toml::Value>(s) {
            if let Some(id) = v.get("id").and_then(toml::Value::as_str) {
                check_not_refused(id)?;
            }
        }
        let d: TraitDef = toml::from_str(s).map_err(|e| invalid(e.to_string()))?;
        d.validate()?;
        Ok(d)
    }
    pub fn validate(&self) -> Result<(), Refusal> {
        check_not_refused(&self.id)?;
        if !trait_id_ok(&self.id) {
            return Err(invalid(format!("id {:?} must match ^[a-z0-9_]+\\.[a-z0-9_]+$", self.id)));
        }
        let first = self.id.split('.').next().unwrap_or("");
        let project_ok = self.family.strip_prefix("project.").is_some_and(|slug| id_ok(slug) && slug == first);
        if self.family != first && !project_ok {
            return Err(invalid(format!(
                "family {:?} must be the first segment of the id ({first}), or project.{first} for a project's own trait",
                self.family
            )));
        }
        if self.version < 1 {
            return Err(invalid("version must be 1 or more"));
        }
        if self.title.trim().is_empty() {
            return Err(invalid("title is empty"));
        }
        if !EVIDENCE.contains(&self.evidence.as_str()) {
            return Err(invalid(format!("evidence {:?} is not one of {}", self.evidence, EVIDENCE.join(", "))));
        }
        match self.items_copied {
            Some(false) => {}
            Some(true) => return Err(invalid("items_copied is true; the catalog never copies questionnaire items")),
            None => return Err(invalid("items_copied is missing; state items_copied = false")),
        }
        for m in &self.moves {
            if !MOVES.contains(&m.as_str()) {
                return Err(invalid(format!("moves names {m:?}; the words are {}", MOVES.join(", "))));
            }
        }
        if self.correlates_with.contains(&self.id) {
            return Err(invalid("correlates_with names the trait itself"));
        }
        match self.shape.as_str() {
            "band" => {
                if self.scale != Some([1, 5]) {
                    return Err(invalid("a band trait has scale = [1, 5]"));
                }
                let keys: Vec<&str> = self.anchors.keys().map(String::as_str).collect();
                if keys != ["1", "3", "5"] {
                    return Err(invalid(format!("a band trait has exactly the anchors 1, 3 and 5, not {keys:?}")));
                }
                for (k, s) in &self.anchors {
                    check_sentence(&format!("anchor {k}"), s)?;
                }
                if !self.options.is_empty() || self.multi || self.max_chars.is_some() {
                    return Err(invalid("a band trait has anchors only: no option, multi or max_chars"));
                }
            }
            "category" => {
                if self.scale.is_some() || !self.anchors.is_empty() || self.max_chars.is_some() {
                    return Err(invalid("a category trait has options only: no scale, anchors or max_chars"));
                }
                if self.options.len() < 2 {
                    return Err(invalid("a category trait has at least two options"));
                }
                let mut seen = std::collections::BTreeSet::new();
                for o in &self.options {
                    if !id_ok(&o.id) {
                        return Err(invalid(format!("option id {:?} must match ^[a-z0-9_]+$", o.id)));
                    }
                    if !seen.insert(o.id.as_str()) {
                        return Err(invalid(format!("option {} appears twice", o.id)));
                    }
                    check_sentence(&format!("option {}", o.id), &o.sentence)?;
                }
            }
            "words" => {
                if self.scale.is_some() || !self.anchors.is_empty() || !self.options.is_empty() || self.multi {
                    return Err(invalid("a words trait has no scale, anchors or options"));
                }
                if self.max_chars.is_none_or(|m| m == 0) {
                    return Err(invalid("a words trait has max_chars"));
                }
            }
            other => return Err(invalid(format!("shape {other:?} is not band, category or words"))),
        }
        Ok(())
    }
    /// sha256 of the canonical JSON with `version` set to 0, so a version bump alone never changes it.
    pub fn body_and_hash(&self) -> (String, String) {
        let mut c = self.clone();
        c.version = 0;
        let v = serde_json::to_value(&c).expect("trait serialises");
        let body = canonical_json(&v);
        (body.clone(), sha256_hex(body.as_bytes()))
    }
    /// The bands a cover counts: low, mid, high for a band trait; the options of a category; none for words.
    pub fn bands(&self) -> Vec<String> {
        match self.shape.as_str() {
            "band" => vec!["low".into(), "mid".into(), "high".into()],
            "category" => self.options.iter().map(|o| o.id.clone()).collect(),
            _ => vec![],
        }
    }
    /// The bands a held value stands in (a multi category holds several; a value off the trait holds none).
    pub fn bands_held(&self, v: &TraitValue) -> Vec<String> {
        match (self.shape.as_str(), v) {
            ("band", TraitValue::Int(n)) => vec![band_word(*n).into()],
            ("category", TraitValue::Str(s)) if self.options.iter().any(|o| o.id == *s) => vec![s.clone()],
            ("category", TraitValue::List(l)) => {
                let mut out: Vec<String> = self.options.iter().filter(|o| l.contains(&o.id)).map(|o| o.id.clone()).collect();
                out.dedup();
                out
            }
            _ => vec![],
        }
    }
    pub fn option_title(&self, id: &str) -> Option<&str> {
        self.options.iter().find(|o| o.id == id).map(|o| o.title.as_str())
    }
    /// Refuse a value this trait cannot hold (`trait_value_off_scale`, `trait_option_unknown`).
    pub fn check_value(&self, v: &TraitValue) -> Result<(), Refusal> {
        let off = || Refusal::TraitValueOffScale { id: self.id.clone(), value: v.show() };
        let unknown = || Refusal::TraitOptionUnknown { id: self.id.clone(), value: v.show() };
        match self.shape.as_str() {
            "band" => match v {
                TraitValue::Int(n) if (1..=5).contains(n) => Ok(()),
                _ => Err(off()),
            },
            "words" => match v {
                TraitValue::Str(s) => check_text(&self.id, s, self.max_chars.unwrap_or(MAX_TRAIT_TEXT)),
                _ => Err(off()),
            },
            _ => match v {
                TraitValue::Str(s) if self.options.iter().any(|o| o.id == *s) => Ok(()),
                TraitValue::List(l) if self.multi && !l.is_empty() && l.iter().all(|s| self.options.iter().any(|o| o.id == *s)) => Ok(()),
                _ => Err(unknown()),
            },
        }
    }
    /// The lines the brief prints for a held value, without the leading "- ": the heading's phrasing rule (D31).
    pub fn bullets(&self, v: &TraitValue) -> Vec<String> {
        match (self.shape.as_str(), v) {
            ("band", TraitValue::Int(n)) => {
                let (lead, key) = match n {
                    1 => ("You tend to", "1"),
                    2 => ("You tend to lean this way", "1"),
                    3 => ("You tend to", "3"),
                    4 => ("You tend to lean this way", "5"),
                    _ => ("You tend to", "5"),
                };
                self.anchors.get(key).map(|a| vec![format!("{lead}: {a}")]).unwrap_or_default()
            }
            ("category", TraitValue::Str(s)) => self
                .options
                .iter()
                .filter(|o| o.id == *s)
                .map(|o| format!("You tend to: {}", o.sentence))
                .collect(),
            ("category", TraitValue::List(l)) => self
                .options
                .iter()
                .filter(|o| l.contains(&o.id))
                .map(|o| format!("You tend to: {}", o.sentence))
                .collect(),
            ("words", TraitValue::Str(s)) => {
                let one_line = s.split_whitespace().collect::<Vec<_>>().join(" ");
                vec![format!("Your work: {one_line}")]
            }
            _ => vec![],
        }
    }
}

/// low = 1 to 2, mid = 3, high = 4 to 5 (D30).
pub fn band_word(n: i64) -> &'static str {
    match n {
        i64::MIN..=2 => "low",
        3 => "mid",
        _ => "high",
    }
}

/// The derived persuasion route (D31): central when involvement and need for cognition are both 4 or more, else peripheral.
pub fn route_line(involvement: &TraitValue, need_for_cognition: &TraitValue) -> Option<&'static str> {
    match (involvement, need_for_cognition) {
        (TraitValue::Int(a), TraitValue::Int(b)) => Some(if *a >= 4 && *b >= 4 {
            "You tend to: read the argument itself and weigh what it proves, more than who says it or how it looks."
        } else {
            "You tend to: weigh the cues first, who says it, how it looks, how long it is, and read the argument only if the cues pass."
        }),
        _ => None,
    }
}

/// A trait set as `trait-sets/<name>.toml` reads and as `trait_set.body_json` stores it (D29).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitSetDef {
    pub name: String,
    pub version: i64,
    pub traits: Vec<String>,
    #[serde(default)]
    pub cover: Vec<String>,
    #[serde(default)]
    pub note: String,
}
impl TraitSetDef {
    pub fn from_toml(s: &str) -> Result<TraitSetDef, Refusal> {
        let d: TraitSetDef = toml::from_str(s).map_err(|e| invalid(format!("trait set: {e}")))?;
        d.validate()?;
        Ok(d)
    }
    pub fn validate(&self) -> Result<(), Refusal> {
        if !super::model::is_slug(&self.name) || self.name == "all" || self.name == "off" {
            return Err(invalid(format!("trait set name {:?} must be a slug and not \"all\" or \"off\"", self.name)));
        }
        if self.version < 1 {
            return Err(invalid("trait set: version must be 1 or more"));
        }
        if self.traits.len() > MAX_SET_TRAITS {
            return Err(Refusal::TraitSetTooLarge { name: self.name.clone(), traits: self.traits.len() });
        }
        if self.cover.len() > MAX_COVER {
            return Err(Refusal::CoverTooLarge { name: self.name.clone(), cover: self.cover.len() });
        }
        for t in &self.traits {
            check_not_refused(t)?;
        }
        if let Some(c) = self.cover.iter().find(|c| !self.traits.contains(c)) {
            return Err(Refusal::UnknownTrait { id: c.clone() });
        }
        Ok(())
    }
    pub fn body_and_hash(&self) -> (String, String) {
        let mut c = self.clone();
        c.version = 0;
        let v = serde_json::to_value(&c).expect("set serialises");
        let body = canonical_json(&v);
        (body.clone(), sha256_hex(body.as_bytes()))
    }
}

/// Every pair of two traits that either lists in `correlates_with`, among `ids`, in id order.
pub fn correlated_pairs(defs: &BTreeMap<String, TraitDef>, ids: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (i, a) in ids.iter().enumerate() {
        for b in &ids[i + 1..] {
            if are_correlated(defs, a, b) {
                let (x, y) = if a < b { (a, b) } else { (b, a) };
                out.push((x.clone(), y.clone()));
            }
        }
    }
    out.sort();
    out
}
/// Whether either trait lists the other in `correlates_with`.
pub fn are_correlated(defs: &BTreeMap<String, TraitDef>, a: &str, b: &str) -> bool {
    defs.get(a).is_some_and(|d| d.correlates_with.iter().any(|c| c == b))
        || defs.get(b).is_some_and(|d| d.correlates_with.iter().any(|c| c == a))
}

/// What a persona file's `traits` map may hold before the store is asked: the ceiling and the refused list, in the order
/// the refusals fix (the refusal names the reason before a store lookup could say "unknown").
pub fn check_map_shape(map: &BTreeMap<String, TraitValue>) -> Result<(), Refusal> {
    for id in map.keys() {
        check_not_refused(id)?;
        screen_id(id)?;
    }
    if map.len() > MAX_PERSONA_TRAITS {
        return Err(Refusal::TraitsTooMany { count: map.len() });
    }
    Ok(())
}

/// A panel term's condition on a trait: `trust.marketing_skepticism>=4`, `buying.role=gatekeeper`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Cond {
    pub trait_id: String,
    /// `=`, `!=`, `>=` or `<=`
    pub op: String,
    pub value: String,
}
impl Cond {
    pub fn render(&self) -> String {
        format!("{}{}{}", self.trait_id, self.op, self.value)
    }
    /// Whether a persona holding `v` for the trait passes. A persona without the trait never matches.
    pub fn matches(&self, held: Option<&TraitValue>) -> bool {
        let Some(held) = held else { return false };
        match held {
            TraitValue::Int(n) => match self.value.parse::<i64>() {
                Ok(want) => match self.op.as_str() {
                    "=" => *n == want,
                    "!=" => *n != want,
                    ">=" => *n >= want,
                    _ => *n <= want,
                },
                Err(_) => false,
            },
            TraitValue::Str(s) => match self.op.as_str() {
                "=" => *s == self.value,
                "!=" => *s != self.value,
                _ => false,
            },
            TraitValue::List(l) => match self.op.as_str() {
                "=" => l.contains(&self.value),
                "!=" => !l.contains(&self.value),
                _ => false,
            },
        }
    }
}
/// Parse the inside of `[...]`: conditions separated by commas. `term` names the whole term in a refusal.
pub fn parse_conds(term: &str, inner: &str) -> Result<Vec<Cond>, Refusal> {
    let bad = |why: &str| Refusal::InvalidFilter { term: term.to_string(), why: why.to_string() };
    if inner.trim().is_empty() {
        return Err(bad("the filter is empty"));
    }
    let mut out = Vec::new();
    for raw in inner.split(',') {
        let raw = raw.trim();
        let Some(at) = raw.find(['=', '!', '>', '<']) else { return Err(bad("a condition is <trait><op><value>")) };
        let (id, rest) = raw.split_at(at);
        let (op, value) = if let Some(v) = rest.strip_prefix("!=") {
            ("!=", v)
        } else if let Some(v) = rest.strip_prefix(">=") {
            (">=", v)
        } else if let Some(v) = rest.strip_prefix("<=") {
            ("<=", v)
        } else if let Some(v) = rest.strip_prefix('=') {
            ("=", v)
        } else {
            return Err(bad("the operators are =, !=, >= and <="));
        };
        if id.is_empty() || value.is_empty() || value.contains(|c: char| c.is_whitespace() || matches!(c, '=' | '!' | '<' | '>')) {
            return Err(bad("a condition is <trait><op><value>"));
        }
        out.push(Cond { trait_id: id.to_string(), op: op.to_string(), value: value.to_string() });
    }
    out.sort();
    out.dedup();
    Ok(out)
}
/// Check every condition of the terms against the traits the store holds.
pub fn check_conds(conds: &[Cond], term: &str, defs: &BTreeMap<String, TraitDef>) -> Result<(), Refusal> {
    for c in conds {
        let Some(def) = defs.get(&c.trait_id) else {
            check_not_refused(&c.trait_id)?;
            return Err(Refusal::UnknownTrait { id: c.trait_id.clone() });
        };
        match def.shape.as_str() {
            "band" => match c.value.parse::<i64>() {
                Ok(n) if (1..=5).contains(&n) => {}
                _ => return Err(Refusal::TraitValueOffScale { id: c.trait_id.clone(), value: c.value.clone() }),
            },
            "category" => {
                if matches!(c.op.as_str(), ">=" | "<=") {
                    return Err(Refusal::InvalidFilter {
                        term: term.to_string(),
                        why: format!("{} is a category: use = or !=, not {}", c.trait_id, c.op),
                    });
                }
                if !def.options.iter().any(|o| o.id == c.value) {
                    return Err(Refusal::TraitOptionUnknown { id: c.trait_id.clone(), value: c.value.clone() });
                }
            }
            _ => {
                return Err(Refusal::InvalidFilter {
                    term: term.to_string(),
                    why: format!("{} holds words; a filter reads a band or a category", c.trait_id),
                })
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refused_ids_match_the_last_segment_or_the_whole() {
        assert_eq!(refused_why("gender"), None, "a sensitive id is labelled, not refused (D39)");
        assert_eq!(refused_why("culture.beauty"), Some(ATTRIBUTE));
        assert_eq!(refused_why("x.mbti"), Some(TYPOLOGY));
        assert_eq!(refused_why("trust.propensity"), None);
    }
    #[test]
    fn a_trait_value_reads_an_integer_a_string_or_a_list() {
        let m: BTreeMap<String, TraitValue> =
            serde_json::from_str(r#"{"a.b": 4, "c.d": "x", "e.f": ["x", "y"]}"#).unwrap();
        assert_eq!(m["a.b"], TraitValue::Int(4));
        assert_eq!(m["c.d"], TraitValue::Str("x".into()));
        assert_eq!(m["e.f"], TraitValue::List(vec!["x".into(), "y".into()]));
        assert!(serde_json::from_str::<BTreeMap<String, TraitValue>>(r#"{"a.b": 1.5}"#).is_err());
        assert!(serde_json::from_str::<BTreeMap<String, TraitValue>>(r#"{"a.b": true}"#).is_err());
    }
    #[test]
    fn conditions_parse_sorted_and_match() {
        let c = parse_conds("t", "b.c>=4, a.b=2").unwrap();
        assert_eq!(c.iter().map(Cond::render).collect::<Vec<_>>(), vec!["a.b=2", "b.c>=4"]);
        assert!(c[1].matches(Some(&TraitValue::Int(5))));
        assert!(!c[1].matches(Some(&TraitValue::Int(3))));
        assert!(!c[1].matches(None), "a persona without the trait never matches");
        assert!(parse_conds("t", "").is_err());
        assert!(parse_conds("t", "a.b").is_err());
    }
}
