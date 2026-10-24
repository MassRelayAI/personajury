//! The v0.6-B fix round (review `v06b-review.md`, the maintainer's rulings, then D39 "allow, label, record"): every text of a
//! trait is read on every entry path and a sensitive attribute is labelled, never refused (C1, I1); the stem table with
//! normalisation (I2); the quote floor (I3); the three-juror rule and the initial-batch filter held by tests (I4); and
//! `label_echo` on rendered traits only (M1).
#[path = "judging_support/mod.rs"]
mod fx;
#[path = "traits_support/mod.rs"]
mod support;
use personajury::domain::traits::{sensitive_in, SENSITIVE, SENSITIVE_STEMS};
use personajury::domain::Stake;
use serde_json::{json, Value};
use std::path::Path;
use support::{code, jg, ok};

const PLAIN: &str = "read the page from the top and stop at the first thing that is not clear to you.";
const BY_GENDER: &str = "read the page as a woman reads it, by gender and religion and race first of all.";

/// A category trait of the family `project.mat`; each field can be swapped for a probe.
fn category(id: &str, title: &str, opt_id: &str, opt_title: &str, opt_sentence: &str, caveats: &str) -> String {
    format!(
        "id = \"{id}\"\nversion = 1\nfamily = \"project.mat\"\ntitle = \"{title}\"\nshape = \"category\"\nmoves = [\"buy\"]\n\
evidence = \"project\"\nsource = \"material:abc\"\nitems_copied = false\ncaveats = \"{caveats}\"\n\n\
[[option]]\nid = \"{opt_id}\"\ntitle = \"{opt_title}\"\nsentence = \"{opt_sentence}\"\n\n\
[[option]]\nid = \"plain\"\ntitle = \"Plain\"\nsentence = \"{PLAIN}\"\n"
    )
}
fn band(id: &str, family: &str, title: &str, anchor3: &str) -> String {
    format!(
        "id = \"{id}\"\nversion = 1\nfamily = \"{family}\"\ntitle = \"{title}\"\nshape = \"band\"\nscale = [1, 5]\nmoves = [\"buy\"]\n\
evidence = \"project\"\nsource = \"own words\"\nitems_copied = false\n\n[anchors]\n\"1\" = \"{PLAIN}\"\n\"3\" = \"{anchor3}\"\n\"5\" = \"{PLAIN}\"\n"
    )
}
fn reader_kind() -> String {
    category("mat.reader_kind", "Which kind of reader", "woman", "Woman", BY_GENDER, "none")
}
fn add(w: &Path, body: &str, from: Option<&str>) -> Value {
    let f = w.join("t.toml");
    std::fs::write(&f, body).unwrap();
    let mut a = vec!["trait", "add", "--file", f.to_str().unwrap()];
    if let Some(h) = from {
        a.extend(["--generated-from", h]);
    }
    jg(w, &a)
}
fn material(w: &Path, name: &str) -> Value {
    let f = w.join(name);
    std::fs::write(&f, "Our readers are procurement leads and engineers.\n").unwrap();
    jg(w, &["trait", "generate", "--from", f.to_str().unwrap()])
}

// ---- C1 (as D39): every text of a trait is read and labelled, on every path; nothing is refused for it ----

#[test]
fn c1_options_anchors_and_caveats_are_read_and_labelled_on_trait_add_and_generated_from() {
    let ws = support::installed();
    let w = ws.path();
    let h = ok(material(w, "readers.md"))["material_hash"].as_str().unwrap().to_string();
    let probes = [
        ("an option id", category("mat.k1", "Which kind of reader", "woman_gender", "Careful", PLAIN, "none"), json!(["gender"])),
        ("an option title", category("mat.k2", "Which kind of reader", "careful", "By religion", PLAIN, "none"), json!(["religion"])),
        ("an option sentence", category("mat.k3", "Which kind of reader", "careful", "Careful", BY_GENDER, "none"), json!(["gender", "ethnicity", "religion"])),
        ("the caveats", category("mat.k4", "Which kind of reader", "careful", "Careful", PLAIN, "Splits readers by age."), json!(["age"])),
        ("an anchor", band("mat.k5", "project.mat", "Reads the terms", "read the terms as your religion and your ethnic group would read them."), json!(["ethnicity", "religion"])),
        ("the title", band("mat.k6", "project.mat", "Gen\u{200B}der first", PLAIN), json!(["gender"])),
        ("the id", band("mat.gen_der", "project.mat", "A plain title", PLAIN), json!(["gender"])),
        ("a demographic family", band("demo.age_band", "project.demo", "Age band", PLAIN), json!(["age"])),
    ];
    for (what, body, want) in &probes {
        let a = ok(add(w, body, None));
        assert_eq!(&a["sensitive"], want, "trait add, {what}");
        let b = ok(add(w, body, Some(&h)));
        assert_eq!(&b["sensitive"], want, "trait add --generated-from, {what}");
        assert_eq!(b["origin"], "project");
    }
    let plain = category("mat.plainer", "Which kind of reader", "careful", "Careful", PLAIN, "none");
    assert_eq!(ok(add(w, &plain, Some(&h)))["sensitive"], json!([]), "a trait without the words has no label");
    // what still refuses is the refused list and the country dimension, for another reason than a sensitive word
    assert_eq!(code(&add(w, &band("culture.power_distance", "project.culture", "Power distance", PLAIN), None)), "refused_trait_id");
}

#[test]
fn c1_trait_generate_labels_the_material_name_after_normalising() {
    let ws = support::installed();
    for (name, want) in [
        ("gen_der.md", json!(["gender"])),
        ("transgender-survey.md", json!(["gender"])),
        ("nonbinary.md", json!(["gender"])),
        ("senior-citizens.md", json!(["age"])),
        ("procurement.md", json!([])),
    ] {
        assert_eq!(ok(material(ws.path(), name))["sensitive"], want, "{name}");
    }
}

#[test]
fn c1_persona_add_suggest_check_and_human_traits_accept_and_label() {
    let ws = fx::Ws::new();
    ws.site();
    let w = ws.root();
    ok(jg(w, &["trait", "install"]));
    ok(add(w, &reader_kind(), None));
    let all = json!(["gender", "ethnicity", "religion"]);
    // persona add
    let a = ok(support::add_persona(w, &support::persona_body("mei", &["t"], json!({"mat.reader_kind": "woman"}))));
    assert_eq!(a["sensitive"], all, "persona add");
    // trait suggest --check
    prose_persona(w);
    let s = json!({"persona": "mei", "traits": {"mat.reader_kind": "woman"}, "evidence": {"mat.reader_kind": "who has little time."}});
    let f = w.join("s.json");
    std::fs::write(&f, s.to_string()).unwrap();
    assert_eq!(ok(jg(w, &["trait", "suggest", "--check", f.to_str().unwrap()]))["sensitive"], all, "suggest --check");
    // verdict --by-human --traits
    let run = ws.run("critical", &[("mei", Stake::Primary)], false);
    let t = w.join("ht.json");
    std::fs::write(&t, json!({"mat.reader_kind": "woman"}).to_string()).unwrap();
    let v = crit("mei", 3, 0, "seen");
    let r = ws.cli(&["verdict", "--run", &run.to_string(), "--json", &v.to_string(), "--by-human", "Pat Doe, nurse", "--traits", t.to_str().unwrap()]);
    assert_eq!(ok(r)["sensitive"], all, "human --traits");
}

#[test]
fn c1_a_free_text_value_and_the_prose_are_read_after_normalising() {
    let ws = support::installed();
    for (i, (text, want)) in [
        ("Senior citizen care", json!(["age"])),
        ("Care for the s\u{0435}nior citizen", json!(["age"])),
        ("Gen\u{200B}der studies", json!(["gender"])),
        ("Schwangerschaft care", json!(["pregnancy"])),
        ("compare two plans for my team", json!([])),
    ]
    .into_iter()
    .enumerate()
    {
        let d = ok(support::add_persona(ws.path(), &support::persona_body(&format!("p{i}"), &["t"], json!({"situation.job": text}))));
        assert_eq!(d["sensitive"], want, "{text}");
    }
    let mut b = support::persona_body("q", &["t"], Value::Null);
    b["context"] = json!("A pensioner who reads the page on a phone.");
    assert_eq!(ok(support::add_persona(ws.path(), &b))["sensitive"], json!(["age"]), "the prose is read too");
    // the self-description and the digit rules of a free-text value still refuse: they say nothing about a sensitive word
    let v = support::add_persona(ws.path(), &support::persona_body("r", &["t"], json!({"situation.job": "You are a buyer"})));
    assert_eq!(code(&v), "trait_text_refused");
}

#[test]
fn c1_the_shipped_catalog_is_unlabelled_unless_it_truly_names_one() {
    let ws = support::installed();
    let w = ws.path();
    for (path, text) in personajury::domain::catalog::CATALOG.iter().filter(|(p, _)| p.starts_with("traits/")) {
        let want = if *path == "traits/risk/health_safety.toml" { json!(["health"]) } else { json!([]) };
        let f = w.join("c.toml");
        std::fs::write(&f, text).unwrap();
        assert_eq!(ok(jg(w, &["trait", "add", "--file", f.to_str().unwrap()]))["sensitive"], want, "{path}");
    }
    // an edited text of a shipped trait is read again
    let text = personajury::domain::catalog::shipped_text("trust.marketing_skepticism").unwrap();
    let edited = text.replace("Disbelieves advertising and claims by default", "Disbelieves advertising by gender");
    assert_eq!(ok(add(w, &edited, None))["sensitive"], json!(["gender"]));
}

#[test]
fn c1_a_run_records_uses_sensitive_and_matrix_and_summary_show_it() {
    let ws = fx::Ws::new();
    ws.site();
    let w = ws.root();
    ok(jg(w, &["trait", "install"]));
    let mut b = support::persona_body("ana", &["builder"], json!({"situation.job": "Senior citizen care"}));
    b["context"] = json!("Reads the page at a Catholic school.");
    ok(support::add_persona(w, &b));
    support::add_ok(w, "bob", &["builder"], Value::Null);
    ok(jg(w, &["panel", "add", "p", "--query", "2 builder"]));
    let brief = ok(jg(w, &["brief", "--subject", "site", "--panel", "p", "--lens", "critical", "--stake", "ana=primary,bob=primary", "--spend", "2", "--model", "m"]));
    assert_eq!(brief["uses_sensitive"], json!(["age", "religion"]));
    let run = brief["run"].as_i64().unwrap();
    for p in ["ana", "bob"] {
        ws.ingest(run, &crit(p, 3, 0, "seen"));
    }
    let m = ok(jg(w, &["matrix", &run.to_string()]));
    assert_eq!(m["uses_sensitive"], json!(["age", "religion"]));
    let line = "simulated demographics: age, religion \u{2014} personas are simulations, not real-world data";
    let s = ok(jg(w, &["summary", &run.to_string()]));
    assert!(s["text"].as_str().unwrap().replace('\n', " ").contains(line), "the text wraps at its width: {}", s["text"]);
    let (md, html) = (w.join("s.md"), w.join("s.html"));
    ok(jg(w, &["summary", &run.to_string(), "--md", md.to_str().unwrap(), "--html", html.to_str().unwrap()]));
    assert!(std::fs::read_to_string(&md).unwrap().contains(line));
    assert!(std::fs::read_to_string(&html).unwrap().contains(line));
}

#[test]
fn c1_a_run_of_unlabelled_personas_records_nothing_and_summary_adds_no_line() {
    let ws = fx::Ws::new();
    ws.site();
    let w = ws.root();
    ok(jg(w, &["trait", "install"]));
    support::add_ok(w, "ana", &["builder"], Value::Null);
    ok(jg(w, &["panel", "add", "p", "--query", "1 builder"]));
    let brief = ok(jg(w, &["brief", "--subject", "site", "--panel", "p", "--lens", "critical", "--stake", "ana=primary", "--spend", "1", "--model", "m"]));
    assert_eq!(brief["uses_sensitive"], json!([]));
    let run = brief["run"].as_i64().unwrap();
    ws.ingest(run, &crit("ana", 3, 0, "seen"));
    assert_eq!(ok(jg(w, &["matrix", &run.to_string()]))["uses_sensitive"], json!([]));
    assert!(!ok(jg(w, &["summary", &run.to_string()]))["text"].as_str().unwrap().contains("simulated demographics"));
}

// ---- I1 (as D39): no exemption is needed; disability and health are labelled where the stems hit ----

#[test]
fn i1_a_user_added_access_trait_naming_a_condition_is_accepted_and_labelled() {
    let ws = support::installed();
    let w = ws.path();
    let anchor = "use a screen reader for every page and need real headings, labels and text.";
    for family in ["access", "project.access"] {
        let v = ok(add(w, &band("access.disability_status", family, "Assistive status", anchor), None));
        assert_eq!(v["sensitive"], json!(["disability"]), "{family}");
    }
    let h = ok(material(w, "access.md"))["material_hash"].as_str().unwrap().to_string();
    let v = ok(add(w, &band("access.health_status", "project.access", "Assistive status", anchor), Some(&h)));
    assert_eq!(v["sensitive"], json!(["health"]));
    // the shipped access.need names no condition: its needs say what the page must do
    let d = ok(support::add_persona(w, &support::persona_body("q", &["t"], json!({"access.need": ["screen_reader", "colour_vision"]}))));
    assert_eq!(d["sensitive"], json!([]));
}

// ---- I2: the stem table, the normalisation, the two checks ----

/// One probe per entry of the table, written out here so that an entry dropped from the table goes red.
const PROBES: [(&str, &str); 64] = [
    ("age", "Age in years"),
    ("gender", "Agender readers"),
    ("sex", "Sex of the reader"),
    ("sexual", "Bisexual readers"),
    ("orientation", "Orientation of the reader"),
    ("race", "Race of the reader"),
    ("racial", "Biracial readers"),
    ("ethnic", "Multiethnic readers"),
    ("religion", "Religion first"),
    ("religio", "Religiosity"),
    ("faith", "Unfaithful readers"),
    ("caste", "Caste of the reader"),
    ("nationality", "Nationality first"),
    ("national", "National origin"),
    ("citizenship", "Dual citizenship"),
    ("country", "Country of birth"),
    ("pregnan", "Pregnant readers"),
    ("marital", "Premarital status"),
    ("disab", "Nondisabled readers"),
    ("politic", "Apolitical readers"),
    ("union", "Union member"),
    ("health", "Unhealthy readers"),
    ("genetic", "Epigenetic risk"),
    ("skin", "Skin tone"),
    ("color", "Colored readers"),
    ("colour", "Colouring"),
    ("tribe", "Tribe of the reader"),
    ("tribal", "Intertribal readers"),
    ("trans", "Trans readers"),
    ("transgender", "Transgender readers"),
    ("nonbinary", "Nonbinary readers"),
    ("elderly", "Elderly readers"),
    ("senior citizen", "Senior citizen discount"),
    ("pensioner", "Pensioners"),
    ("muslim", "Muslim readers"),
    ("christian", "Christian readers"),
    ("jewish", "Jewish readers"),
    ("hindu", "Hindu readers"),
    ("buddhist", "Buddhist readers"),
    ("sikh", "Sikh readers"),
    ("islam", "Islamic readers"),
    ("catholic", "Catholic readers"),
    ("atheis", "Atheist readers"),
    ("etni", "Etnia del lector"),
    ("\u{e9}tni", "\u{c9}tnico"),
    ("ethnie", "Origine ethnie"),
    ("raza", "Raza del lector"),
    ("rasse", "Rasse des Lesers"),
    ("religi", "Nonreligious readers"),
    ("nacional", "Nacionalidad del lector"),
    ("nationalit", "Nationalit\u{e4}t des Lesers"),
    ("g\u{e9}nero", "G\u{e9}nero del lector"),
    ("genero", "Genero del lector"),
    ("geschlecht", "Geschlecht des Lesers"),
    ("edad", "Edad del lector"),
    ("alter", "Alter in Jahren"),
    ("schwanger", "Schwangere Leserinnen"),
    ("behinder", "Behinderte Leser"),
    ("handicap", "Handicapped readers"),
    ("female", "Female readers"),
    ("male", "Male readers"),
    ("woman", "Woman readers"),
    ("women", "Women readers"),
    ("immigra", "Immigrant readers"),
];

#[test]
fn i2_every_entry_of_the_stem_table_has_a_probe_and_labels_it_with_its_category() {
    let probed: Vec<&str> = PROBES.iter().map(|(s, _)| *s).collect();
    for (s, cat, _) in &SENSITIVE_STEMS {
        assert!(probed.contains(s), "the table holds {s:?} and no probe tests it");
        assert!(SENSITIVE.contains(cat), "{s}: {cat} is not a category");
    }
    for (stem, text) in PROBES {
        let (_, cat, _) = SENSITIVE_STEMS.iter().find(|(s, ..)| *s == stem).unwrap_or_else(|| panic!("the table lacks {stem:?}"));
        assert!(sensitive_in(text).contains(cat), "{stem}: {text:?} is not labelled {cat}: {:?}", sensitive_in(text));
    }
}

#[test]
fn i2_the_categories_are_the_twelve_and_a_text_lists_each_once_in_order() {
    assert_eq!(
        SENSITIVE,
        ["age", "gender", "ethnicity", "religion", "nationality", "health", "disability", "sexuality", "politics", "marital", "pregnancy", "genetics"]
    );
    assert_eq!(sensitive_in("Religion, gender, race and the gender gap"), vec!["gender", "ethnicity", "religion"]);
}

#[test]
fn i2_short_and_ambiguous_stems_match_whole_words_and_prose_stays_unlabelled() {
    for text in [
        "An alternative reading",
        "Transform the page",
        "A transaction log",
        "Translates the terms",
        "Alters the plan",
        "A generous offer",
        "The agent and the agenda of the agency",
        "Reads every page of the message",
        "International buyers",
        "A reunion of the team",
        "Decides within due time",
        "As a general rule, a gentle tone",
        "Would be hindered by the form",
        "Turns on this lamp",
        "Get nine quotes",
        "A maleficent design",
        "The senior engineer",
    ] {
        assert_eq!(sensitive_in(text), Vec::<&str>::new(), "{text:?} was labelled");
    }
}

#[test]
fn i2_look_alikes_invisible_marks_and_split_words_are_normalised_first() {
    for text in [
        "G\u{0435}nder",        // Cyrillic e
        "Gen\u{200B}der",       // zero-width space
        "Gen\u{00AD}der",       // soft hyphen
        "Ge\u{0301}nder",       // a combining accent
        "\u{FF27}\u{FF45}nder", // fullwidth G and e
        "\u{1D420}ender",       // mathematical bold g
        "\u{0393}\u{0435}nder", // a Greek capital gamma is no look-alike of G
        "gen_der",
        "gen-der",
        "gen.der",
        "Gen der",
        "non-binary",
        "trans_gender",
        "mat.gen_der",
    ] {
        let hit = sensitive_in(text);
        if text.starts_with('\u{0393}') {
            assert_eq!(hit, Vec::<&str>::new(), "Gamma is not a look-alike of G");
        } else {
            assert_eq!(hit, vec!["gender"], "{text:?}");
        }
    }
    assert_eq!(sensitive_in("R\u{03B5}ligion"), vec!["religion"], "Greek epsilon");
    assert_eq!(sensitive_in("\u{0391}ge"), vec!["age"], "Greek capital alpha");
    assert_eq!(sensitive_in("dis ability"), vec!["disability"]);
}

// ---- I3: the quote floor and one field ----

fn prose_persona(w: &Path) {
    let mut b = support::persona_body("mei", &["builder"], Value::Null);
    b["role"] = json!("A designer who has little time.");
    b["context"] = json!("She opens the page between two meetings.");
    b["fears"] = json!("Paying for seats nobody uses.");
    ok(support::add_persona(w, &b));
}
fn check(w: &Path, quote: &str) -> Value {
    let s = json!({"persona": "mei", "traits": {"situation.time_pressure": 5}, "evidence": {"situation.time_pressure": quote}});
    let f = w.join("s.json");
    std::fs::write(&f, s.to_string()).unwrap();
    jg(w, &["trait", "suggest", "--check", f.to_str().unwrap()])
}

#[test]
fn i3_a_quote_is_three_words_and_twelve_characters_inside_one_field() {
    let ws = support::installed();
    let w = ws.path();
    prose_persona(w);
    assert_eq!(code(&check(w, "e")), "quote_too_short");
    assert_eq!(code(&check(w, "a")), "quote_too_short");
    assert_eq!(code(&check(w, "has little")), "quote_too_short", "two words");
    assert_eq!(code(&check(w, "who has a lot")), "quote_not_found", "four words but not in the prose");
    assert_eq!(code(&check(w, "She opens")), "quote_too_short");
    assert_eq!(code(&check(w, "has  little")), "quote_too_short", "whitespace folded first");
    assert_eq!(code(&check(w, "a b c d")), "quote_too_short", "four words, seven characters");
    let e = check(w, "little time. She opens");
    assert_eq!(code(&e), "quote_not_found", "a quote across two fields");
    assert_eq!(e["data"]["id"], "situation.time_pressure");
    ok(check(w, "who has little time."));
    ok(check(w, "between two\n meetings"));
}

// ---- I4: the three-juror rule and the initial batch ----

fn crit(persona: &str, s: i64, blockers: usize, kind: &str) -> Value {
    let lines: Vec<Value> = ["works", "blocks", "missing"]
        .iter()
        .map(|d| {
            let b: Vec<String> = (0..blockers).map(|i| format!("{d} problem {i}")).collect();
            let mut l = json!({"dimension": d, "score": s, "anchor_met": if s >= 3 { 3 } else { 1 }, "evidence": kind, "blockers": b});
            if kind == "seen" {
                l["citations"] = json!(["page.json#n1"]);
            }
            l
        })
        .collect();
    json!({"persona": persona, "lines": lines})
}
fn comp(persona: &str, choice: &str, s: i64, price_about: Option<&str>) -> Value {
    let lines: Vec<Value> = ["clarity", "price", "trust"]
        .iter()
        .map(|d| {
            let mut l = json!({"dimension": d, "score": s, "anchor_met": if s >= 3 { 3 } else { 1 }, "evidence": "seen",
                               "citations": ["page.json#n1", "cmp:rival/rival.json#r1"]});
            if let (true, Some(a)) = (*d == "price", price_about) {
                l["about"] = json!(a);
            }
            l
        })
        .collect();
    json!({"persona": persona, "lines": lines, "pick": {"choice": choice, "reason": "because"}})
}
fn world(people: &[&str]) -> fx::Ws {
    let ws = fx::Ws::new();
    ws.site();
    ws.rival();
    ok(jg(ws.root(), &["trait", "install"]));
    for p in people {
        support::add_ok(ws.root(), p, &["t"], json!({"adoption.ease_demanded": 5}));
    }
    ws
}
fn observed(ws: &fx::Ws, who: &str) -> Value {
    ok(jg(ws.root(), &["trait", "observed", "--persona", who]))
}

#[test]
fn i4_a_stratum_of_two_adds_no_severity_blockers_bottom_or_majority() {
    let ws = world(&["ana", "cat"]);
    let two = [("ana", Stake::Primary), ("cat", Stake::Primary)];
    let run = ws.run("critical", &two, false);
    ws.ingest(run, &crit("ana", 1, 2, "seen"));
    ws.ingest(run, &crit("cat", 5, 0, "seen"));
    let ana = observed(&ws, "ana");
    assert_eq!(ana["signals"]["severity"], json!([]), "{ana}");
    assert_eq!(ana["signals"]["blockers"]["runs"], json!([]), "two jurors are not a stratum to compare with");
    assert_eq!(ana["signals"]["bottom"], json!({"more": 0, "same": 0, "fewer": 0}));
    let run = ws.run("compare", &two, true);
    ws.ingest(run, &comp("ana", "page.json", 3, None));
    ws.ingest(run, &comp("cat", "page.json", 3, None));
    let pick = &observed(&ws, "ana")["signals"]["pick"];
    assert_eq!(pick, &json!({"subject": 0, "comparator": 0, "attachment": 1, "with_majority": 0, "against_majority": 0}));
}

#[test]
fn i4_a_cell_of_two_scores_in_a_stratum_of_three_adds_no_severity() {
    let ws = world(&["ana", "cat", "dan"]);
    let three = [("ana", Stake::Primary), ("cat", Stake::Primary), ("dan", Stake::Primary)];
    let run = ws.run("compare", &three, true);
    ws.ingest(run, &comp("ana", "page.json", 2, None));
    ws.ingest(run, &comp("cat", "page.json", 3, None));
    ws.ingest(run, &comp("dan", "page.json", 3, Some("cmp:rival")));
    let ana = observed(&ws, "ana");
    let sev = ana["signals"]["severity"].as_array().unwrap();
    let dims: Vec<&str> = sev.iter().map(|c| c["dimension"].as_str().unwrap()).collect();
    assert_eq!(dims, vec!["clarity", "trust"], "price about the subject has two scores only: {ana}");
    assert_eq!(ana["signals"]["pick"]["with_majority"], 1, "three picks make a majority");
}

const LOW: &str = "[[rule]]\nclass = \"public-surface\"\npaths = [\"sources/site/**\"]\nquorum = \"low\"\npanel = \"p\"\nlenses = [\"critical\"]\n";

#[test]
fn i4_only_the_latest_initial_batch_counts() {
    let ws = world(&["ana", "cat", "dan"]);
    let three = [("ana", Stake::Primary), ("cat", Stake::Primary), ("dan", Stake::Primary)];
    let run = ws.run("critical", &three, false);
    ws.store.conn().execute("UPDATE run SET rule_class = 'public-surface', quorum = 'low' WHERE id = ?1", [run]).unwrap();
    std::fs::write(ws.root().join("personajury.toml"), LOW).unwrap();
    // a superseded batch: ana first says told and scores low, then sees and scores at the median
    ws.ingest(run, &crit("ana", 1, 1, "told"));
    ws.ingest(run, &crit("ana", 3, 0, "seen"));
    ws.ingest(run, &crit("cat", 3, 0, "seen"));
    ws.ingest(run, &crit("dan", 3, 0, "seen"));
    // a deliberated batch, recalled and low, after the round
    assert_eq!(ws.cli(&["deliberate", &run.to_string()])["ok"], true);
    let mut d = crit("ana", 1, 1, "recalled");
    d["phase"] = json!("deliberated");
    let r = ws.cli(&["verdict", "--run", &run.to_string(), "--json", &d.to_string(), "--deliberated"]);
    assert_eq!(r["ok"], true, "{r}");
    let ana = observed(&ws, "ana");
    assert_eq!(ana["signals"]["evidence"], json!({"seen": 3, "told": 0, "recalled": 0}), "{ana}");
    assert!(ana["signals"]["severity"].as_array().unwrap().iter().all(|c| c["at"] == 1 && c["below"] == 0), "{ana}");
}

// ---- M1: label_echo on the traits the brief rendered ----

fn echo_world() -> fx::Ws {
    let ws = fx::Ws::new();
    ws.site();
    ok(jg(ws.root(), &["trait", "install"]));
    support::add_ok(ws.root(), "ana", &["t"], json!({"trust.marketing_skepticism": 5, "expertise.domain": 4}));
    ws
}
fn noted(note: &str) -> Value {
    let mut v = crit("ana", 3, 0, "seen");
    v["lines"][1]["note"] = json!(note);
    v
}

#[test]
fn m1_label_echo_never_fires_when_the_brief_rendered_no_traits() {
    let ws = echo_world();
    let run = ws.run("critical", &[("ana", Stake::Primary)], false);
    let d = ws.ingest(run, &noted("As a marketing skeptic I stop at the domain name in the footer."));
    assert!(d.get("warnings").is_none(), "{d}");
}

#[test]
fn m1_label_echo_reads_only_the_traits_the_brief_rendered() {
    let ws = echo_world();
    // a set that renders trust.marketing_skepticism and not expertise.domain
    let set = ws.root().join("s.toml");
    std::fs::write(&set, "name = \"only-trust\"\nversion = 1\ntraits = [\"trust.marketing_skepticism\"]\ncover = [\"trust.marketing_skepticism\"]\n").unwrap();
    ok(jg(ws.root(), &["trait", "set", "add", "--file", set.to_str().unwrap()]));
    let run = ws.run("critical", &[("ana", Stake::Primary)], false);
    ws.store.set_run_traits(run, Some(&json!({"mode": "set", "set": "only-trust"})), None).unwrap();
    let d = ws.ingest(run, &noted("As a marketing skeptic I stop at the domain name in the footer."));
    assert_eq!(d["warnings"], json!([{"code": "label_echo", "line": 2, "trait": "trust.marketing_skepticism"}]));
    let run = ws.run("critical", &[("ana", Stake::Primary)], false);
    ws.store.set_run_traits(run, Some(&json!({"mode": "all"})), None).unwrap();
    let d = ws.ingest(run, &noted("As a marketing skeptic I stop at the domain name in the footer."));
    assert_eq!(d["warnings"].as_array().unwrap().len(), 2, "both rendered: {d}");
}
