//! D31: `brief --traits`, the block "How this reader tends to decide (told)", the trait set of a rule, and the run's record.
#[path = "traits_support/mod.rs"]
mod support;
use personajury::store::Store;
use serde_json::{json, Value};
use std::path::Path;
use support::*;

fn doc(ws: &Path) {
    let f = ws.join("page.md");
    std::fs::write(&f, "# page\nline two\n").unwrap();
    ok(jg(ws, &["subject", "add", "site", "--kind", "document", "--attach", f.to_str().unwrap()]));
}
fn brief(ws: &Path, extra: &[&str]) -> Value {
    let mut a = vec!["brief", "--subject", "site", "--query", "1 a=primary, 1 b=secondary", "--lens", "critical", "--model", "m"];
    a.extend(extra);
    jg(ws, &a)
}
fn text_of(ws: &Path, d: &Value, persona: &str) -> String {
    let b = d["briefs"].as_array().unwrap().iter().find(|b| b["persona"] == persona).unwrap();
    std::fs::read_to_string(ws.join(b["path"].as_str().unwrap())).unwrap()
}
/// Two personas: a (all the listed traits) and b (a few).
fn world(a_traits: Value) -> tempfile::TempDir {
    let ws = installed();
    add_ok(ws.path(), "a", &["a"], a_traits);
    add_ok(ws.path(), "b", &["b"], json!({"expertise.domain": 2}));
    doc(ws.path());
    ws
}
const BLOCK: &str = "## How this reader tends to decide (told)";

#[test]
fn mode_off_changes_no_byte() {
    let with = world(json!({"trust.marketing_skepticism": 5, "expertise.domain": 4}));
    let without = installed();
    add_ok(without.path(), "a", &["a"], Value::Null);
    add_ok(without.path(), "b", &["b"], Value::Null);
    doc(without.path());
    let x = ok(brief(with.path(), &[]));
    let y = ok(brief(without.path(), &[]));
    for p in ["a", "b"] {
        assert_eq!(text_of(with.path(), &x, p), text_of(without.path(), &y, p), "{p}");
    }
    let hashes = |d: &Value| d["briefs"].as_array().unwrap().iter().map(|b| b["hash"].clone()).collect::<Vec<_>>();
    assert_eq!(hashes(&x), hashes(&y));
    // nothing was rendered and the panel has no cover: nothing is recorded
    let store = Store::open(with.path()).unwrap();
    assert_eq!(store.run_traits(x["run"].as_i64().unwrap()).unwrap().0, None);
}

#[test]
fn all_renders_the_block_between_the_persona_and_the_stake() {
    let ws = world(json!({"trust.marketing_skepticism": 5, "expertise.domain": 4}));
    let d = ok(brief(ws.path(), &["--traits", "all"]));
    let t = text_of(ws.path(), &d, "a");
    let at = t.find(BLOCK).expect("block");
    assert!(t.find("## Persona").unwrap() < at && at < t.find("## Your stake in this run").unwrap());
    assert!(t.contains("- You tend to: assume every claim is written to sell"));
    // b holds one trait
    assert!(text_of(ws.path(), &d, "b").contains(BLOCK));
    let recorded = Store::open(ws.path()).unwrap().run_traits(d["run"].as_i64().unwrap()).unwrap().0.unwrap();
    assert_eq!(recorded["mode"], "all");
    assert_eq!(recorded["set"], Value::Null);
    assert_eq!(recorded["draw"], "seeded");
    assert_eq!(recorded["versions"]["trust.marketing_skepticism"], 1);
    assert_eq!(recorded["versions"]["expertise.domain"], 1);
}

#[test]
fn a_set_renders_only_its_traits_and_reports_what_is_missing() {
    let ws = world(json!({"trust.marketing_skepticism": 5, "expertise.domain": 4, "personality.openness": 2}));
    let d = ok(brief(ws.path(), &["--traits", "docs"]));
    let t = text_of(ws.path(), &d, "a");
    // expertise.domain is in the docs set, the other two are not
    assert!(t.contains("know this field well"));
    assert!(!t.contains("assume every claim is written to sell"));
    assert!(!t.contains("stay with what you know"));
    let inc = d["trait_set_incomplete"].as_array().unwrap();
    assert_eq!(inc.len(), 2, "{inc:?}");
    let a = inc.iter().find(|i| i["persona"] == "a").unwrap();
    assert_eq!(a["missing"].as_array().unwrap().len(), 7);
    let recorded = Store::open(ws.path()).unwrap().run_traits(d["run"].as_i64().unwrap()).unwrap().0.unwrap();
    assert_eq!((recorded["mode"].as_str(), recorded["set"].as_str()), (Some("set"), Some("docs")));
    assert_eq!(recorded["versions"].as_object().unwrap().len(), 1);
}

#[test]
fn a_set_that_does_not_exist_is_refused() {
    let ws = world(json!({"expertise.domain": 4}));
    let v = brief(ws.path(), &["--traits", "no-such-set"]);
    assert_eq!(code(&v), "trait_set_unknown");
}

#[test]
fn bands_one_to_five_render_the_right_anchor_and_prefix() {
    let ws = world(json!({"trust.marketing_skepticism": 1}));
    for (band, lead, anchor) in [
        (1, "You tend to: ", "take a claim on the page at its word"),
        (2, "You tend to lean this way: ", "take a claim on the page at its word"),
        (3, "You tend to: ", "read a claim and look once"),
        (4, "You tend to lean this way: ", "assume every claim is written to sell"),
        (5, "You tend to: ", "assume every claim is written to sell"),
    ] {
        add_ok(ws.path(), "a", &["a"], json!({"trust.marketing_skepticism": band}));
        let d = ok(brief(ws.path(), &["--traits", "all"]));
        let t = text_of(ws.path(), &d, "a");
        assert!(t.contains(&format!("- {lead}{anchor}")), "band {band}:\n{t}");
    }
}

#[test]
fn the_route_is_derived_both_ways_and_never_printed_as_a_trait() {
    let central = world(json!({"persuasion.involvement": 5, "cognition.need_for_cognition": 4}));
    let t = text_of(central.path(), &ok(brief(central.path(), &["--traits", "all"])), "a");
    assert!(t.contains("- You tend to: read the argument itself and weigh what it proves"));
    assert!(!t.contains("weigh the cues first"));
    let periph = world(json!({"persuasion.involvement": 5, "cognition.need_for_cognition": 3}));
    let t = text_of(periph.path(), &ok(brief(periph.path(), &["--traits", "all"])), "a");
    assert!(t.contains("- You tend to: weigh the cues first"));
    // the route line is the last bullet
    let last = t.lines().rfind(|l| l.starts_with("- You tend to")).unwrap();
    assert!(last.contains("weigh the cues first"), "{last}");
    // one input alone derives nothing
    let one = world(json!({"persuasion.involvement": 5}));
    let t = text_of(one.path(), &ok(brief(one.path(), &["--traits", "all"])), "a");
    assert!(!t.contains("weigh the cues first") && !t.contains("read the argument itself"));
}

#[test]
fn the_block_speaks_in_tendencies_with_no_digit_and_no_you_are() {
    let all = json!({
        "trust.marketing_skepticism": 4, "expertise.domain": 1, "choice.decision_style": "dependent",
        "access.need": ["screen_reader", "captions"], "situation.job": "compare   two plans\nfor my team",
        "persuasion.involvement": 5, "cognition.need_for_cognition": 5, "risk.financial": 2 
    });
    let ws = world(all);
    let d = ok(brief(ws.path(), &["--traits", "all"]));
    let t = text_of(ws.path(), &d, "a");
    let from = t.find(BLOCK).unwrap();
    let to = t.find("## Your stake in this run").unwrap();
    let block = &t[from..to];
    let bullets: Vec<&str> = block.lines().filter(|l| l.starts_with("- ")).collect();
    assert!(bullets.len() >= 9, "{block}");
    // a tendency, or the persona's work (a words trait, review I3): never "You are"
    for b in &bullets {
        assert!(b.starts_with("- You tend to") || b.starts_with("- Your work: "), "{b}");
    }
    assert!(!block.contains("You are"), "{block}");
    assert!(!block.chars().any(|c| c.is_ascii_digit()), "{block}");
    assert!(block.contains("- Your work: compare two plans for my team"));
    // ids and titles are never printed
    assert!(!block.contains("trust.") && !block.contains("Disbelieves"));
    // sorted by trait id: access.need first
    assert!(block.find("screen reader").unwrap() < block.find("Disbelieves").unwrap_or(usize::MAX));
}

#[test]
fn briefs_with_traits_are_deterministic() {
    let ws = world(json!({"trust.marketing_skepticism": 4, "expertise.domain": 1}));
    let x = ok(brief(ws.path(), &["--traits", "all"]));
    let y = ok(brief(ws.path(), &["--traits", "all"]));
    let hashes = |d: &Value| d["briefs"].as_array().unwrap().iter().map(|b| b["hash"].clone()).collect::<Vec<_>>();
    assert_eq!(hashes(&x), hashes(&y));
}

fn rule_ws(trait_set: Option<&str>) -> tempfile::TempDir {
    let ws = world(json!({"trust.marketing_skepticism": 5, "expertise.domain": 4}));
    ok(jg(ws.path(), &["panel", "add", "p", "--query", "1 a=primary, 1 b=secondary"]));
    let set = trait_set.map(|s| format!("trait_set = \"{s}\"\n")).unwrap_or_default();
    std::fs::write(
        ws.path().join("personajury.toml"),
        format!("[[rule]]\nclass = \"docs\"\npaths = [\"page.md\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n{set}"),
    )
    .unwrap();
    ws
}
#[test]
fn a_rules_trait_set_is_the_default_and_a_different_flag_is_an_override() {
    let ws = rule_ws(Some("docs"));
    let d = ok(jg(ws.path(), &["brief", "--subject", "site", "--rule", "docs", "--model", "m"]));
    assert!(text_of(ws.path(), &d["runs"][0], "a").contains(BLOCK));
    // the same set named again departs from nothing
    ok(jg(ws.path(), &["brief", "--subject", "site", "--rule", "docs", "--model", "m", "--traits", "docs"]));
    let v = jg(ws.path(), &["brief", "--subject", "site", "--rule", "docs", "--model", "m", "--traits", "all"]);
    assert_eq!(code(&v), "override_without_reason");
    let d = ok(jg(ws.path(), &["brief", "--subject", "site", "--rule", "docs", "--model", "m", "--traits", "all", "--reason", "every trait"]));
    assert_eq!(d["override"]["fields"]["traits"], json!({"rule": "docs", "given": "all"}));
}
#[test]
fn a_rule_without_a_trait_set_renders_none_and_a_flag_is_an_override() {
    let ws = rule_ws(None);
    let d = ok(jg(ws.path(), &["brief", "--subject", "site", "--rule", "docs", "--model", "m"]));
    assert!(!text_of(ws.path(), &d["runs"][0], "a").contains(BLOCK));
    let v = jg(ws.path(), &["brief", "--subject", "site", "--rule", "docs", "--model", "m", "--traits", "all"]);
    assert_eq!(code(&v), "override_without_reason");
}

#[test]
fn apply_check_notes_an_incomplete_set_without_changing_the_answer() {
    let ws = world(json!({"expertise.domain": 4}));
    let d = ok(brief(ws.path(), &["--traits", "docs"]));
    let run = d["run"].to_string();
    let a = ok(jg(ws.path(), &["apply-check", &run]));
    let r = a["reasons"].as_array().unwrap().iter().find(|r| r["code"] == "trait_set_incomplete").expect("the reason");
    assert_eq!(r["effect"], "info");
    assert_eq!(r["seats"].as_array().unwrap().len(), 2);
    // a run that rendered nothing says nothing of the kind
    let d = ok(brief(ws.path(), &[]));
    let a = ok(jg(ws.path(), &["apply-check", &d["run"].to_string()]));
    assert!(a["reasons"].as_array().unwrap().iter().all(|r| r["code"] != "trait_set_incomplete"));
}

#[test]
fn set_mode_renders_the_cover_and_the_set_traits_that_say_something() {
    // the docs set: cover expertise.domain, access.need, situation.time_pressure; conscientiousness and need for cognition
    // are set traits off the cover
    let ws = world(json!({"expertise.domain": 3, "personality.conscientiousness": 3, "cognition.need_for_cognition": 5, "risk.financial": 5}));
    let t = text_of(ws.path(), &ok(brief(ws.path(), &["--traits", "docs"])), "a");
    assert!(t.contains("know the basics of this field"), "a cover trait renders even at the middle: {t}");
    assert!(t.contains("like to work an argument through"), "a set trait off the middle renders: {t}");
    assert!(!t.contains("follow the steps in order"), "a set trait at the middle, off the cover, is left out: {t}");
    assert!(!t.contains("Willing to spend") && !t.contains("money"), "a trait outside the set is left out: {t}");
    // all mode still renders every trait the persona holds
    let t = text_of(ws.path(), &ok(brief(ws.path(), &["--traits", "all"])), "a");
    assert!(t.contains("follow the steps in order"), "{t}");
}

#[test]
fn the_closing_line_is_short() {
    let ws = world(json!({"expertise.domain": 3}));
    let t = text_of(ws.path(), &ok(brief(ws.path(), &["--traits", "all"])), "a");
    let line = "Told, not seen: a line leaning on one is `told`; a line about the subject still cites it.";
    assert!(t.contains(&format!("\n{line}\n")), "{t}");
    assert!(line.len() <= 90);
}
