//! brief: deterministic, hashed, one file per persona × lens; creates the run (plan Task 4)
use super::{AppError, Result};
use crate::domain::hash::{self, sha256_hex};
use crate::domain::traits::{self, TraitDef, TraitSetDef};
use crate::domain::*;
use crate::store::{now_iso, Store};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// Per-attachment cap of a full brief (D3).
const CAP: usize = 65_536;

/// What `personajury brief` was asked for. A caller that names a `rule` takes its panel, lenses, spend and detail as
/// defaults and may override any of them, with a `reason` (D10).
#[derive(Debug, Clone, Default)]
pub struct BriefArgs {
    pub subject: String,
    /// a saved panel; or `query`, an ad-hoc panel that is not saved; or neither, when a rule names one
    pub panel: Option<String>,
    pub query: Option<String>,
    /// the lenses, one run each; empty = the rule's
    pub lenses: Vec<String>,
    pub spend: Option<usize>,
    /// explicit stakes: they win over the panel's own term stakes
    pub stakes: Vec<(String, Stake)>,
    pub model: String,
    pub comparators: Vec<String>,
    pub detail: Option<Detail>,
    pub rule: Option<String>,
    /// hex seed of the draw, instead of the one derived from the subject's name
    pub draw: Option<String>,
    pub allow_large: bool,
    pub reason: Option<String>,
    pub by: Option<String>,
    /// seats per wave in the `waves` answer (D12); default 30
    pub wave: Option<usize>,
    /// how much this run may spend, over the toml's (D17); recorded on the run
    pub effort: Option<Effort>,
    /// whether the jurors are separate contexts (D18)
    pub independence: Independence,
    /// `off`, `all` or the name of a trait set: which of the persona's traits the briefs render (D31)
    pub traits: Option<String>,
    /// `auto`: seat every persona that serves the subject's audience, plus one control (see `seats_auto`)
    pub seats: Option<String>,
    /// `subject` or `attachments` (D32); none = by the subject's kind
    pub options: Option<String>,
}

/// What the briefs of a run render of the personas' traits (D31).
pub(crate) struct TraitCtx {
    mode: &'static str,
    set: Option<TraitSetDef>,
    defs: BTreeMap<String, TraitDef>,
}
impl TraitCtx {
    /// The context a run rendered its briefs under, from the `traits_json` the run recorded (D31); none when it rendered nothing.
    pub(crate) fn for_run(store: &Store, recorded: Option<&Value>) -> Result<Option<TraitCtx>> {
        let Some(t) = recorded else { return Ok(None) };
        Ok(match t["mode"].as_str() {
            Some("all") => Some(TraitCtx { mode: "all", set: None, defs: store.trait_map()? }),
            Some("set") => match t["set"].as_str() {
                Some(name) => store.trait_set(name)?.map(|set| TraitCtx { mode: "set", set: Some(set), defs: store.trait_map().unwrap_or_default() }),
                None => None,
            },
            _ => None,
        })
    }
    pub(crate) fn block_for(&self, p: &Persona) -> Option<String> {
        self.block(p).map(|(b, _)| b)
    }
    /// The block of one persona and the trait ids it renders; none when the mode renders nothing for it.
    fn block(&self, p: &Persona) -> Option<(String, Vec<String>)> {
        let held = p.traits.as_ref()?;
        let mut ids: Vec<String> = Vec::new();
        let mut bullets: Vec<String> = Vec::new();
        for (id, v) in held {
            // set mode renders the set's cover traits, and the persona's other set traits that say something: a band
            // at 3 (the middle) is the default reading and is left out (the 15% budget of D29)
            if let Some(set) = &self.set {
                if !set.traits.contains(id) || (!set.cover.contains(id) && is_default(v)) {
                    continue;
                }
            }
            let Some(def) = self.defs.get(id) else { continue };
            let b = def.bullets(v);
            if !b.is_empty() {
                ids.push(id.clone());
                bullets.extend(b);
            }
        }
        if ids.is_empty() {
            return None;
        }
        if let (Some(a), Some(b)) = (held.get("persuasion.involvement"), held.get("cognition.need_for_cognition")) {
            let both = ids.iter().any(|i| i == "persuasion.involvement") && ids.iter().any(|i| i == "cognition.need_for_cognition");
            if let (true, Some(line)) = (both, traits::route_line(a, b)) {
                bullets.push(line.to_string());
            }
        }
        let mut t = String::from("\n## How this reader tends to decide (told)\n\n");
        for b in &bullets {
            t.push_str(&format!("- {b}\n"));
        }
        t.push_str(CLOSING);
        Some((t, ids))
    }
}

/// Seats per wave when the caller does not say (D12).
pub const DEFAULT_WAVE: usize = 30;

/// The closing line of the trait block (D31), kept short: it is paid once per brief.
pub(crate) const CLOSING: &str = "\nTold, not seen: a line leaning on one is `told`; a line about the subject still cites it.\n";
/// A trait value that is the default reading: the middle of a band. Set mode leaves it out unless it is a cover trait.
fn is_default(v: &traits::TraitValue) -> bool {
    matches!(v, traits::TraitValue::Int(3))
}

/// What a brief costs the agent when the project says nothing (the same default `should-run` documents).
pub(crate) const DEFAULT_TOKENS_PER_BRIEF: u64 = 15_000;

fn stake_meaning(s: Stake) -> &'static str {
    match s {
        Stake::Primary => "you would use it",
        Stake::Secondary => "you buy, approve or support it",
        Stake::Bystander => "you pass by",
    }
}
fn evidence_meaning(e: Evidence) -> &'static str {
    match e {
        Evidence::Seen => "you looked at an attachment of this run and cite it",
        Evidence::Told => "this brief told you",
        Evidence::Recalled => "from your own memory; say so",
    }
}

/// An aria-json walk, one line per node: two spaces per depth, then `<id> <role> "<name>"`.
fn flatten_walk(v: &Value) -> String {
    fn go(nodes: &[Value], depth: usize, out: &mut String) {
        for n in nodes {
            let f = |k: &str| n.get(k).and_then(Value::as_str).unwrap_or("").to_string();
            out.push_str(&format!(
                "{}{} {} {}\n",
                "  ".repeat(depth),
                f("id"),
                f("role"),
                serde_json::to_string(&f("name")).expect("string serialises")
            ));
            if let Some(c) = n.get("children").and_then(Value::as_array) {
                go(c, depth + 1, out);
            }
        }
    }
    let mut out = String::new();
    if let Some(nodes) = v.get("nodes").and_then(Value::as_array) {
        go(nodes, 0, &mut out);
    }
    out
}

fn inline(root: &Path, a: &Attachment) -> Result<Option<String>> {
    if a.media == "image" {
        return Ok(None);
    }
    let p = root.join(".personajury").join("blobs").join(&a.hash);
    let bytes = std::fs::read(&p).map_err(|e| AppError::Io(p.clone(), e))?;
    let mut text = if a.media == "aria-json" {
        match serde_json::from_slice::<Value>(&bytes) {
            Ok(v) => flatten_walk(&v),
            Err(_) => String::from_utf8_lossy(&bytes).into_owned(),
        }
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    };
    let mut cut = false;
    if text.len() > CAP {
        let mut n = CAP;
        while !text.is_char_boundary(n) {
            n -= 1;
        }
        text.truncate(n);
        cut = true;
    }
    if !text.ends_with('\n') {
        text.push('\n');
    }
    if cut {
        text.push_str(&format!("[truncated at {CAP} bytes]\n"));
    }
    let longest = text
        .split(|c| c != '`')
        .map(str::len)
        .max()
        .unwrap_or(0)
        .max(2);
    let fence = "`".repeat(longest + 1);
    Ok(Some(format!("{fence}\n{text}{fence}\n")))
}

fn attachment_block(
    root: &Path,
    prefix: &str,
    atts: &[Attachment],
    detail: Detail,
    out: &mut String,
) -> Result<()> {
    for a in atts {
        out.push_str(&format!(
            "- `{prefix}{}` — {}, file `.personajury/blobs/{}`, sha256 {}\n",
            a.label, a.media, a.hash, a.hash
        ));
        if detail == Detail::Full {
            if let Some(t) = inline(root, a)? {
                out.push('\n');
                out.push_str(&t);
                out.push('\n');
            }
        }
    }
    Ok(())
}

struct Cmp {
    subject: Subject,
}

/// The persona block of a brief (and of its deliberation file): the profile bullets, then the trait block when the run renders one.
pub(crate) fn persona_section(persona: &Persona, traits_block: Option<&str>) -> String {
    let mut t = String::from("## Persona\n\n");
    if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&persona.profile_json) {
        for (k, label) in [
            ("role", "Role"),
            ("credential", "Credential"),
            ("context", "Context"),
            ("daily_tasks", "Daily tasks"),
            ("fears", "Fears"),
            ("never_reads", "Never reads"),
            ("decides_by", "Decides by"),
        ] {
            if let Some(v) = m.get(k).and_then(Value::as_str) {
                t.push_str(&format!("- {label}: {v}\n"));
            }
        }
    }
    if let Some(b) = traits_block {
        t.push_str(b);
    }
    t
}
/// The stake block: what the persona's stake is in this run and the rules it follows.
pub(crate) fn stake_section(persona: &Persona, stake: Stake) -> String {
    format!(
        "\n## Your stake in this run\n\n{}: {}.\n\nStake rules: {}\n\n",
        stake.name(),
        stake_meaning(stake),
        persona.stake_rules
    )
}

#[allow(clippy::too_many_arguments)]
fn compose_text(
    root: &Path,
    persona: &Persona,
    stake: Stake,
    lens: &Lens,
    subject: &Subject,
    cmps: &[Cmp],
    options: &[String],
    detail: Detail,
    run_on: Option<&str>,
    traits_block: Option<&str>,
) -> Result<String> {
    let mut t = String::new();
    t.push_str("personajury-brief 1\n\n");
    t.push_str(&format!(
        "# {} judges {} under the {} lens\n\n",
        persona.name, subject.name, lens.name
    ));
    t.push_str(&format!(
        "You are {} ({}). Judge the subject below as this person would, not as an expert. \
The engine that wrote this brief calls no model; you are the judge, and your answer is checked line by line against the rules at the end.\n\n",
        persona.name, persona.id
    ));
    t.push_str(&persona_section(persona, traits_block));
    t.push_str(&stake_section(persona, stake));
    // D15: a run whose panel terms name models says, in every brief, which one this brief is for
    if let Some(m) = run_on {
        t.push_str(&format!("## Model\n\nRun this brief on the model `{m}`.\n\n"));
    }
    t.push_str(&format!(
        "## Lens: {} (version {})\n\nScores are integers from {} to {}.\n\n",
        lens.name, lens.version, lens.scale[0], lens.scale[1]
    ));
    for d in &lens.dimensions {
        t.push_str(&format!("### {} — {}\n\n", d.id, d.title));
        let mut anchors: Vec<(i64, &String)> = d
            .anchors
            .iter()
            .filter_map(|(k, v)| k.parse::<i64>().ok().map(|n| (n, v)))
            .collect();
        anchors.sort();
        for (n, w) in anchors {
            t.push_str(&format!("- {n}: {w}\n"));
        }
        t.push('\n');
    }
    t.push_str("## Evidence classes this lens allows\n\n");
    for e in &lens.evidence {
        t.push_str(&format!("- `{}`: {}\n", e.name(), evidence_meaning(*e)));
    }
    t.push_str(&format!(
        "\n## Subject: {} (kind {}, version {}, hash {})\n\n",
        subject.name,
        subject.kind.name(),
        subject.version,
        subject.hash
    ));
    t.push_str("Attachments. Open what you need; paths are relative to the workspace root.\n\n");
    attachment_block(root, "", &subject.attachments, detail, &mut t)?;
    if !cmps.is_empty() {
        t.push_str("\n## Comparators (snapshots this run holds)\n\n");
        for c in cmps {
            t.push_str(&format!(
                "### cmp:{} (kind {}, version {}, hash {})\n\n",
                c.subject.name,
                c.subject.kind.name(),
                c.subject.version,
                c.subject.hash
            ));
            attachment_block(
                root,
                &format!("cmp:{}/", c.subject.name),
                &c.subject.attachments,
                detail,
                &mut t,
            )?;
            t.push('\n');
        }
        t.push_str(
            "A comparator this run holds no snapshot of may only be mentioned as `recalled`.\n",
        );
    }
    if !lens.standard.trim().is_empty() {
        t.push_str(&format!(
            "\n## The standard (`told`)\n\n{}\n",
            lens.standard.trim()
        ));
    }
    let scores_options = lens.asks_pick || lens.compares;
    if scores_options && !options.is_empty() {
        t.push_str("\n## Options\n\n`about` names the option a line scores, and the score is that option's own score on the lens anchors. A line without `about` scores the subject as a whole.");
        if lens.compares {
            t.push_str(" Under this lens a line without `about` judges the subject against the comparators: such a `seen` line must cite a held comparator (`cmp:<name>/<label>`) and may also cite subject attachments. A line `about: \"cmp:<name>\"` scores that comparator itself. Never put a comparator in `about` to score the subject against it.");
        }
        t.push_str(
            " Score every option you were asked to weigh, one line per dimension and option.\n\n",
        );
        for o in options {
            t.push_str(&format!("- `{o}`\n"));
        }
    }
    if lens.asks_pick {
        t.push_str("\n## Pick\n\nThis lens also asks you to pick one of the options above and give a reason.\n");
    }
    t.push_str("\n## How to cite\n\n- `<label>`: an attachment of the subject\n- `<label>#<node id>`: a node of an aria-json walk\n- `<label>#L<n>`: line n of a diff\n");
    t.push_str("- `cmp:<name>/<label>` (and `#…` as above): an attachment of a comparator\n");
    t.push_str("\n## Your answer\n\nAnswer with one JSON object and nothing else, in this shape (a filled example for you and this lens):\n\n");
    let example = example_verdict(persona, lens, subject, cmps, options);
    t.push_str("```json\n");
    t.push_str(&serde_json::to_string_pretty(&example).expect("example serialises"));
    t.push_str("\n```\n\nRules, checked when the verdict is ingested:\n\n");
    if scores_options {
        t.push_str("- One line per dimension and option: at most one line for each (`dimension`, `about`) pair, and at least one line for every lens dimension.\n");
        t.push_str("- `about`, when given, is one of the options above.\n");
    } else {
        t.push_str("- One line per dimension: this lens scores the subject as a whole; leave `about` out.\n");
    }
    t.push_str(&format!(
        "- `score` is an integer from {} to {}.\n",
        lens.scale[0], lens.scale[1]
    ));
    let anchors: Vec<String> = lens.anchor_scores().iter().map(i64::to_string).collect();
    t.push_str(&format!(
        "- `anchor_met`: one of {}, at or below your `score`.\n",
        anchors.join(", ")
    ));
    t.push_str(
        "- A `seen` line needs at least one citation that names something this run holds.\n",
    );
    if scores_options {
        t.push_str("- A line `about` a comparator the run holds no snapshot of (`cmp:<name>`) is `recalled` only.\n");
    }
    t.push_str(&format!(
        "- A score at the bottom of the scale ({}) needs a blocker that says what failed.\n",
        lens.scale[0]
    ));
    if lens.compares {
        t.push_str("- Under this lens a `seen` line must be `about` a comparator this run holds (`cmp:<name>`) or cite one (`cmp:<name>/<label>`): a comparison with no comparator snapshot cannot be seen.\n");
    }
    t.push_str(
        "- `blockers` lists what stops you, one sentence each; leave it empty when nothing does.\n",
    );
    if lens.asks_pick {
        t.push_str(
            "- `pick` is required: `choice` is one of the options above, with a `reason`.\n",
        );
    } else {
        t.push_str("- Leave `pick` out.\n");
    }
    Ok(t)
}

fn example_line(
    dimension: &str,
    mid: i64,
    evidence: Evidence,
    cites: Vec<String>,
    about: Option<String>,
    note: &str,
) -> Value {
    let mut line = serde_json::Map::new();
    line.insert("dimension".into(), json!(dimension));
    if let Some(a) = about {
        line.insert("about".into(), json!(a));
    }
    line.insert("score".into(), json!(mid));
    line.insert("anchor_met".into(), json!(mid));
    line.insert("evidence".into(), json!(evidence.name()));
    line.insert("citations".into(), json!(cites));
    line.insert("blockers".into(), json!([]));
    line.insert("note".into(), json!(note));
    Value::Object(line)
}

fn example_verdict(
    persona: &Persona,
    lens: &Lens,
    subject: &Subject,
    cmps: &[Cmp],
    options: &[String],
) -> Value {
    // a comparing lens with no comparator in the run cannot show a seen line (D6 A2): show the next class it allows
    let evidence = lens
        .evidence
        .iter()
        .copied()
        .find(|e| !(lens.compares && cmps.is_empty() && *e == Evidence::Seen))
        .unwrap_or(Evidence::Recalled);
    let anchors = lens.anchor_scores();
    let mid = anchors
        .get(anchors.len() / 2)
        .copied()
        .unwrap_or(lens.scale[0]);
    let mut lines = Vec::new();
    let cmp_first = cmps
        .first()
        .and_then(|c| c.subject.attachments.first().map(|a| (c, a.label.clone())));
    for (i, d) in lens.dimensions.iter().enumerate() {
        let mut cites: Vec<String> = Vec::new();
        if evidence == Evidence::Seen {
            if let Some(a) = subject.attachments.first() {
                cites.push(a.label.clone());
            }
            if lens.compares {
                if let Some((c, label)) = &cmp_first {
                    cites.push(format!("cmp:{}/{}", c.subject.name, label));
                }
            }
        }
        lines.push(example_line(
            &d.id,
            mid,
            evidence,
            cites,
            None,
            "one sentence in your own voice",
        ));
        if i == 0 && lens.compares && evidence == Evidence::Seen {
            if let Some((c, label)) = &cmp_first {
                lines.push(example_line(
                    &d.id,
                    mid,
                    evidence,
                    vec![format!("cmp:{}/{}", c.subject.name, label)],
                    Some(format!("cmp:{}", c.subject.name)),
                    "this line scores the comparator itself",
                ));
            }
        }
    }
    let mut v = serde_json::Map::new();
    v.insert("persona".into(), json!(persona.id));
    v.insert("lines".into(), Value::Array(lines));
    if lens.asks_pick {
        v.insert(
            "pick".into(),
            json!({"choice": options.first().cloned().unwrap_or_default(), "reason": "why, in one sentence"}),
        );
    }
    Value::Object(v)
}


/// The panel a brief draws from.
enum PanelSrc {
    Saved(String),
    Adhoc(String),
    /// `--seats auto`: the personas that serve the subject's audience, plus one control
    Auto,
}

/// The seats `--seats auto` chose (see `seats_auto`).
struct AutoSeats {
    resolved: super::panel::Resolved,
    /// persona → why it sits: `serves <tag>` or `control: unrelated`
    why: BTreeMap<String, String>,
    warnings: Vec<Value>,
}

/// `--seats auto`: every persona whose `serves` meets the subject's audience (a seeded draw, or the cover of the trait
/// set when it has one), then exactly one control persona that serves none of it (a seeded draw). `cap` is the total:
/// the control counts inside it, so the serving seats are at most `cap - 1` when a control sits. Serving seats are
/// primary and the control a bystander unless `--stake` says otherwise.
fn seats_auto(
    audience: &[String],
    personas: &[Persona],
    seed: [u8; 8],
    cap: usize,
    cover: &[crate::domain::cover::CoverTrait],
) -> Result<AutoSeats> {
    let meets = |p: &Persona| p.serves.iter().any(|t| audience.contains(t));
    let mut serving: Vec<&Persona> = personas.iter().filter(|p| meets(p)).collect();
    serving.sort_by(|a, b| a.id.cmp(&b.id));
    if serving.is_empty() {
        return Err(Refusal::NoMatchingPersonas { audience: audience.join(",") }.into());
    }
    // the control sits inside the cap: the total is the effort's number, never one above it (review I1)
    let has_control = personas.iter().any(|p| !meets(p));
    let cap = if has_control { cap.saturating_sub(1).max(1) } else { cap.max(1) };
    let mut rng = crate::domain::rng::SplitMix64::from_bytes(hash::term_seed(seed, "serves"));
    if serving.len() > cap {
        rng.shuffle(&mut serving);
    }
    let seated: Vec<&Persona> = if serving.len() <= cap {
        serving
    } else if cover.is_empty() {
        serving.into_iter().take(cap).collect()
    } else {
        crate::domain::cover::choose(cover, &mut crate::domain::cover::State::default(), &serving, cap, "primary")
    };
    let mut control: Vec<&Persona> = personas.iter().filter(|p| !meets(p)).collect();
    control.sort_by(|a, b| a.id.cmp(&b.id));
    if control.len() > 1 {
        crate::domain::rng::SplitMix64::from_bytes(hash::term_seed(seed, "control")).shuffle(&mut control);
    }
    let mut warnings = Vec::new();
    let mut why = BTreeMap::new();
    let mut ids = Vec::new();
    let mut stakes = Vec::new();
    for p in &seated {
        let tag = p.serves.iter().find(|t| audience.contains(t)).cloned().unwrap_or_default();
        why.insert(p.id.clone(), format!("serves {tag}"));
        ids.push(p.id.clone());
        stakes.push(Some(Stake::Primary));
    }
    let mut terms = vec![(
        Term { count: seated.len(), tag: format!("serves:{}", audience.join("+")), stake: Some(Stake::Primary), model: None, filter: vec![] },
        seated.len(),
    )];
    match control.first() {
        Some(c) => {
            why.insert(c.id.clone(), "control: unrelated".into());
            ids.push(c.id.clone());
            stakes.push(Some(Stake::Bystander));
            terms.push((Term { count: 1, tag: "control".into(), stake: Some(Stake::Bystander), model: None, filter: vec![] }, 1));
        }
        None => warnings.push(json!({"code": "no_control_persona", "note": "every persona serves this audience; no control sits"})),
    }
    let models = vec![None; ids.len()];
    let query = format!("auto: serves {}", audience.join(","));
    let hash = hash::panel_hash(&query, &ids);
    Ok(AutoSeats { resolved: super::panel::Resolved { query, ids, stakes, models, hash, terms, seed: Some(seed) }, why, warnings })
}

/// Everything the runs of one brief invocation share.
struct Shared<'a> {
    subject: &'a Subject,
    cmps: &'a [Cmp],
    panel_name: &'a str,
    resolved: &'a super::panel::Resolved,
    stakes: &'a BTreeMap<String, Stake>,
    /// the model of each persona whose panel term names one (D15)
    models: &'a BTreeMap<String, String>,
    selected: &'a [String],
    model: &'a str,
    detail: Detail,
    seed_hex: &'a str,
    override_json: Option<&'a Value>,
    wave: usize,
    effort: Option<Effort>,
    trimmed: &'a [Value],
    independence: Independence,
    /// the rule class and the effective quorum (the higher of the rule's and the sources'), recorded on the run (D13)
    rule: Option<&'a str>,
    quorum: Option<&'static str>,
    traits: Option<&'a TraitCtx>,
    /// whether the panel's draw covered traits (D30)
    cover: bool,
    /// the seats `--seats auto` chose, recorded on the run
    seats: Option<&'a Value>,
    /// `subject` or `attachments`: what a line may be about and a pick may choose (D32)
    options_mode: &'static str,
}

/// Compose and write the briefs of one lens and create its run.
fn one_run(root: &Path, store: &Store, sh: &Shared, lens: &Lens) -> Result<Value> {
    // the same list `verdict` checks `about` and a pick against: the subject's name (or its labels, D32), then
    // `cmp:<name>` per comparator
    let mut options: Vec<String> = if sh.options_mode == "subject" {
        vec![sh.subject.name.clone()]
    } else {
        sh.subject.attachments.iter().map(|x| x.label.clone()).collect()
    };
    options.extend(sh.cmps.iter().map(|c| format!("cmp:{}", c.subject.name)));
    let run = RunRow {
        id: 0,
        subject_name: sh.subject.name.clone(),
        subject_version: sh.subject.version,
        subject_hash: sh.subject.hash.clone(),
        panel_name: sh.panel_name.to_string(),
        panel_hash: sh.resolved.hash.clone(),
        lens_name: lens.name.clone(),
        lens_version: lens.version,
        model: sh.model.to_string(),
        spend: sh.selected.len() as i64,
        stakes: sh.stakes.clone(),
        personas: sh.selected.to_vec(),
        comparators: sh
            .cmps
            .iter()
            .map(|c| (c.subject.name.clone(), c.subject.version))
            .collect(),
        briefs: vec![],
        created_at: now_iso(),
        draw_seed: Some(sh.seed_hex.to_string()),
        override_json: sh.override_json.cloned(),
        effort: sh.effort.map(|e| e.name().to_string()),
        trimmed: sh.trimmed.to_vec(),
        independence: sh.independence.name().to_string(),
        rule: sh.rule.map(str::to_string),
        quorum: sh.quorum.map(str::to_string),
    };
    // compose every brief first (the text names no run), so nothing can fail between the run row and its files
    let mut texts = Vec::new();
    let mut versions: BTreeMap<String, i64> = BTreeMap::new();
    let mut incomplete: Vec<Value> = Vec::new();
    for id in sh.selected {
        let persona = store
            .persona(id)?
            .ok_or_else(|| AppError::NotFound(format!("no persona {id}")))?;
        let stake = sh.stakes[id];
        let model = sh.models.get(id).cloned().unwrap_or_else(|| sh.model.to_string());
        // every brief of a run with a model in its terms states its own; a run with none is as it always was
        let run_on = (!sh.models.is_empty()).then_some(model.as_str());
        let block = sh.traits.and_then(|c| c.block(&persona));
        if let (Some(c), Some((_, ids))) = (sh.traits, &block) {
            for i in ids {
                versions.insert(i.clone(), c.defs[i].version);
            }
        }
        if let Some(missing) = sh.traits.and_then(|c| trait_set_missing(c, &persona)) {
            incomplete.push(json!({"persona": id, "missing": missing}));
        }
        let text = compose_text(
            root, &persona, stake, lens, sh.subject, sh.cmps, &options, sh.detail, run_on,
            block.as_ref().map(|(b, _)| b.as_str()),
        )?;
        texts.push((id.clone(), persona.version, stake, text, model));
    }
    let run_id = store.insert_run(&run)?;
    // what the run rendered and how it was drawn (D31); null when nothing was rendered and the panel has no cover
    let traits_json = (sh.traits.is_some() || sh.cover).then(|| {
        let mut v = json!({
            "mode": sh.traits.map_or("off", |c| c.mode),
            "set": sh.traits.and_then(|c| c.set.as_ref()).map(|s| s.name.clone()),
            "versions": versions,
            "draw": if sh.cover { "cover" } else { "seeded" },
        });
        if !incomplete.is_empty() {
            v["incomplete"] = json!(incomplete);
        }
        v
    });
    if traits_json.is_some() || sh.seats.is_some() {
        store.set_run_traits(run_id, traits_json.as_ref(), sh.seats)?;
    }
    store.set_run_options_mode(run_id, sh.options_mode)?;
    // the terms the seats were drawn for, with who sits under each, so `matrix` can count who answered (D34)
    let mut at = 0usize;
    let mut term_rows: Vec<Value> = Vec::new();
    for (t, got) in &sh.resolved.terms {
        let ids = sh.resolved.ids.get(at..at + got).unwrap_or(&[]);
        at += got;
        let seated: Vec<&String> = ids.iter().filter(|i| sh.selected.contains(i)).collect();
        let text = match t.stake {
            Some(s) => format!("{} {}={}", t.count, t.tag, s.name()),
            None => format!("{} {}", t.count, t.tag),
        };
        term_rows.push(json!({"term": text, "tag": t.tag, "stake": t.stake.map(|s| s.name()), "asked": t.count,
                              "seated": seated.len(), "personas": seated}));
    }
    store.set_run_panel_terms(run_id, &json!(term_rows))?;
    let dir_rel = format!(".personajury/runs/{run_id}");
    let dir = root.join(&dir_rel);
    // a write that fails leaves no orphan run (final review: an orphan clears nothing and confuses check)
    let written = (|| -> Result<Vec<BriefRef>> {
        std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(dir.clone(), e))?;
        let mut refs = Vec::new();
        for (id, version, stake, text, model) in &texts {
            let rel = format!("{dir_rel}/{id}--{}.md", lens.name);
            let path = root.join(&rel);
            std::fs::write(&path, text.as_bytes()).map_err(|e| AppError::Io(path.clone(), e))?;
            refs.push(BriefRef {
                persona: id.clone(),
                persona_version: *version,
                stake: *stake,
                path: rel,
                hash: sha256_hex(text.as_bytes()),
                bytes: text.len(),
                model: model.clone(),
            });
        }
        store.set_run_briefs(run_id, &refs)?;
        Ok(refs)
    })();
    let refs = match written {
        Ok(r) => r,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            store.delete_run(run_id)?;
            return Err(e);
        }
    };
    let bytes_total: usize = refs.iter().map(|r| r.bytes).sum();
    let briefs: Vec<Value> = refs
        .iter()
        .map(|r| json!({"persona": r.persona, "stake": r.stake.name(), "path": r.path, "hash": r.hash, "bytes": r.bytes, "model": r.model}))
        .collect();
    let mut out = json!({
        "run": run_id,
        "subject": {"name": sh.subject.name, "version": sh.subject.version, "hash": sh.subject.hash},
        "lens": {"name": lens.name, "version": lens.version},
        "model": sh.model,
        "spend": sh.selected.len(),
        // what the panel query asked for and what it resolved to: under-resolution is visible, not silent
        "panel": {
            "name": sh.panel_name,
            "personas": sh.resolved.ids.len(),
            "terms": sh.resolved.terms.iter().map(|(t, got)| {
                let mut v = json!({"tag": t.tag, "wanted": t.count, "got": got});
                if let Some(s) = t.stake { v["stake"] = json!(s.name()); }
                v
            }).collect::<Vec<_>>(),
        },
        "options": {"mode": sh.options_mode, "list": options},
        "detail": if sh.detail == Detail::Full { "full" } else { "compact" },
        // run one wave, ingest it, then the next (D12): persona ids in the order of `briefs`
        "waves": sh.selected.chunks(sh.wave).map(|c| c.to_vec()).collect::<Vec<_>>(),
        "briefs": briefs,
        "bytes_total": bytes_total,
    });
    // D18: one context will read every brief; the run says so, in the order the panel sits
    if sh.independence == Independence::Sequential {
        out["sequence"] = json!(refs.iter().map(|r| r.path.clone()).collect::<Vec<_>>());
        out["note"] = json!("one context will read all of these; the run is labelled sequential");
    }
    Ok(out)
}

/// The traits of the set a persona lacks, when the run renders a set and there are some.
fn trait_set_missing(c: &TraitCtx, p: &Persona) -> Option<Vec<String>> {
    let set = c.set.as_ref()?;
    let missing: Vec<String> = set
        .traits
        .iter()
        .filter(|t| !p.traits.as_ref().is_some_and(|m| m.contains_key(*t)))
        .cloned()
        .collect();
    (!missing.is_empty()).then_some(missing)
}

/// An effort's seat cap (D17): one seat per stratum present first (primary first), then the next in the selection's
/// order (primary, then secondary, then bystander, draw order within each) up to `cap`; kept in brief order. A cap never
/// drops a whole stratum.
fn capped_seats(selected: &[String], stakes: &BTreeMap<String, Stake>, cap: usize) -> Vec<String> {
    let mut keep = std::collections::BTreeSet::new();
    let mut seen = std::collections::BTreeSet::new();
    for (i, id) in selected.iter().enumerate() {
        if seen.insert(stakes[id]) {
            keep.insert(i);
        }
    }
    let mut i = 0;
    while keep.len() < cap && i < selected.len() {
        keep.insert(i);
        i += 1;
    }
    selected.iter().enumerate().filter(|(i, _)| keep.contains(i)).map(|(_, id)| id.clone()).collect()
}

pub(crate) fn quorum_word(q: Quorum) -> &'static str {
    match q {
        Quorum::None => "none",
        Quorum::Low => "low",
        Quorum::High => "high",
        Quorum::Human => "human",
    }
}

fn detail_name(d: Detail) -> &'static str {
    if d == Detail::Full {
        "full"
    } else {
        "compact"
    }
}

pub fn compose(root: &Path, a: &BriefArgs) -> Result<Value> {
    if a.wave == Some(0) {
        return Err(AppError::InvalidArguments("--wave must be at least 1".into()));
    }
    if a.comparators.iter().any(|c| c.trim().is_empty()) {
        return Err(AppError::InvalidArguments(
            "--comparator has an empty element; name each comparator".into(),
        ));
    }
    if a.model.trim().is_empty() {
        return Err(AppError::InvalidArguments(
            "--model names the model the run is pinned to; it cannot be empty".into(),
        ));
    }
    let store = Store::open(root)?;
    let subject = store
        .subject(&a.subject)?
        .ok_or_else(|| AppError::NotFound(format!("no subject named {}", a.subject)))?;
    if subject.is_comparator {
        return Err(AppError::InvalidArguments(format!(
            "{} is a comparator; give it with --comparator and brief a subject",
            a.subject
        )));
    }

    let options_mode: &'static str = match a.options.as_deref() {
        None => subject.kind.default_options_mode(),
        Some("subject") => "subject",
        Some("attachments") => "attachments",
        Some(other) => return Err(Refusal::InvalidOptionsMode { given: other.to_string() }.into()),
    };

    // the rule, when one is named: its panel, lenses, spend and detail are the defaults (D9)
    let cfg_path = root.join("personajury.toml");
    let cfg: Option<TriggerConfig> = match &a.rule {
        Some(_) => Some(trigger::parse_config(&super::read_text(&cfg_path)?)?),
        // a toml that is there but malformed is refused, not ignored (M7)
        None => match std::fs::read_to_string(&cfg_path) {
            Ok(t) => Some(trigger::parse_config(&t)?),
            Err(_) => None,
        },
    };
    let tokens_per_brief = cfg
        .as_ref()
        .and_then(|c| c.tokens_per_brief)
        .unwrap_or(DEFAULT_TOKENS_PER_BRIEF);
    // the highest jury quorum the subject's recorded sources fall under (D13): a rule below it is an override
    let mut sources_quorum: Option<Quorum> = None;
    if let (None, Some(cfg)) = (&a.rule, &cfg) {
        let paths: Vec<String> = subject
            .attachments
            .iter()
            .map(|x| super::persona::display_path(root, Path::new(&x.source_path)))
            .collect();
        sources_quorum = trigger::classify(cfg, &paths)
            .hits
            .iter()
            .map(|h| h.quorum)
            .filter(|q| matches!(q, Quorum::Low | Quorum::High))
            .max();
    }
    let rule: Option<&TriggerRule> = match (&a.rule, &cfg) {
        (Some(class), Some(cfg)) => {
            let r = cfg.rules.iter().find(|r| r.class == *class).ok_or_else(|| {
                AppError::NotFound(format!("no rule {class} in {}", cfg_path.display()))
            })?;
            // no override can run a jury on a human class as if it decided
            if HUMAN_CLASSES.contains(&trigger::normalise_class(&r.class).as_str())
                || r.quorum == Quorum::Human
            {
                return Err(Refusal::HumanClassUnderPersonaQuorum {
                    class: r.class.clone(),
                    quorum: "a persona jury".into(),
                }
                .into());
            }
            if r.quorum == Quorum::None {
                return Err(AppError::InvalidArguments(format!(
                    "rule {class} has quorum none: no jury runs on it"
                )));
            }
            // a page that falls under a human rule is never briefed under another rule's name, whatever the overrides
            let paths: Vec<String> = subject
                .attachments
                .iter()
                .map(|x| super::persona::display_path(root, Path::new(&x.source_path)))
                .collect();
            for hit in trigger::classify(cfg, &paths).hits {
                if matches!(hit.quorum, Quorum::Low | Quorum::High) {
                    sources_quorum = sources_quorum.max(Some(hit.quorum));
                }
                if hit.quorum == Quorum::Human
                    || HUMAN_CLASSES.contains(&trigger::normalise_class(&hit.class).as_str())
                {
                    return Err(Refusal::HumanPathUnderPersonaQuorum {
                        class: hit.class.clone(),
                        path: hit.paths.first().cloned().unwrap_or_default(),
                    }
                    .into());
                }
            }
            Some(r)
        }
        _ => None,
    };

    if a.panel.is_some() && a.query.is_some() {
        return Err(AppError::InvalidArguments(
            "give --panel (a saved panel) or --query (an ad-hoc one), not both".into(),
        ));
    }
    let auto = match a.seats.as_deref() {
        None => false,
        Some("auto") => true,
        Some(other) => {
            return Err(AppError::InvalidArguments(format!("--seats {other:?}: the only value is auto")));
        }
    };
    if auto && (a.panel.is_some() || a.query.is_some()) {
        return Err(AppError::InvalidArguments(
            "--seats auto picks its own seats from the subject's audience; give no --panel or --query".into(),
        ));
    }
    let panel_src = match (&a.query, &a.panel, rule) {
        _ if auto => PanelSrc::Auto,
        (Some(q), _, _) => PanelSrc::Adhoc(q.clone()),
        (None, Some(p), _) => PanelSrc::Saved(p.clone()),
        (None, None, Some(r)) => PanelSrc::Saved(r.panel.clone().unwrap_or_default()),
        (None, None, None) => {
            return Err(AppError::InvalidArguments(
                "give --panel <saved panel>, --query \"<terms>\" or --rule <class>".into(),
            ))
        }
    };
    let mut lens_names: Vec<String> = Vec::new();
    for l in &a.lenses {
        if !lens_names.contains(l) {
            lens_names.push(l.clone());
        }
    }
    let lenses_from_rule = lens_names.is_empty();
    if lenses_from_rule {
        match rule {
            Some(r) => lens_names = r.lenses.clone(),
            None => {
                return Err(AppError::InvalidArguments(
                    "give --lens <name> or --rule <class>".into(),
                ))
            }
        }
    }
    let detail = a
        .detail
        .or(rule.map(|r| r.detail))
        .unwrap_or(Detail::Compact);

    // the trait mode (D31): the flag, else the rule's trait_set, else off; a set must exist
    let rule_traits: String = rule.and_then(|r| r.trait_set.clone()).unwrap_or_else(|| "off".into());
    let mode_word: String = a.traits.clone().map(|t| t.trim().to_string()).unwrap_or_else(|| rule_traits.clone());
    let trait_ctx: Option<TraitCtx> = match mode_word.as_str() {
        "off" => None,
        "all" => Some(TraitCtx { mode: "all", set: None, defs: store.trait_map()? }),
        name => {
            let set = store.trait_set(name)?.ok_or_else(|| Refusal::TraitSetUnknown { name: name.to_string() })?;
            Some(TraitCtx { mode: "set", set: Some(set), defs: store.trait_map()? })
        }
    };

    // a rule that names a panel or a lens the store lacks is its own refusal; a caller's own choice is a plain not_found
    let (panel_from_rule, rule_class) = (
        matches!((&a.panel, &a.query), (None, None)),
        rule.map(|r| r.class.clone()).unwrap_or_default(),
    );
    if let PanelSrc::Saved(name) = &panel_src {
        if store.panel(name)?.is_none() {
            return Err(if panel_from_rule && rule.is_some() {
                Refusal::RulePanelUnknown {
                    rule: rule_class,
                    panel: name.clone(),
                }
                .into()
            } else {
                AppError::NotFound(format!("no panel named {name}"))
            });
        }
    }
    // a subject whose sources moved since `subject add` would brief stale blobs (final review, minor)
    if let Some(why) = super::verdict::sources_moved(&subject)? {
        return Err(Refusal::SubjectMoved {
            subject: subject.name.clone(),
            why,
        }
        .into());
    }
    let mut lenses: Vec<Lens> = Vec::new();
    for n in &lens_names {
        let lens = store.lens(n)?.ok_or_else(|| {
            if lenses_from_rule {
                Refusal::RuleLensUnknown {
                    rule: rule_class.clone(),
                    lens: n.clone(),
                }
                .into()
            } else {
                AppError::NotFound(format!("no lens named {n}"))
            }
        })?;
        // the lens name and every persona id become a file name below (I6): checked again here, before the join
        super::slug("lens name", &lens.name)?;
        lenses.push(lens);
    }
    let mut cmp_names = a.comparators.clone();
    cmp_names.sort();
    cmp_names.dedup();
    let mut cmps = Vec::new();
    for n in &cmp_names {
        let c = store
            .subject(n)?
            .ok_or_else(|| AppError::NotFound(format!("no comparator named {n}")))?;
        if !c.is_comparator {
            return Err(AppError::InvalidArguments(format!(
                "{n} is a subject, not a comparator; add it with `personajury comparator add`"
            )));
        }
        cmps.push(Cmp { subject: c });
    }

    // the draw: seeded from the subject's NAME and the panel's name, so a second round draws the same seats
    let (panel_name, seed_name) = match &panel_src {
        PanelSrc::Auto => ("(auto)".to_string(), "auto".to_string()),
        PanelSrc::Saved(n) => (n.clone(), n.clone()),
        // seeded by the parsed, normalised terms: spacing and case in the query never change who sits
        PanelSrc::Adhoc(q) => ("(query)".to_string(), format!("query:{}", query::normalise(&query::parse(q)?))),
    };
    let seed = match &a.draw {
        Some(h) => super::panel::parse_seed(h)?,
        None => hash::draw_seed(&subject.name, &seed_name),
    };
    let mut auto_seats: Option<AutoSeats> = None;
    let mut auto_cover: Vec<String> = Vec::new();
    let resolved = match &panel_src {
        PanelSrc::Auto => {
            let audience = store.subject_audience(&subject.name)?;
            if audience.is_empty() {
                return Err(Refusal::AudienceUnknown { subject: subject.name.clone() }.into());
            }
            // the cap on the whole jury, the control included: the effort's seats, else the ceiling of D10
            let e = a.effort.or(cfg.as_ref().and_then(|c| c.effort)).or(rule.map(|_| Effort::Medium));
            let cap = e.and_then(Effort::seat_cap).unwrap_or(if a.allow_large { SEAT_HARD_MAX } else { SEAT_CEILING });
            auto_cover = trait_ctx.as_ref().and_then(|c| c.set.as_ref()).map(|s| s.cover.clone()).unwrap_or_default();
            let defs = store.trait_map()?;
            let cover = super::panel::cover_traits(&auto_cover, &defs)?;
            let got = seats_auto(&audience, &store.personas()?, seed, cap, &cover)?;
            let r = got.resolved.clone();
            auto_seats = Some(got);
            r
        }
        PanelSrc::Saved(n) => super::panel::resolve_in(&store, n, Some(seed))?,
        PanelSrc::Adhoc(q) => {
            let terms = query::parse(q)?;
            super::panel::check_seats(super::panel::seats(&terms), a.allow_large)?;
            super::panel::check_filters(q, &store.trait_map()?)?;
            super::panel::draw_query(&query::normalise(&terms), &store.personas()?, Some(seed), &[])?
        }
    };
    if let Some((t, _)) = resolved.terms.iter().find(|(_, got)| *got == 0) {
        return Err(Refusal::PanelTermEmpty {
            panel: panel_name.clone(),
            tag: t.tag.clone(),
        }
        .into());
    }

    // a stake per persona: --stake wins over the panel's term, a term over nothing
    let explicit: BTreeMap<String, Stake> = a.stakes.iter().cloned().collect();
    for k in explicit.keys() {
        if !resolved.ids.contains(k) {
            return Err(AppError::InvalidArguments(format!(
                "--stake names {k}, who is not in panel {panel_name}"
            )));
        }
    }
    let mut stakes: BTreeMap<String, Stake> = BTreeMap::new();
    let mut stake_overrides: Vec<Value> = Vec::new();
    for (id, term) in resolved.ids.iter().zip(&resolved.stakes) {
        let stake = match (explicit.get(id), term) {
            (Some(g), Some(t)) if g != t => {
                stake_overrides.push(
                    json!({"persona": id, "panel": t.name(), "given": g.name()}),
                );
                *g
            }
            (Some(g), _) => *g,
            (None, Some(t)) => *t,
            (None, None) => return Err(Refusal::PersonaWithoutStake { persona: id.clone() }.into()),
        };
        stakes.insert(id.clone(), stake);
    }
    let size = resolved.ids.len();
    let spend_arg = a.spend.or(rule.filter(|r| r.spend > 0).map(|r| r.spend));
    // a rule's `spend = 0` is the whole panel, capped at 12 (1000 with --allow-large): the rest are cut in draw order
    let cap = if a.allow_large { SEAT_HARD_MAX } else { SEAT_CEILING };
    let capped = rule.is_some() && spend_arg.is_none() && size > cap;
    let spend = if capped { cap } else { spend_arg.unwrap_or(size) };
    if spend == 0 {
        return Err(AppError::InvalidArguments(
            "--spend must be at least 1".into(),
        ));
    }
    if spend > size {
        return Err(Refusal::PanelTooSmall {
            panel: panel_name.clone(),
            got: size,
            spend,
        }
        .into());
    }
    let mut ordered: Vec<&String> = resolved.ids.iter().collect();
    ordered.sort_by_key(|id| stakes[*id]); // stable: panel order within a stake
    let mut selected: Vec<String> = ordered.into_iter().take(spend).cloned().collect();
    for id in &selected {
        super::slug("persona id", id)?;
    }

    // D17: the rule says what a change needs, effort says what you will spend; the run gets the smaller. Only a run
    // under a rule, an effort the caller or the toml named, or a budget is planned; any other brief is as in v0.3.
    let effort: Option<Effort> = a
        .effort
        .or(cfg.as_ref().and_then(|c| c.effort))
        .or(rule.map(|_| Effort::Medium));
    let budget = cfg.as_ref().and_then(|c| c.budget_tokens);
    let mut trimmed: Vec<Value> = Vec::new();
    if let Some(e) = effort {
        let why = format!("effort {}", e.name());
        // a lens list the caller wrote is the caller's; the rule's list is cut to its first under `low`
        if e == Effort::Low && lenses_from_rule && lenses.len() > 1 {
            let from: Vec<String> = lenses.iter().map(|l| l.name.clone()).collect();
            lenses.truncate(1);
            trimmed.push(json!({"what": "lenses", "from": from, "to": [lenses[0].name], "why": why}));
        }
        // an explicit --spend is the caller's own choice (and a recorded override where it differs from the rule)
        if a.spend.is_none() && !auto {
            let kept = match e.seat_cap() {
                Some(c) if selected.len() > c => capped_seats(&selected, &stakes, c),
                _ => selected.clone(),
            };
            if kept.len() < selected.len() {
                trimmed.push(json!({"what": "seats", "from": selected.len(), "to": kept.len(), "why": why}));
                selected = kept;
            }
        }
    }
    if let Some(b) = budget {
        let per = tokens_per_brief;
        let fits = |seats: usize, lens: usize| {
            (seats as u64).checked_mul(lens as u64).and_then(|n| n.checked_mul(per)).is_some_and(|t| t <= b)
        };
        let strata = selected.iter().map(|id| stakes[id]).collect::<std::collections::BTreeSet<_>>().len();
        let floor = (strata as u64).saturating_mul(per);
        if floor > b {
            return Err(Refusal::OverBudget {
                needed: floor,
                budget: b,
                minimum_plan: json!({"seats": strata, "lenses": 1, "briefs": strata, "tokens": floor}),
            }
            .into());
        }
        let why = format!("budget_tokens {b}");
        // lenses after the first go first, then seats in reverse draw order, never the last of a stratum
        if lenses.len() > 1 && !fits(selected.len(), lenses.len()) {
            let from: Vec<String> = lenses.iter().map(|l| l.name.clone()).collect();
            while lenses.len() > 1 && !fits(selected.len(), lenses.len()) {
                lenses.pop();
            }
            let to: Vec<String> = lenses.iter().map(|l| l.name.clone()).collect();
            trimmed.push(json!({"what": "lenses", "from": from, "to": to, "why": why}));
        }
        if !fits(selected.len(), lenses.len()) {
            let from = selected.len();
            while !fits(selected.len(), lenses.len()) {
                let drop = (0..selected.len()).rev().find(|i| {
                    selected.iter().filter(|o| stakes[*o] == stakes[&selected[*i]]).count() > 1
                });
                match drop {
                    Some(i) => {
                        selected.remove(i);
                    }
                    None => break,
                }
            }
            trimmed.push(json!({"what": "seats", "from": from, "to": selected.len(), "why": why}));
        }
    }
    // D10: 12 seats a brief unless the caller says so; never more than 1000
    super::panel::check_seats(selected.len(), a.allow_large)?;
    let large = selected.len() > SEAT_CEILING;

    // where the caller departs from the rule, or runs more than 12 seats, the run says so and says why
    let override_json: Option<Value> = match rule {
        // no rule, no departure; but a brief above 12 seats still says why, with `rule: null` (D12, review M9)
        None if large => {
            let reason = a.reason.as_deref().map(str::trim).unwrap_or("");
            if reason.is_empty() {
                return Err(Refusal::OverrideWithoutReason {
                    rule: "(none)".into(),
                    fields: "seats".into(),
                }
                .into());
            }
            Some(json!({
                "rule": Value::Null,
                "fields": {"seats": {"rule": SEAT_CEILING, "given": selected.len()}},
                "reason": reason,
                "by": a.by.as_deref().map(str::trim).filter(|b| !b.is_empty()).unwrap_or("agent"),
            }))
        }
        None => None,
        Some(r) => {
            let mut fields = serde_json::Map::new();
            match &panel_src {
                PanelSrc::Adhoc(q) => {
                    fields.insert(
                        "panel".into(),
                        json!({"rule": r.panel, "given": format!("query: {q}")}),
                    );
                }
                PanelSrc::Saved(p) if Some(p) != r.panel.as_ref() => {
                    fields.insert("panel".into(), json!({"rule": r.panel, "given": p}));
                }
                PanelSrc::Saved(_) => {}
                // the seats follow the subject's audience, the same way every time: not an override by itself (I5);
                // apply-check's `thin_primary` guards a primary stratum of one
                PanelSrc::Auto => {}
            }
            let (mut given_l, mut rule_l) = (lens_names.clone(), r.lenses.clone());
            given_l.sort();
            rule_l.sort();
            if !lenses_from_rule && given_l != rule_l {
                fields.insert("lenses".into(), json!({"rule": r.lenses, "given": lens_names}));
            }
            if let Some(n) = a.spend {
                if n != r.spend && !(r.spend == 0 && n == size) {
                    fields.insert("spend".into(), json!({"rule": r.spend, "given": n}));
                }
            }
            if let Some(d) = a.detail {
                if d != r.detail {
                    fields.insert(
                        "detail".into(),
                        json!({"rule": detail_name(r.detail), "given": detail_name(d)}),
                    );
                }
            }
            if !stake_overrides.is_empty() {
                let pick = |k: &str| -> serde_json::Map<String, Value> {
                    stake_overrides
                        .iter()
                        .map(|o| (o["persona"].as_str().unwrap_or("").to_string(), o[k].clone()))
                        .collect()
                };
                fields.insert(
                    "stakes".into(),
                    json!({"rule": Value::Object(pick("panel")), "given": Value::Object(pick("given"))}),
                );
            }
            if a.traits.is_some() && mode_word != rule_traits {
                fields.insert("traits".into(), json!({"rule": rule_traits, "given": mode_word}));
            }
            if a.draw.is_some() {
                let derived = super::panel::hex_seed(hash::draw_seed(&subject.name, &seed_name));
                if derived != super::panel::hex_seed(seed) {
                    fields.insert(
                        "draw".into(),
                        json!({"rule": derived, "given": super::panel::hex_seed(seed)}),
                    );
                }
            }
            if large {
                fields.insert(
                    "seats".into(),
                    json!({"rule": SEAT_CEILING, "given": selected.len()}),
                );
            }
            // a rule whose quorum is below the one the sources fall under lowers the jury: an override (D13)
            if let Some(sq) = sources_quorum.filter(|sq| *sq > r.quorum) {
                fields.insert(
                    "quorum".into(),
                    json!({"sources": quorum_word(sq), "given": quorum_word(r.quorum)}),
                );
            }
            if fields.is_empty() {
                None
            } else {
                let reason = a.reason.as_deref().map(str::trim).unwrap_or("");
                if reason.is_empty() {
                    return Err(Refusal::OverrideWithoutReason {
                        rule: r.class.clone(),
                        fields: fields.keys().cloned().collect::<Vec<_>>().join(", "),
                    }
                    .into());
                }
                Some(json!({
                    "rule": r.class,
                    "fields": Value::Object(fields),
                    "reason": reason,
                    "by": a.by.as_deref().map(str::trim).filter(|b| !b.is_empty()).unwrap_or("agent"),
                }))
            }
        }
    };

    // the cost, before anyone spends it; an overflowing estimate is refused before a single run is written
    let briefs = selected
        .len()
        .checked_mul(lenses.len())
        .filter(|b| (*b as u64).checked_mul(tokens_per_brief).is_some())
        .ok_or(Refusal::PanelTooLarge {
            seats: usize::MAX,
            ceiling: SEAT_HARD_MAX,
        })?;
    let tokens = briefs as u64 * tokens_per_brief;
    let seed_hex = super::panel::hex_seed(seed);
    // the models panel terms name, by persona (D15); empty when no term does
    let models: BTreeMap<String, String> = resolved
        .ids
        .iter()
        .zip(&resolved.models)
        .filter_map(|(id, m)| m.clone().map(|m| (id.clone(), m)))
        .filter(|(id, _)| selected.contains(id))
        .collect();
    let quorum = rule.map(|r| r.quorum).max(sources_quorum).filter(|q| matches!(q, Quorum::Low | Quorum::High));
    let cover_ids: Vec<String> = match &panel_src {
        PanelSrc::Saved(n) => store.panel_cover(n)?,
        PanelSrc::Adhoc(_) => vec![],
        PanelSrc::Auto => auto_cover.clone(),
    };
    // the seats `--seats auto` chose and why, for the seats that survived the effort and the budget
    let seats_value: Option<Value> = auto_seats.as_ref().map(|g| {
        json!(selected
            .iter()
            .map(|id| json!({"persona": id, "why": g.why.get(id).cloned().unwrap_or_default()}))
            .collect::<Vec<_>>())
    });
    let sh = Shared {
        subject: &subject,
        cmps: &cmps,
        panel_name: &panel_name,
        resolved: &resolved,
        stakes: &stakes,
        models: &models,
        selected: &selected,
        model: &a.model,
        detail,
        seed_hex: &seed_hex,
        override_json: override_json.as_ref(),
        wave: a.wave.unwrap_or(DEFAULT_WAVE),
        effort,
        trimmed: &trimmed,
        independence: a.independence,
        rule: rule.map(|r| r.class.as_str()),
        quorum: quorum.map(quorum_word),
        traits: trait_ctx.as_ref(),
        cover: !cover_ids.is_empty(),
        seats: seats_value.as_ref(),
        options_mode,
    };
    let mut runs = Vec::new();
    for lens in &lenses {
        runs.push(one_run(root, &store, &sh, lens)?);
    }

    let mut out = if a.rule.is_none() && runs.len() == 1 {
        runs.remove(0)
    } else {
        json!({ "rule": a.rule, "runs": runs })
    };
    // the cost, before anyone spends it: briefs × the project's tokens per brief
    out["estimate"] = json!({"briefs": briefs, "tokens": tokens});
    if capped {
        out["capped"] = json!(true);
    }
    out["draw"] = json!({"seed": seed_hex});
    if let Some(e) = effort {
        out["effort"] = json!(e.name());
        out["trimmed"] = json!(trimmed);
    } else if !trimmed.is_empty() {
        out["trimmed"] = json!(trimmed);
    }
    if let Some(b) = budget {
        out["budget_tokens"] = json!(b);
    }
    out["independence"] = json!(a.independence.name());
    if let (Some(v), Some(g)) = (&seats_value, &auto_seats) {
        out["seats"] = v.clone();
        if !g.warnings.is_empty() {
            out["warnings"] = json!(g.warnings);
        }
    }
    // what the seated panel covers of the panel's traits, counted per stratum and panel-wide (D30)
    let seated: Vec<(String, String)> = selected.iter().map(|id| (id.clone(), stakes[id].name().to_string())).collect();
    if let Some(c) = super::panel::coverage(&store, &cover_ids, &seated)? {
        out["coverage"] = c;
    }
    if let Some(c) = trait_ctx.as_ref().filter(|c| c.set.is_some()) {
        let incomplete: Vec<Value> = selected
            .iter()
            .filter_map(|id| {
                let p = store.persona(id).ok().flatten()?;
                trait_set_missing(c, &p).map(|m| json!({"persona": id, "missing": m}))
            })
            .collect();
        if !incomplete.is_empty() {
            out["trait_set_incomplete"] = json!(incomplete);
        }
    }
    if large {
        out["large"] = json!(true);
    }
    if !stake_overrides.is_empty() {
        out["stake_overrides"] = json!(stake_overrides);
    }
    if let Some(o) = override_json {
        out["override"] = o;
    }
    Ok(out)
}
