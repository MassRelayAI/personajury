//! D35: `trait generate --from` writes a brief for the agent; its example is a valid trait file; what it names is labelled (D39).
#[path = "traits_support/mod.rs"]
mod support;
use serde_json::json;
use support::*;

fn material(ws: &std::path::Path, name: &str) -> String {
    let f = ws.join(name);
    std::fs::write(&f, "# Our buyers\nOur buyers are either procurement or an engineering lead.\n").unwrap();
    f.to_string_lossy().into_owned()
}
fn brief(ws: &std::path::Path, d: &serde_json::Value) -> String {
    std::fs::read_to_string(ws.join(d["brief"].as_str().unwrap())).unwrap()
}
/// The fenced `toml` blocks of a brief.
fn blocks(t: &str) -> Vec<String> {
    t.split("```toml\n").skip(1).map(|b| b.split("```").next().unwrap().to_string()).collect()
}

#[test]
fn generate_writes_a_brief_named_by_the_material_hash_and_records_it() {
    let ws = installed();
    let m = material(ws.path(), "buyers.md");
    let d = ok(jg(ws.path(), &["trait", "generate", "--from", &m]));
    let hash = d["material_hash"].as_str().unwrap();
    assert_eq!(d["brief"], format!(".personajury/generate/trait-{hash}.md"));
    assert_eq!(d["count"], 3, "the default is three");
    let t = brief(ws.path(), &d);
    assert!(t.starts_with("personajury-trait-generate 1\n"));
    assert!(t.contains("Step 1") && t.contains("Step 2"));
    assert!(t.contains("family = \"project.buyers\""), "the slug is the material file stem: {t}");
    assert!(t.contains(&format!("source = \"material:{hash}\"")));
    assert!(t.contains("evidence = \"project\"") && t.contains("items_copied = false"));
    assert!(t.contains("You tend to"));
    let low = t.to_lowercase();
    assert!(low.contains("no typology") && low.contains("no country"));
    assert!(low.contains("labelled `sensitive`") && low.contains("never a real or named person"), "D39: labelled, composites only");
    // deterministic
    let again = ok(jg(ws.path(), &["trait", "generate", "--from", &m]));
    assert_eq!(again["brief_hash"], d["brief_hash"]);
}

#[test]
fn count_is_one_to_eight() {
    let ws = installed();
    let m = material(ws.path(), "buyers.md");
    for bad in ["0", "9"] {
        let v = jg(ws.path(), &["trait", "generate", "--from", &m, "--count", bad]);
        assert_eq!(code(&v), "invalid_arguments", "{bad}");
    }
    let d = ok(jg(ws.path(), &["trait", "generate", "--from", &m, "--count", "8"]));
    assert!(brief(ws.path(), &d).contains("8 trait files"));
}

#[test]
fn every_example_in_the_brief_is_a_valid_file_that_trait_add_takes_as_a_project_trait() {
    let ws = installed();
    let m = material(ws.path(), "buyers.md");
    let d = ok(jg(ws.path(), &["trait", "generate", "--from", &m]));
    let hash = d["material_hash"].as_str().unwrap().to_string();
    let ex = blocks(&brief(ws.path(), &d));
    assert!(ex.len() >= 2, "a band example and a category example");
    for (i, body) in ex.iter().enumerate() {
        let f = ws.path().join(format!("ex{i}.toml"));
        std::fs::write(&f, body).unwrap();
        let a = ok(jg(ws.path(), &["trait", "add", "--file", f.to_str().unwrap(), "--generated-from", &hash]));
        assert_eq!(a["origin"], "project", "{a}");
    }
    let list = ok(jg(ws.path(), &["trait", "list"]));
    let mine: Vec<_> = list["traits"].as_array().unwrap().iter().filter(|t| t["family"].as_str().unwrap().starts_with("project.")).collect();
    assert_eq!(mine.len(), 2);
    assert!(mine.iter().all(|t| t["badge"] == "P"), "a project trait shows (P): {mine:?}");
    let shipped = list["traits"].as_array().unwrap().iter().find(|t| t["id"] == "trust.marketing_skepticism").unwrap();
    assert!(shipped.get("badge").is_none());
}

#[test]
fn a_material_whose_name_names_a_sensitive_attribute_is_labelled_and_culture_is_refused() {
    let ws = installed();
    let m = material(ws.path(), "gender-survey.md");
    let v = ok(jg(ws.path(), &["trait", "generate", "--from", &m]));
    assert_eq!(v["sensitive"], json!(["gender"]), "{v}");
    let m = material(ws.path(), "culture.md");
    let v = jg(ws.path(), &["trait", "generate", "--from", &m]);
    assert_eq!(code(&v), "refused_trait_id", "{v}");
}

#[test]
fn a_generated_trait_that_names_a_sensitive_attribute_is_added_and_labelled() {
    let ws = installed();
    let m = material(ws.path(), "buyers.md");
    let d = ok(jg(ws.path(), &["trait", "generate", "--from", &m]));
    let hash = d["material_hash"].as_str().unwrap().to_string();
    let body = blocks(&brief(ws.path(), &d))[0].replace("Reads the fine print before deciding", "Decides by nationality of the vendor");
    let f = ws.path().join("bad.toml");
    std::fs::write(&f, body).unwrap();
    let v = ok(jg(ws.path(), &["trait", "add", "--file", f.to_str().unwrap(), "--generated-from", &hash]));
    assert_eq!(v["sensitive"], json!(["nationality"]), "{v}");
    assert_eq!(v["origin"], "project");
}
