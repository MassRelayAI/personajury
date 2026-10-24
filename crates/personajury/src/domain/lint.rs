//! Warnings about the personas a store holds and the lines a juror writes (D36). Pure. A warning is never a refusal: the
//! engine cannot read prose for meaning, so these are the few checks a count can make.
use super::model::Persona;
use super::traits::TraitDef;
use std::collections::{BTreeMap, BTreeSet};

/// The words of a text, lower-cased, split on anything that is not a letter or a digit.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_string).collect()
}
/// The first seven letters of a word: `skeptic` for `skepticism`, so "marketing skeptic" echoes "marketing skepticism".
fn stem(w: &str) -> String {
    w.chars().take(7).collect()
}
/// Whether the words of `phrase` appear in `text` next to each other, in order, each by its stem (case-insensitive).
pub fn phrase_in(text: &str, phrase: &str) -> bool {
    let (t, p) = (words(text), words(phrase));
    if p.is_empty() || t.len() < p.len() {
        return false;
    }
    let stems: Vec<String> = p.iter().map(|w| stem(w)).collect();
    (0..=t.len() - p.len()).any(|i| stems.iter().enumerate().all(|(j, s)| t[i + j].starts_with(s.as_str())))
}
/// Whether a line's `note` or a blocker names a trait the persona carries: the trait's title (contained, any case) or the
/// id's last segment with `_` as a space (by `phrase_in`). The word the juror was handed is the word it echoes.
pub fn label_echo(def: &TraitDef, texts: &[&str]) -> bool {
    let title = def.title.trim().to_lowercase();
    let last = def.id.rsplit('.').next().unwrap_or("").replace('_', " ");
    texts.iter().any(|t| (!title.is_empty() && t.to_lowercase().contains(&title)) || phrase_in(t, &last))
}

/// Pairs of personas (ids, `a < b`) with the same non-empty trait map who share a tag, or have the same tags.
pub fn near_copies(personas: &[Persona]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (i, a) in personas.iter().enumerate() {
        for b in &personas[i + 1..] {
            let (Some(ta), Some(tb)) = (&a.traits, &b.traits) else { continue };
            if ta.is_empty() || ta.iter().map(|(k, v)| (k, v.canonical())).ne(tb.iter().map(|(k, v)| (k, v.canonical()))) {
                continue;
            }
            if a.tags.iter().any(|t| b.tags.contains(t)) || a.tags == b.tags {
                let (x, y) = if a.id < b.id { (&a.id, &b.id) } else { (&b.id, &a.id) };
                out.push((x.clone(), y.clone()));
            }
        }
    }
    out.sort();
    out
}

fn key(def: Option<&TraitDef>, p: &Persona, id: &str) -> Option<String> {
    let v = p.traits.as_ref()?.get(id)?;
    let held = match def {
        Some(d) => d.bands_held(v),
        None => vec![v.show()],
    };
    (!held.is_empty()).then(|| held.join("+"))
}
/// `(trait, tag, band)` where two or more personas with the tag hold the band and at least one persona without the tag holds
/// another: the tag and the band cannot be told apart in what this store would show. Sorted; each once.
pub fn confounded(personas: &[Persona], defs: &BTreeMap<String, TraitDef>) -> Vec<(String, String, String)> {
    let ids: BTreeSet<&String> = personas.iter().filter_map(|p| p.traits.as_ref()).flat_map(|t| t.keys()).collect();
    let tags: BTreeSet<&String> = personas.iter().flat_map(|p| p.tags.iter()).collect();
    let mut out = Vec::new();
    for id in ids {
        let held: Vec<(&Persona, String)> = personas.iter().filter_map(|p| key(defs.get(id.as_str()), p, id).map(|k| (p, k))).collect();
        let bands: BTreeSet<&String> = held.iter().map(|(_, k)| k).collect();
        for tag in &tags {
            for band in &bands {
                let with = held.iter().filter(|(p, k)| p.tags.contains(tag) && k == *band).count();
                let other = held.iter().any(|(p, k)| !p.tags.contains(tag) && k != *band);
                if with >= 2 && other {
                    out.push((id.to_string(), tag.to_string(), band.to_string()));
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_phrase_echoes_by_stem_and_in_order() {
        assert!(phrase_in("As a marketing skeptic I distrust this", "marketing skepticism"));
        assert!(phrase_in("MARKETING SKEPTICISM here", "marketing skepticism"));
        assert!(!phrase_in("the marketing is skeptical", "marketing skepticism"), "not adjacent");
        assert!(!phrase_in("skepticism of marketing", "marketing skepticism"), "not in order");
        assert!(!phrase_in("nothing here", "marketing skepticism"));
        assert!(!phrase_in("anything", ""));
    }
}
