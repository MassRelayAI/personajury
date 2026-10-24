//! The types of the store (report §4). Pure data: no I/O. Workers fill behaviour in the sibling modules; names here are
//! pinned by the v0.1 plan (docs/plans/2026-10-23-v0-1-the-engine.md) and change only with a decision.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The most seats a panel or a brief holds unless the caller says `--allow-large` (D10).
pub const SEAT_CEILING: usize = 12;
/// The most seats any panel or brief holds, even with `--allow-large` (D10).
pub const SEAT_HARD_MAX: usize = 1000;

/// A persona's stake in one run (report §2.3). Ordered primary < secondary < bystander: the `--spend` order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stake {
    Primary,
    Secondary,
    Bystander,
}
impl Stake {
    pub fn parse(s: &str) -> Option<Stake> {
        match s {
            "primary" => Some(Stake::Primary),
            "secondary" => Some(Stake::Secondary),
            "bystander" => Some(Stake::Bystander),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Stake::Primary => "primary",
            Stake::Secondary => "secondary",
            Stake::Bystander => "bystander",
        }
    }
}

/// What a verdict line stands on (report §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Evidence {
    /// attached to the run: an ARIA node, a screenshot, a diff line, a comparator snapshot
    Seen,
    /// text the brief supplied
    Told,
    /// the model's own memory
    Recalled,
}
impl Evidence {
    pub fn parse(s: &str) -> Option<Evidence> {
        match s {
            "seen" => Some(Evidence::Seen),
            "told" => Some(Evidence::Told),
            "recalled" => Some(Evidence::Recalled),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Evidence::Seen => "seen",
            Evidence::Told => "told",
            Evidence::Recalled => "recalled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubjectKind {
    Page,
    Diff,
    Design,
    Logo,
    Document,
    Menu,
    Prices,
}
impl SubjectKind {
    pub const ALL: [SubjectKind; 7] = [
        SubjectKind::Page,
        SubjectKind::Diff,
        SubjectKind::Design,
        SubjectKind::Logo,
        SubjectKind::Document,
        SubjectKind::Menu,
        SubjectKind::Prices,
    ];
    pub fn parse(s: &str) -> Option<SubjectKind> {
        Self::ALL.into_iter().find(|k| k.name() == s)
    }
    pub fn name(self) -> &'static str {
        match self {
            SubjectKind::Page => "page",
            SubjectKind::Diff => "diff",
            SubjectKind::Design => "design",
            SubjectKind::Logo => "logo",
            SubjectKind::Document => "document",
            SubjectKind::Menu => "menu",
            SubjectKind::Prices => "prices",
        }
    }
}

/// How much a brief or an answer carries (D3): compact for a cheap mid-development run, full for a release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Detail {
    #[default]
    Compact,
    Full,
}
impl Detail {
    pub fn parse(s: &str) -> Option<Detail> {
        match s {
            "compact" => Some(Detail::Compact),
            "full" => Some(Detail::Full),
            _ => None,
        }
    }
}

/// How much a jury may spend (D17): the words people know from their agent's effort setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    Low,
    Medium,
    High,
    Max,
}
impl Effort {
    pub fn parse(s: &str) -> Option<Effort> {
        match s {
            "low" => Some(Effort::Low),
            "medium" => Some(Effort::Medium),
            "high" => Some(Effort::High),
            "max" => Some(Effort::Max),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High => "high",
            Effort::Max => "max",
        }
    }
    /// The most seats a brief of this effort takes of the rule's panel; none for `max` (the ceilings of D10 apply).
    pub fn seat_cap(self) -> Option<usize> {
        match self {
            Effort::Low => Some(3),
            Effort::Medium => Some(7),
            Effort::High => Some(12),
            Effort::Max => None,
        }
    }
}

/// Whether the jurors of a run were separate contexts or one context that read every brief (D18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Independence {
    #[default]
    Independent,
    Sequential,
}
impl Independence {
    pub fn parse(s: &str) -> Option<Independence> {
        match s {
            "independent" => Some(Independence::Independent),
            "sequential" => Some(Independence::Sequential),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Independence::Independent => "independent",
            Independence::Sequential => "sequential",
        }
    }
}

/// `hand`, or `generated-from:<material sha256>` in the store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    Hand,
    GeneratedFrom(String),
}
impl Origin {
    pub fn to_db(&self) -> String {
        match self {
            Origin::Hand => "hand".into(),
            Origin::GeneratedFrom(h) => format!("generated-from:{h}"),
        }
    }
    pub fn from_db(s: &str) -> Origin {
        match s.strip_prefix("generated-from:") {
            Some(h) => Origin::GeneratedFrom(h.to_string()),
            None => Origin::Hand,
        }
    }
}

/// The fixed persona JSON (report §4): what `persona add --file` reads and `persona generate` asks the agent to write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonaFile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub role: String,
    pub context: String,
    pub daily_tasks: String,
    pub fears: String,
    pub never_reads: String,
    pub decides_by: String,
    pub stake_rules: String,
    /// who this person is as an expert, in words (D19): shown, never checked; the tag `expert` is a convention
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}
/// The longest credential a persona file may carry: a line, not a biography (D19).
pub const CREDENTIAL_MAX: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Persona {
    pub id: String,
    /// 0 before the store assigns one
    pub version: i64,
    pub name: String,
    /// lower-cased, sorted, unique
    pub tags: Vec<String>,
    pub stake_rules: String,
    /// the canonical JSON of the PersonaFile fields role, context, daily_tasks, fears, never_reads, decides_by
    pub profile_json: String,
    pub origin: Origin,
    /// sha256 of the canonical JSON of the whole PersonaFile
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimension {
    pub id: String,
    pub title: String,
    /// anchor score ("1", "3", "5" in the shipped lenses) → what that score looks like, in words
    pub anchors: BTreeMap<String, String>,
}

/// A lens as its TOML file reads (lenses/*.toml) and as `lens.body_json` stores it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lens {
    pub name: String,
    pub version: i64,
    /// inclusive [min, max]
    pub scale: [i64; 2],
    #[serde(default)]
    pub asks_pick: bool,
    /// a lens that judges against comparators: a `seen` line must be about a held comparator or cite one (D6 A2)
    #[serde(default)]
    pub compares: bool,
    /// the evidence classes a line under this lens may carry
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub standard: String,
    #[serde(rename = "dimension")]
    pub dimensions: Vec<Dimension>,
}
/// An id or name the engine may use in a path (final review I6): `^[a-z0-9][a-z0-9._-]{0,63}$`, never `..`.
pub fn is_slug(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && b.len() <= 64
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'.' | b'_' | b'-')
        })
        && !s.contains("..")
}
/// `Err` naming what is wrong unless `s` is a slug (I6).
pub fn check_slug(what: &str, s: &str) -> Result<(), String> {
    if is_slug(s) {
        Ok(())
    } else {
        Err(format!(
            "{what} {s:?} must match ^[a-z0-9][a-z0-9._-]{{0,63}}$ and must not contain \"..\""
        ))
    }
}

impl Dimension {
    /// this dimension's anchor scores as integers, ascending (keys that do not parse are refused by `Lens::validate`)
    pub fn anchor_scores(&self) -> Vec<i64> {
        let mut v: Vec<i64> = self.anchors.keys().filter_map(|k| k.parse().ok()).collect();
        v.sort();
        v.dedup();
        v
    }
}

impl Lens {
    /// Parse and validate (final review, minor "no lens validation").
    pub fn from_toml(s: &str) -> Result<Lens, String> {
        let l: Lens = toml::from_str(s).map_err(|e| e.to_string())?;
        l.validate()?;
        Ok(l)
    }
    /// A lens the refusals can stand on: a slug name, scale min < max, at least one evidence class and one dimension,
    /// unique dimension ids, every anchor an integer inside the scale.
    pub fn validate(&self) -> Result<(), String> {
        check_slug("lens name", &self.name)?;
        let [min, max] = self.scale;
        if min >= max {
            return Err(format!(
                "scale [{min}, {max}]: the minimum must be below the maximum"
            ));
        }
        if self.evidence.is_empty() {
            return Err("evidence lists no class; a lens takes at least one".into());
        }
        if self.dimensions.is_empty() {
            return Err("the lens has no dimension; it needs at least one".into());
        }
        let mut ids = BTreeSet::new();
        for d in &self.dimensions {
            if d.id.trim().is_empty() {
                return Err("a dimension has an empty id".into());
            }
            if !ids.insert(d.id.as_str()) {
                return Err(format!("dimension {} appears twice", d.id));
            }
            if d.anchors.is_empty() {
                return Err(format!("dimension {} has no anchor", d.id));
            }
            for k in d.anchors.keys() {
                let n: i64 = k
                    .parse()
                    .map_err(|_| format!("dimension {}: anchor {k:?} is not an integer", d.id))?;
                if n < min || n > max {
                    return Err(format!(
                        "dimension {}: anchor {n} is outside the scale [{min}, {max}]",
                        d.id
                    ));
                }
            }
        }
        Ok(())
    }
    /// the anchor scores as integers, ascending
    pub fn anchor_scores(&self) -> Vec<i64> {
        let mut v: Vec<i64> = self
            .dimensions
            .iter()
            .flat_map(|d| d.anchors.keys().filter_map(|k| k.parse().ok()))
            .collect::<BTreeSet<i64>>()
            .into_iter()
            .collect();
        v.sort();
        v
    }
    /// spacing between adjacent anchors (2 for 1/3/5): "one anchor" in delta (report §10(2))
    pub fn anchor_step(&self) -> i64 {
        let a = self.anchor_scores();
        a.windows(2).map(|w| w[1] - w[0]).min().unwrap_or(1)
    }
}

/// `aria-json` | `image` | `diff` | `file`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    pub label: String,
    pub media: String,
    pub hash: String,
    /// the path as given to `subject add`, kept to detect a moved subject
    pub source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Subject {
    pub name: String,
    pub version: i64,
    pub kind: SubjectKind,
    pub is_comparator: bool,
    /// domain::hash::subject_hash(kind, attachments)
    pub hash: String,
    /// sorted by label
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PanelRow {
    pub name: String,
    pub query: String,
    pub persona_ids: Vec<String>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BriefRef {
    pub persona: String,
    pub persona_version: i64,
    pub stake: Stake,
    /// relative to the workspace root
    pub path: String,
    pub hash: String,
    pub bytes: usize,
    /// the model this brief is to be run on (D15): the term's, else the run's `--model`
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunRow {
    /// 0 before insert
    pub id: i64,
    pub subject_name: String,
    pub subject_version: i64,
    pub subject_hash: String,
    pub panel_name: String,
    pub panel_hash: String,
    pub lens_name: String,
    pub lens_version: i64,
    pub model: String,
    pub spend: i64,
    /// every panel persona's stake, as declared at brief time
    pub stakes: BTreeMap<String, Stake>,
    /// the personas briefed, in spend order
    pub personas: Vec<String>,
    /// (comparator name, version) the run holds snapshots of
    pub comparators: Vec<(String, i64)>,
    pub briefs: Vec<BriefRef>,
    pub created_at: String,
    /// the hex seed the panel draw used (schema v3); none for a run made before the draw existed
    pub draw_seed: Option<String>,
    /// `{rule, fields:{name:{rule,given}}, reason, by}` when the caller departed from a rule (schema v3, D10)
    pub override_json: Option<serde_json::Value>,
    /// the effort word the run was planned under (schema v4, D17); none for a run made with no rule and no effort
    pub effort: Option<String>,
    /// what effort or the budget cut from the plan: `[{what, from, to, why}]` (schema v4, D17)
    pub trimmed: Vec<serde_json::Value>,
    /// `independent` (one context per brief) or `sequential` (one context read every brief), schema v4, D18
    pub independence: String,
    /// the rule class the run was briefed under (`brief --rule`), schema v4, D13; none for a run with no rule
    pub rule: Option<String>,
    /// the run's effective quorum at brief time, `low | high`: the higher of its rule's and its sources' (D13)
    pub quorum: Option<String>,
}

/// One term of a panel query: "5 builders" → { count: 5, tag: "builders", stake: None };
/// "3 builder=primary" carries the stake the audience gives those seats, and "3 builder=primary@model" the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Term {
    pub count: usize,
    pub tag: String,
    pub stake: Option<Stake>,
    /// the model these seats run on (D15): `3 builder=primary@claude-sonnet-5-5`
    pub model: Option<String>,
}

// ---- the verdict file the agent writes (ingested by `jurypersona verdict`) ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerdictInput {
    pub persona: String,
    pub lines: Vec<VerdictLine>,
    #[serde(default)]
    pub pick: Option<PickInput>,
    /// what the agent says this verdict cost: `{input_tokens, output_tokens}`, non-negative integers (D12). The engine
    /// measures nothing; `validate` refuses a shape it cannot read as `invalid_usage`.
    #[serde(default)]
    pub usage: Option<serde_json::Value>,
}

/// Tokens an agent reported for one verdict (D12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerdictLine {
    pub dimension: String,
    pub score: i64,
    pub anchor_met: i64,
    pub evidence: Evidence,
    /// the option this line is about (D6 A3): an attachment label of the subject or `cmp:<name>`; none = the subject
    /// as a whole. One line per (dimension, about).
    #[serde(default)]
    pub about: Option<String>,
    /// `<label>[#<locator>]` or `cmp:<name>/<label>[#<locator>]`
    #[serde(default)]
    pub citations: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PickInput {
    pub choice: String,
    pub reason: String,
}

/// What one attachment offers to citation resolution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AttachmentView {
    pub hash: String,
    pub media: String,
    /// node ids of an aria-json walk (every depth)
    pub aria_ids: BTreeSet<String>,
    /// line count of a diff or text file
    pub lines: usize,
    /// a `file` that decodes as UTF-8: its `#L<n>` locators are checked against `lines`
    pub text: bool,
    /// a text attachment whose label ends `.csv`: the fields in each record, for `R<row>C<col>` locators (D21)
    pub csv: Option<Vec<usize>>,
}

/// Everything a run holds as evidence: the subject's attachments by label, each comparator's by name then label.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunEvidence {
    pub attachments: BTreeMap<String, AttachmentView>,
    pub comparators: BTreeMap<String, BTreeMap<String, AttachmentView>>,
}

#[derive(Debug, Clone)]
pub struct VerdictContext {
    pub lens: Lens,
    pub subject_name: String,
    /// the personas briefed in this run and their stakes
    pub stakes: BTreeMap<String, Stake>,
    pub evidence: RunEvidence,
    /// subject attachment labels, then `cmp:<name>` for each comparator of the run: what a line may be `about` and,
    /// when the lens asks for one, what a pick may choose
    pub options: Vec<String>,
    /// "the same point" of this persona's dismissed findings on this subject (report §10(3), D6 B3):
    /// `<dimension>\t<citation hash>` for a cited finding, the finding id for an uncited one
    pub dismissed: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidLine {
    pub line: VerdictLine,
    /// resolved citations `<attachment hash>#<locator>`, sorted
    pub resolved: Vec<String>,
    pub citation_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub id: String,
    pub persona: String,
    pub dimension: String,
    /// the option the line was about (I9): part of the finding id and of the dismissed key
    pub about: Option<String>,
    pub text: String,
    pub citation_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidVerdict {
    pub persona: String,
    pub stake: Stake,
    pub lines: Vec<ValidLine>,
    pub pick: Option<PickInput>,
    /// one per blocker, id = hash::finding_id(..)
    pub findings: Vec<Finding>,
    pub usage: Option<Usage>,
}

// ---- what the store returns for judging ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerdictRow {
    pub run_id: i64,
    pub batch: i64,
    pub persona: String,
    pub persona_version: i64,
    pub dimension: String,
    pub score: i64,
    pub anchor_met: i64,
    pub evidence: Evidence,
    pub about: Option<String>,
    pub citations: Vec<String>,
    pub citation_hash: String,
    pub blockers: Vec<String>,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PickRow {
    pub run_id: i64,
    pub batch: i64,
    pub persona: String,
    pub choice: String,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FindingRow {
    pub id: String,
    pub run_id: i64,
    pub batch: i64,
    pub persona: String,
    pub subject_name: String,
    pub dimension: String,
    pub about: Option<String>,
    pub text: String,
    pub citation_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecisionRow {
    pub id: i64,
    pub finding_id: String,
    /// fixed | dismissed | row
    pub outcome: String,
    pub why: Option<String>,
    pub row_ref: Option<String>,
    pub by: String,
    pub created_at: String,
    /// the highest run id when the decision was recorded: a run above it was created after the decision (C1)
    pub after_run: i64,
}

/// One score in a matrix or delta: (persona, lens, dimension, about) → score, with the blockers of that line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScoreRow {
    pub persona: String,
    pub lens: String,
    pub dimension: String,
    pub about: Option<String>,
    pub score: i64,
    pub anchor_met: i64,
    pub evidence: Evidence,
    pub citations: Vec<String>,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregate {
    PerStratum,
    /// never served: domain::matrix::build refuses it (report §5)
    AcrossStrata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cell {
    pub lens: String,
    pub dimension: String,
    /// the option the cell is about (D6 A3); none = the subject as a whole
    pub about: Option<String>,
    /// persona → score (Detail::Full only; empty in compact answers)
    pub scores: BTreeMap<String, i64>,
    pub n: usize,
    /// the lower middle of the sorted scores (D6 A6): an integer, never a mean
    pub median: i64,
    pub min: i64,
    pub max: i64,
    pub blockers: usize,
    /// how many of the cell's scores stand on each evidence class (I4)
    pub evidence: EvidenceCounts,
    /// persona → evidence class of its score; drives the table's suffixes, never serialised
    #[serde(skip)]
    pub classes: BTreeMap<String, Evidence>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct EvidenceCounts {
    pub seen: usize,
    pub told: usize,
    pub recalled: usize,
}
impl EvidenceCounts {
    pub fn add(&mut self, e: Evidence) {
        match e {
            Evidence::Seen => self.seen += 1,
            Evidence::Told => self.told += 1,
            Evidence::Recalled => self.recalled += 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Stratum {
    pub stake: Stake,
    pub personas: Vec<String>,
    /// ordered by lens then dimension as the lens declares them
    pub cells: Vec<Cell>,
}

/// Strata side by side. There is deliberately no field that spans strata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Matrix {
    pub strata: Vec<Stratum>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Move {
    pub persona: String,
    pub lens: String,
    pub dimension: String,
    pub about: Option<String>,
    pub before: Option<i64>,
    pub after: Option<i64>,
    pub moved: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Delta {
    pub moves: Vec<Move>,
    pub blockers_before: usize,
    pub blockers_after: usize,
    /// blockers per stratum, before and after (I1); a stratum absent from both runs is left out
    pub blockers_by_stratum: BTreeMap<Stake, BlockerCount>,
    /// cells where after < before - step
    pub regressions: Vec<Move>,
    /// (persona, lens, dimension, about) cells of the earlier run the later run lacks (C2)
    pub incomplete: Vec<CellKey>,
    /// no regression, no stratum's blockers up, every earlier cell present, and blockers down (or none either side)
    pub landed: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct BlockerCount {
    pub before: usize,
    pub after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CellKey {
    pub persona: String,
    pub lens: String,
    pub dimension: String,
    pub about: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quorum {
    None,
    Low,
    High,
    Human,
}

/// The human classes (report §10): these classes can only ever sit under `human`.
pub const HUMAN_CLASSES: [&str; 7] = [
    "money",
    "permission",
    "deploy",
    "secret",
    "licence",
    "public-claim",
    "brand",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerRule {
    pub class: String,
    pub paths: Vec<String>,
    pub quorum: Quorum,
    #[serde(default)]
    pub panel: Option<String>,
    #[serde(default)]
    pub lenses: Vec<String>,
    /// personas per lens; 0 = the full panel
    #[serde(default)]
    pub spend: usize,
    /// the weight of the briefs this rule asks for (D6 C)
    #[serde(default)]
    pub detail: Detail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerConfig {
    #[serde(default)]
    pub tokens_per_brief: Option<u64>,
    /// how much a jury may spend (D17); `medium` when a rule is named and this is silent
    #[serde(default)]
    pub effort: Option<Effort>,
    /// a hard cap on a brief's estimated tokens (D17): the run is fitted under it or refused as `over_budget`
    #[serde(default)]
    pub budget_tokens: Option<u64>,
    /// where `summary --save` writes `<date>-<subject>.md` (D24); unset: nothing is written
    #[serde(default)]
    pub report_dir: Option<String>,
    #[serde(rename = "rule", default)]
    pub rules: Vec<TriggerRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleHit {
    pub class: String,
    pub quorum: Quorum,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Classification {
    pub quorum: Quorum,
    /// every rule that matched at least one path, in config order
    pub hits: Vec<RuleHit>,
    /// paths no rule matched
    pub unmatched: Vec<String>,
}
