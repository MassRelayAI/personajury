//! D31: `persona generate --traits <set>` adds the set's traits and a Latin-square row per persona to the brief.
#[path = "traits_support/mod.rs"]
mod support;
use support::*;

fn material(ws: &std::path::Path) -> String {
    let f = ws.join("m.md");
    std::fs::write(&f, "# material\nBuilders ask about price.\n").unwrap();
    f.to_string_lossy().into_owned()
}
fn read(ws: &std::path::Path, d: &serde_json::Value) -> String {
    std::fs::read_to_string(ws.join(d["brief"].as_str().unwrap())).unwrap()
}

#[test]
fn without_traits_the_brief_is_the_old_one() {
    let ws = installed();
    let m = material(ws.path());
    let plain = ok(jg(ws.path(), &["persona", "generate", "--from", &m, "--count", "3"]));
    let t = read(ws.path(), &plain);
    assert!(!t.contains("Step 3") && !t.contains("`traits`"));
    let again = ok(jg(ws.path(), &["persona", "generate", "--from", &m, "--count", "3"]));
    assert_eq!(read(ws.path(), &again), t);
}

#[test]
fn with_a_set_the_brief_lists_the_traits_and_one_latin_square_row_per_persona() {
    let ws = installed();
    let m = material(ws.path());
    let d = ok(jg(ws.path(), &["persona", "generate", "--from", &m, "--count", "3", "--traits", "docs"]));
    let t = read(ws.path(), &d);
    assert!(t.contains("## Step 3: the traits (set `docs`, version 1)"));
    assert!(t.contains("- `traits`: an object of trait ids"));
    assert!(t.contains("- `expertise.domain` (cover)"));
    assert!(t.contains("  - 3: "), "the anchors are listed");
    assert!(t.contains("  - `screen_reader`: "), "the options are listed");
    assert!(t.contains("| persona | expertise.domain | access.need | situation.time_pressure |"));
    assert!(t.contains("| 1 | low (1 or 2) | `low_vision_zoom` | high (4 or 5) |"), "{t}");
    assert!(t.contains("| 2 | mid (3) | `colour_vision` | low (1 or 2) |"), "{t}");
    assert!(t.contains("| 3 | high (4 or 5) | `keyboard_only` | mid (3) |"), "{t}");
    assert!(!t.contains("| 4 |"));
    let v = jg(ws.path(), &["persona", "generate", "--from", &m, "--count", "3", "--traits", "no-such"]);
    assert_eq!(code(&v), "trait_set_unknown");
}
