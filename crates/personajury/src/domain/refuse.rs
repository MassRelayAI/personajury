//! The refusals of report §5 and §10 that a verdict can trip, in the order the plan fixes. Pure.
use super::model::*;
use super::{citation, hash};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error("line {line}: a seen line needs at least one citation")]
    SeenWithoutCitation { line: usize },
    #[error("citation {citation}: {why}")]
    CitationUnresolved { citation: String, why: String },
    #[error("line {line}: comparator {comparator} has no snapshot in this run; such a line may only be recalled")]
    ComparatorNotSnapshotted { line: usize, comparator: String },
    #[error("line {line}: score {score} is outside the lens scale {min}..={max}")]
    ScoreOutOfRange {
        line: usize,
        score: i64,
        min: i64,
        max: i64,
    },
    #[error("line {line}: the lens has no dimension {dimension}")]
    UnknownDimension { line: usize, dimension: String },
    #[error("line {line}: anchor_met {anchor} is not an anchor of the lens at or below the score")]
    BadAnchor { line: usize, anchor: i64 },
    #[error("line {line}: the lens {lens} does not take {evidence} evidence")]
    EvidenceNotAllowed {
        line: usize,
        lens: String,
        evidence: String,
    },
    #[error("scores are reported per stratum; a table that averages across strata is refused")]
    StrataAverageRefused,
    #[error(
        "line {line}: dimension {dimension} about {} already has a line; one line per dimension and option",
        about.as_deref().unwrap_or("the whole subject")
    )]
    DuplicateDimension {
        line: usize,
        dimension: String,
        about: Option<String>,
    },
    #[error("line {line}: about {about:?} is not an option of this run (an attachment label or cmp:<name>); a lens that neither picks nor compares scores the subject as a whole: leave `about` out")]
    UnknownAbout { line: usize, about: String },
    #[error("line {line}: a seen line under a comparing lens must be about a comparator this run holds or cite one (cmp:<name>/<label>)")]
    SeenCompareWithoutComparator { line: usize },
    #[error("line {line}: the bottom score needs a blocker that says what failed")]
    BottomScoreWithoutBlocker { line: usize },
    #[error("the verdict has no line for dimension {dimension}; one line per lens dimension")]
    MissingDimension { dimension: String },
    #[error("subject {subject} changed since the brief: {why}")]
    SubjectMoved { subject: String, why: String },
    #[error("persona {persona} has no stake in this run")]
    PersonaWithoutStake { persona: String },
    #[error("a pick needs a reason")]
    PickWithoutReason,
    #[error("pick {choice} is not one of: {options}")]
    PickUnknownOption { choice: String, options: String },
    #[error("the lens asks for a pick and the verdict has none")]
    PickMissing,
    #[error("the lens {lens} asks for no pick; leave `pick` out")]
    PickNotAsked { lens: String },
    #[error("panel {panel} resolves to {got} personas, fewer than --spend {spend}")]
    PanelTooSmall {
        panel: String,
        got: usize,
        spend: usize,
    },
    #[error("rule {class}: a human class can never sit under a persona quorum ({quorum})")]
    HumanClassUnderPersonaQuorum { class: String, quorum: String },
    #[error(
        "persona {persona} raised a dismissed point again with the same evidence ({citation_hash})"
    )]
    DismissedPointRepeated {
        persona: String,
        citation_hash: String,
    },
    #[error("the panel asks for {seats} seats, above the ceiling of {ceiling}{}", if *ceiling == SEAT_CEILING { "; pass --allow-large to go up to 1000" } else { "; 1000 is the most any panel or brief can seat" })]
    PanelTooLarge { seats: usize, ceiling: usize },
    #[error("rule {class}: a rule with quorum low or high needs a `panel`")]
    RuleWithoutPanel { class: String },
    #[error("rule {rule} names the panel {panel}, which the store lacks; add it with `personajury panel add`")]
    RulePanelUnknown { rule: String, panel: String },
    #[error("rule {rule} names the lens {lens}, which the store lacks; add it with `personajury lens add`")]
    RuleLensUnknown { rule: String, lens: String },
    #[error("panel {panel}: the term {tag:?} resolves to no persona in the pool; a panel with an empty term is not the audience it names")]
    PanelTermEmpty { panel: String, tag: String },
    #[error("this brief departs from rule {rule} ({fields}); say why with --reason \"<text>\"")]
    OverrideWithoutReason { rule: String, fields: String },
    #[error("{path} falls under the human class {class}: a persona jury can advise on it but never decide it, under any rule or override")]
    HumanPathUnderPersonaQuorum { class: String, path: String },
    #[error("usage: {0}; give {{\"input_tokens\": <integer >= 0>, \"output_tokens\": <integer >= 0>}} or leave it out")]
    InvalidUsage(String),
    #[error("the brief needs {needed} tokens at the least (one seat per stratum, one lens) and budget_tokens is {budget}")]
    OverBudget {
        needed: u64,
        budget: u64,
        minimum_plan: serde_json::Value,
    },
    #[error("finding {finding} was raised by a human batch: only a person decides it; give --by-human \"<name>\"")]
    HumanDecisionRequired { finding: String },
    #[error("run {run} has no verdict entered with --by-human: calibrate compares the jury with a human run")]
    NotHumanRun { run: i64 },
    #[error("the credential is {len} characters; keep it a line of at most {max}")]
    CredentialTooLong { len: usize, max: usize },
    #[error("invalid trait file: {why}")]
    InvalidTraitFile { why: String },
    #[error("trait id {id}: {why}; it is not a trait")]
    RefusedTraitId { id: String, why: String },
    #[error("the text of trait {id} is refused: {why}")]
    TraitTextRefused { id: String, why: String },
    #[error("trait set {name} lists {traits} traits; a set holds at most 8")]
    TraitSetTooLarge { name: String, traits: usize },
    #[error("{name} names {cover} cover traits; a cover holds at most 4")]
    CoverTooLarge { name: String, cover: usize },
    #[error("the store has no trait {id}; install the catalog with `personajury trait install` or add the file")]
    UnknownTrait { id: String },
    #[error("trait {id}: {value} is not a value this trait can hold")]
    TraitValueOffScale { id: String, value: String },
    #[error("trait {id}: {value} is not an option of this trait (or a list on a trait that holds one)")]
    TraitOptionUnknown { id: String, value: String },
    #[error("a persona holds at most 8 traits, not {count}")]
    TraitsTooMany { count: usize },
    #[error("query term {term:?}: {why}")]
    InvalidFilter { term: String, why: String },
    #[error("trait {id} has no quote: every suggested trait carries a verbatim sentence or phrase of the persona's prose as its evidence")]
    SuggestionWithoutQuote { id: String },
    #[error("trait {id}: the quote {quote:?} is too short; a quote is at least 3 words and 12 characters of the persona's prose")]
    QuoteTooShort { id: String, quote: String },
    #[error("trait {id}: the quote {quote:?} is not inside one field of the persona's prose (whitespace folded); copy the words as they are written")]
    QuoteNotFound { id: String, quote: String },
    #[error("{path} is not a calibration file")]
    NotACalibration { path: String },
    #[error("no trait set named {name}; add it with `personajury trait set add` or install the catalog")]
    TraitSetUnknown { name: String },
    #[error("no persona serves the audience {audience}")]
    NoMatchingPersonas { audience: String },
    #[error("subject {subject} has no audience; add it with `subject add --audience tag,...`")]
    AudienceUnknown { subject: String },
    #[error("--options {given:?}: the words are subject and attachments")]
    InvalidOptionsMode { given: String },
    #[error("seats without an accepted initial verdict: {}; a deliberation needs the whole jury's first word", missing.join(", "))]
    IncompleteJuryForDeliberation { missing: Vec<String> },
    #[error("run {run} was already deliberated; v0.6 runs one round")]
    AlreadyDeliberated { run: i64 },
    #[error("run {run} has no deliberation; run `personajury deliberate {run}` first")]
    NoDeliberation { run: i64 },
    #[error("persona {persona} has no accepted initial verdict in this run; a deliberated verdict answers one")]
    DeliberatedWithoutInitial { persona: String },
    #[error("--deliberated needs the file to carry \"phase\": \"deliberated\"")]
    DeliberatedWithoutPhase,
    #[error("the file says \"phase\": \"deliberated\"; ingest it with --deliberated")]
    PhaseWithoutFlag,
    #[error("the questions file: {why}")]
    InvalidQuestions { why: String },
    #[error("the questions file: {why}")]
    QuestionClassMissing { why: String },
    #[error("--neutral: the questions lean on an option ({hits}); a juror must not be told what the author prefers")]
    LeakedRecommendation { hits: String },
    #[error("run {run} has a deliberation round: its independent batch was frozen when the round started, so no initial verdict is taken after it")]
    InitialClosed { run: i64 },
    #[error("{}", if question.is_empty() { format!("picks: {why}") } else { format!("pick for {question}: {why}") })]
    QuestionPickInvalid { question: String, why: String },
    #[error("{0}")]
    InvalidQuery(String),
    #[error("{0}")]
    InvalidConfig(String),
    #[error("{0}")]
    InvalidDiff(String),
}
impl Refusal {
    /// Stable error codes (the plan's refusal table).
    pub fn code(&self) -> &'static str {
        match self {
            Refusal::SeenWithoutCitation { .. } => "seen_without_citation",
            Refusal::CitationUnresolved { .. } => "citation_unresolved",
            Refusal::ComparatorNotSnapshotted { .. } => "comparator_not_snapshotted",
            Refusal::ScoreOutOfRange { .. } => "score_out_of_range",
            Refusal::UnknownDimension { .. } => "unknown_dimension",
            Refusal::BadAnchor { .. } => "bad_anchor",
            Refusal::EvidenceNotAllowed { .. } => "evidence_not_allowed",
            Refusal::StrataAverageRefused => "strata_average_refused",
            Refusal::DuplicateDimension { .. } => "duplicate_dimension",
            Refusal::UnknownAbout { .. } => "unknown_about",
            Refusal::SeenCompareWithoutComparator { .. } => "seen_compare_without_comparator",
            Refusal::BottomScoreWithoutBlocker { .. } => "bottom_score_without_blocker",
            Refusal::MissingDimension { .. } => "missing_dimension",
            Refusal::SubjectMoved { .. } => "subject_moved",
            Refusal::PersonaWithoutStake { .. } => "persona_without_stake",
            Refusal::PickWithoutReason => "pick_without_reason",
            Refusal::PickUnknownOption { .. } => "pick_unknown_option",
            Refusal::PickMissing => "pick_missing",
            Refusal::PickNotAsked { .. } => "pick_not_asked",
            Refusal::PanelTooSmall { .. } => "panel_too_small",
            Refusal::HumanClassUnderPersonaQuorum { .. } => "human_class_under_persona_quorum",
            Refusal::DismissedPointRepeated { .. } => "dismissed_point_repeated",
            Refusal::PanelTooLarge { .. } => "panel_too_large",
            Refusal::RuleWithoutPanel { .. } => "rule_without_panel",
            Refusal::RulePanelUnknown { .. } => "rule_panel_unknown",
            Refusal::RuleLensUnknown { .. } => "rule_lens_unknown",
            Refusal::PanelTermEmpty { .. } => "panel_term_empty",
            Refusal::OverrideWithoutReason { .. } => "override_without_reason",
            Refusal::HumanPathUnderPersonaQuorum { .. } => "human_class_under_persona_quorum",
            Refusal::OverBudget { .. } => "over_budget",
            Refusal::CredentialTooLong { .. } => "credential_too_long",
            Refusal::NotHumanRun { .. } => "not_human_run",
            Refusal::HumanDecisionRequired { .. } => "human_decision_required",
            Refusal::InvalidUsage(_) => "invalid_usage",
            Refusal::InvalidTraitFile { .. } => "invalid_trait_file",
            Refusal::RefusedTraitId { .. } => "refused_trait_id",
            Refusal::TraitTextRefused { .. } => "trait_text_refused",
            Refusal::TraitSetTooLarge { .. } => "trait_set_too_large",
            Refusal::CoverTooLarge { .. } => "cover_too_large",
            Refusal::UnknownTrait { .. } => "unknown_trait",
            Refusal::TraitValueOffScale { .. } => "trait_value_off_scale",
            Refusal::TraitOptionUnknown { .. } => "trait_option_unknown",
            Refusal::TraitsTooMany { .. } => "traits_too_many",
            Refusal::InvalidFilter { .. } => "invalid_filter",
            Refusal::TraitSetUnknown { .. } => "trait_set_unknown",
            Refusal::SuggestionWithoutQuote { .. } => "suggestion_without_quote",
            Refusal::QuoteTooShort { .. } => "quote_too_short",
            Refusal::QuoteNotFound { .. } => "quote_not_found",
            Refusal::NotACalibration { .. } => "not_a_calibration",
            Refusal::NoMatchingPersonas { .. } => "no_matching_personas",
            Refusal::AudienceUnknown { .. } => "audience_unknown",
            Refusal::InvalidOptionsMode { .. } => "invalid_options_mode",
            Refusal::IncompleteJuryForDeliberation { .. } => "incomplete_jury_for_deliberation",
            Refusal::AlreadyDeliberated { .. } => "already_deliberated",
            Refusal::NoDeliberation { .. } => "no_deliberation",
            Refusal::DeliberatedWithoutInitial { .. } => "deliberated_without_initial",
            Refusal::DeliberatedWithoutPhase => "deliberated_without_phase",
            Refusal::PhaseWithoutFlag => "phase_without_flag",
            Refusal::InvalidQuestions { .. } => "invalid_questions",
            Refusal::QuestionClassMissing { .. } => "question_class_missing",
            Refusal::LeakedRecommendation { .. } => "leaked_recommendation",
            Refusal::InitialClosed { .. } => "initial_closed",
            Refusal::QuestionPickInvalid { .. } => "question_pick_invalid",
            Refusal::InvalidQuery(_) => "invalid_query",
            Refusal::InvalidConfig(_) => "invalid_config",
            Refusal::InvalidDiff(_) => "invalid_diff",
        }
    }
}

impl Refusal {
    /// Data an envelope carries beside the error: the ceiling a too-large panel ran into.
    pub fn data(&self) -> Option<serde_json::Value> {
        match self {
            Refusal::PanelTooLarge { seats, ceiling } => {
                Some(serde_json::json!({"ceiling": ceiling, "seats": seats}))
            }
            Refusal::HumanPathUnderPersonaQuorum { class, path } => {
                Some(serde_json::json!({"class": class, "path": path}))
            }
            Refusal::OverBudget { needed, budget, minimum_plan } => Some(
                serde_json::json!({"needed": needed, "budget": budget, "minimum_plan": minimum_plan}),
            ),
            Refusal::RefusedTraitId { id, why } | Refusal::TraitTextRefused { id, why } => {
                Some(serde_json::json!({"id": id, "why": why}))
            }
            Refusal::TraitSetTooLarge { name, traits } => Some(serde_json::json!({"name": name, "traits": traits})),
            Refusal::CoverTooLarge { name, cover } => Some(serde_json::json!({"name": name, "cover": cover})),
            Refusal::UnknownTrait { id } => Some(serde_json::json!({"id": id})),
            Refusal::TraitValueOffScale { id, value } | Refusal::TraitOptionUnknown { id, value } => {
                Some(serde_json::json!({"id": id, "value": value}))
            }
            Refusal::TraitsTooMany { count } => Some(serde_json::json!({"count": count})),
            Refusal::InvalidFilter { term, why } => Some(serde_json::json!({"term": term, "why": why})),
            Refusal::TraitSetUnknown { name } => Some(serde_json::json!({"name": name})),
            Refusal::SuggestionWithoutQuote { id } => Some(serde_json::json!({"id": id})),
            Refusal::QuoteTooShort { id, quote } | Refusal::QuoteNotFound { id, quote } => {
                Some(serde_json::json!({"id": id, "quote": quote}))
            }
            Refusal::NotACalibration { path } => Some(serde_json::json!({"path": path})),
            Refusal::IncompleteJuryForDeliberation { missing } => Some(serde_json::json!({"missing": missing})),
            Refusal::AlreadyDeliberated { run } | Refusal::NoDeliberation { run } | Refusal::InitialClosed { run } => Some(serde_json::json!({"run": run})),
            Refusal::QuestionPickInvalid { question, why } => Some(serde_json::json!({"question": question, "why": why})),
            Refusal::InvalidOptionsMode { given } => Some(serde_json::json!({"given": given})),
            _ => None,
        }
    }
}

/// The dismissed key of a cited point (R12, D6 B3, I9): `dimension \t about \t citation hash`, about empty for the
/// subject as a whole. An uncited point is keyed by its finding id instead.
pub fn dismissed_key(dimension: &str, about: Option<&str>, citation_hash: &str) -> String {
    format!("{dimension}\t{}\t{citation_hash}", about.unwrap_or(""))
}

/// Validate one verdict file against its run (plan Task 1: the order of checks is fixed there).
pub fn validate(v: &VerdictInput, cx: &VerdictContext) -> Result<ValidVerdict, Refusal> {
    let stake = *cx
        .stakes
        .get(&v.persona)
        .ok_or_else(|| Refusal::PersonaWithoutStake {
            persona: v.persona.clone(),
        })?;
    let usage = v.usage.as_ref().map(parse_usage).transpose()?;
    let [min, max] = cx.lens.scale;
    let mut lines = Vec::new();
    let mut findings = Vec::new();
    for (i, input_line) in v.lines.iter().enumerate() {
        let n = i + 1;
        // a blank or whitespace blocker is no blocker (final review, minor)
        let mut line = input_line.clone();
        line.blockers.retain(|b| !b.trim().is_empty());
        let line = &line;
        let Some(dim) = cx.lens.dimensions.iter().find(|d| d.id == line.dimension) else {
            return Err(Refusal::UnknownDimension {
                line: n,
                dimension: line.dimension.clone(),
            });
        };
        // the anchors of this line's own dimension, not the union of the lens's (final review, minor)
        let anchors = dim.anchor_scores();
        // one line per (dimension, about) (D6 A3)
        if lines
            .iter()
            .any(|l: &ValidLine| l.line.dimension == line.dimension && l.line.about == line.about)
        {
            return Err(Refusal::DuplicateDimension {
                line: n,
                dimension: line.dimension.clone(),
                about: line.about.clone(),
            });
        }
        if line.score < min || line.score > max {
            return Err(Refusal::ScoreOutOfRange {
                line: n,
                score: line.score,
                min,
                max,
            });
        }
        if !anchors.contains(&line.anchor_met) || line.anchor_met > line.score {
            return Err(Refusal::BadAnchor {
                line: n,
                anchor: line.anchor_met,
            });
        }
        // the worst verdict must name what failed, or it produces no finding and escapes the gate (D6 A5)
        if line.score == min && line.blockers.is_empty() {
            return Err(Refusal::BottomScoreWithoutBlocker { line: n });
        }
        if !cx.lens.evidence.contains(&line.evidence) {
            return Err(Refusal::EvidenceNotAllowed {
                line: n,
                lens: cx.lens.name.clone(),
                evidence: line.evidence.name().into(),
            });
        }
        if line.evidence == Evidence::Seen && line.citations.is_empty() {
            return Err(Refusal::SeenWithoutCitation { line: n });
        }
        let mut resolved = line
            .citations
            .iter()
            .map(|c| citation::resolve(c, &cx.evidence))
            .collect::<Result<Vec<_>, _>>()?;
        resolved.sort();
        // `about` is one of the run's options; a `cmp:<name>` the run holds no snapshot of may only be recalled (R3)
        if let Some(about) = &line.about {
            if !(cx.lens.asks_pick || cx.lens.compares) {
                return Err(Refusal::UnknownAbout {
                    line: n,
                    about: about.clone(),
                });
            }
            if !cx.options.contains(about) {
                match about.strip_prefix("cmp:") {
                    Some(name) if !name.is_empty() && !name.contains('/') => {
                        if line.evidence != Evidence::Recalled {
                            return Err(Refusal::ComparatorNotSnapshotted {
                                line: n,
                                comparator: name.to_string(),
                            });
                        }
                    }
                    _ => {
                        return Err(Refusal::UnknownAbout {
                            line: n,
                            about: about.clone(),
                        })
                    }
                }
            }
        }
        // a comparison with no comparator snapshot cannot be seen (D6 A2): about a held comparator (an unheld one was
        // refused above unless recalled) or citing one (every citation resolved above)
        if cx.lens.compares
            && line.evidence == Evidence::Seen
            && !line.about.as_deref().is_some_and(|a| a.starts_with("cmp:"))
            && !line.citations.iter().any(|c| c.starts_with("cmp:"))
        {
            return Err(Refusal::SeenCompareWithoutComparator { line: n });
        }
        let citation_hash = citation::citation_hash(&resolved);
        // "The same point" (R12, D6 B3, I9): with citations, the same dimension, option and citation hash; without
        // any, the same finding id (the same dimension, option and text), so one shared empty hash never silences every
        // later uncited point.
        let about = line.about.as_deref();
        if !line.blockers.is_empty() {
            let repeated = if resolved.is_empty() {
                line.blockers.iter().any(|b| {
                    cx.dismissed.contains(&hash::finding_id(
                        &cx.subject_name,
                        &v.persona,
                        &line.dimension,
                        about,
                        b,
                        &citation_hash,
                    ))
                })
            } else {
                cx.dismissed
                    .contains(&dismissed_key(&line.dimension, about, &citation_hash))
            };
            if repeated {
                return Err(Refusal::DismissedPointRepeated {
                    persona: v.persona.clone(),
                    citation_hash,
                });
            }
        }
        for b in &line.blockers {
            findings.push(Finding {
                id: hash::finding_id(
                    &cx.subject_name,
                    &v.persona,
                    &line.dimension,
                    about,
                    b,
                    &citation_hash,
                ),
                persona: v.persona.clone(),
                dimension: line.dimension.clone(),
                about: line.about.clone(),
                text: b.clone(),
                citation_hash: citation_hash.clone(),
            });
        }
        lines.push(ValidLine {
            line: line.clone(),
            resolved,
            citation_hash,
        });
    }
    if let Some(d) = cx
        .lens
        .dimensions
        .iter()
        .find(|d| !lines.iter().any(|l| l.line.dimension == d.id))
    {
        return Err(Refusal::MissingDimension {
            dimension: d.id.clone(),
        });
    }
    if !cx.questions.is_empty() {
        // a decision run is picked per question (D37): one entry of `picks` for each, from that question's own options
        let bad = |q: &str, why: String| Refusal::QuestionPickInvalid { question: q.to_string(), why };
        if v.pick.is_some() {
            return Err(bad("", "a decision run is picked per question: give `picks`, leave `pick` out".into()));
        }
        for p in &v.picks {
            if p.reason.trim().is_empty() {
                return Err(Refusal::PickWithoutReason);
            }
            let Some(q) = cx.questions.iter().find(|q| q.id == p.about) else {
                return Err(bad(&p.about, format!("{} is not a question of this run", p.about)));
            };
            if !q.options.contains(&p.choice) {
                return Err(bad(&q.id, format!("{:?} is not one of: {}", p.choice, q.options.join(" | "))));
            }
            if v.picks.iter().filter(|x| x.about == p.about).count() > 1 {
                return Err(bad(&q.id, "picked more than once".into()));
            }
        }
        if let Some(q) = cx.questions.iter().find(|q| !v.picks.iter().any(|p| p.about == q.id)) {
            return Err(bad(&q.id, "no pick; every question needs one".into()));
        }
    } else if !v.picks.is_empty() {
        return Err(Refusal::QuestionPickInvalid { question: String::new(), why: "this run has no questions; use `pick`".into() });
    }
    match &v.pick {
        _ if !cx.questions.is_empty() => {}
        None if cx.lens.asks_pick => return Err(Refusal::PickMissing),
        Some(_) if !cx.lens.asks_pick => {
            return Err(Refusal::PickNotAsked {
                lens: cx.lens.name.clone(),
            })
        }
        Some(p) => {
            if p.reason.trim().is_empty() {
                return Err(Refusal::PickWithoutReason);
            }
            if cx.lens.asks_pick && !cx.options.contains(&p.choice) {
                return Err(Refusal::PickUnknownOption {
                    choice: p.choice.clone(),
                    options: cx.options.join(", "),
                });
            }
        }
        None => {}
    }
    Ok(ValidVerdict {
        persona: v.persona.clone(),
        stake,
        lines,
        pick: v.pick.clone(),
        picks: v.picks.clone(),
        findings,
        usage,
    })
}

/// The agent's reported usage: exactly `input_tokens` and `output_tokens`, each a non-negative integer (D12).
fn parse_usage(u: &serde_json::Value) -> Result<Usage, Refusal> {
    let bad = |why: &str| Refusal::InvalidUsage(why.to_string());
    let m = u.as_object().ok_or_else(|| bad("must be an object"))?;
    if let Some(k) = m.keys().find(|k| *k != "input_tokens" && *k != "output_tokens") {
        return Err(Refusal::InvalidUsage(format!("{k} is not a usage field")));
    }
    let n = |k: &str| match m.get(k) {
        None => Err(Refusal::InvalidUsage(format!("{k} is missing"))),
        Some(v) => v
            .as_u64()
            .filter(|n| *n <= i64::MAX as u64)
            .ok_or_else(|| Refusal::InvalidUsage(format!("{k} must be a non-negative integer no larger than {}", i64::MAX))),
    };
    Ok(Usage {
        input_tokens: n("input_tokens")?,
        output_tokens: n("output_tokens")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{AttachmentView, Dimension};
    use std::collections::{BTreeMap, BTreeSet};

    fn lens(asks_pick: bool, ev: &[Evidence]) -> Lens {
        let anchors = BTreeMap::from([
            ("1".into(), "bad".into()),
            ("3".into(), "ok".into()),
            ("5".into(), "good".into()),
        ]);
        Lens {
            name: "L".into(),
            version: 1,
            scale: [1, 5],
            asks_pick,
            compares: false,
            evidence: ev.to_vec(),
            standard: String::new(),
            dimensions: vec![Dimension {
                id: "clarity".into(),
                title: "Clarity".into(),
                anchors,
            }],
        }
    }
    fn cx() -> VerdictContext {
        let view = AttachmentView {
            hash: "H1".into(),
            media: "aria-json".into(),
            aria_ids: BTreeSet::from(["n1".to_string()]),
            lines: 0,
            ..Default::default()
        };
        VerdictContext {
            lens: lens(false, &[Evidence::Seen, Evidence::Told, Evidence::Recalled]),
            subject_name: "site".into(),
            stakes: BTreeMap::from([("ana".to_string(), Stake::Primary)]),
            evidence: RunEvidence {
                attachments: BTreeMap::from([("page".to_string(), view)]),
                comparators: BTreeMap::new(),
            },
            options: vec!["page".into()],
            dismissed: BTreeSet::new(),
            questions: vec![],
        }
    }
    // a lens that asks for a pick: the only non-comparing lens where `about` is allowed
    fn about_cx() -> VerdictContext {
        let mut c = cx();
        c.lens.asks_pick = true;
        c
    }
    fn vp(lines: Vec<VerdictLine>) -> VerdictInput {
        let mut v = vi(lines);
        v.pick = Some(PickInput {
            choice: "page".into(),
            reason: "r".into(),
        });
        v
    }
    fn line() -> VerdictLine {
        VerdictLine {
            dimension: "clarity".into(),
            score: 3,
            anchor_met: 3,
            evidence: Evidence::Seen,
            about: None,
            citations: vec!["page#n1".into()],
            blockers: vec![],
            note: None,
        }
    }
    fn vi(lines: Vec<VerdictLine>) -> VerdictInput {
        VerdictInput {
            persona: "ana".into(),
            lines,
            pick: None,
            picks: vec![],
            usage: None,
            phase: None,
        }
    }
    fn code(v: &VerdictInput, cx: &VerdictContext) -> &'static str {
        validate(v, cx).unwrap_err().code()
    }

    #[test]
    fn valid_verdict_passes() {
        let mut l = line();
        l.blockers = vec!["cannot find the price".into()];
        let ok = validate(&vi(vec![l]), &cx()).unwrap();
        assert_eq!(ok.stake, Stake::Primary);
        assert_eq!(ok.lines[0].resolved, vec!["H1#n1"]);
        assert_eq!(ok.findings.len(), 1);
        let f = &ok.findings[0];
        assert_eq!(
            f.id,
            crate::domain::hash::finding_id(
                "site",
                "ana",
                "clarity",
                None,
                "cannot find the price",
                &f.citation_hash
            )
        );
        assert_eq!(f.citation_hash, ok.lines[0].citation_hash);
    }
    #[test]
    fn persona_not_in_run_refused() {
        let mut v = vi(vec![line()]);
        v.persona = "bob".into();
        assert_eq!(code(&v, &cx()), "persona_without_stake");
    }
    #[test]
    fn seen_line_without_citation_is_refused() {
        let mut c = cx();
        c.lens.dimensions.push(Dimension {
            id: "trust".into(),
            title: "Trust".into(),
            anchors: BTreeMap::from([("3".into(), "ok".into())]),
        });
        let mut l = line();
        l.dimension = "trust".into();
        l.citations.clear();
        let e = validate(&vi(vec![line(), l]), &c).unwrap_err();
        assert_eq!(e, Refusal::SeenWithoutCitation { line: 2 });
    }
    #[test]
    fn unresolved_citation_is_refused() {
        let mut l = line();
        l.citations = vec!["page#zzz".into()];
        assert_eq!(code(&vi(vec![l]), &cx()), "citation_unresolved");
    }
    #[test]
    fn compare_without_snapshot_refused_unless_recalled() {
        let mut l = line();
        l.evidence = Evidence::Told;
        l.citations.clear();
        l.about = Some("cmp:rival".into());
        assert_eq!(
            code(&vp(vec![l.clone()]), &about_cx()),
            "comparator_not_snapshotted"
        );
        l.evidence = Evidence::Recalled;
        assert!(validate(&vp(vec![l]), &about_cx()).is_ok());
    }
    #[test]
    fn comparator_citation_for_comparator_not_in_run_is_unresolved() {
        let mut l = line();
        l.citations = vec!["cmp:rival/page".into()];
        assert_eq!(code(&vi(vec![l]), &cx()), "citation_unresolved");
    }
    #[test]
    fn duplicate_dimension_refused() {
        let e = validate(&vi(vec![line(), line()]), &cx()).unwrap_err();
        assert_eq!(
            e,
            Refusal::DuplicateDimension {
                line: 2,
                dimension: "clarity".into(),
                about: None
            }
        );
        assert_eq!(e.code(), "duplicate_dimension");
    }
    #[test]
    fn missing_dimension_refused() {
        let mut c = cx();
        c.lens.dimensions.push(Dimension {
            id: "trust".into(),
            title: "Trust".into(),
            anchors: BTreeMap::new(),
        });
        let e = validate(&vi(vec![line()]), &c).unwrap_err();
        assert_eq!(
            e,
            Refusal::MissingDimension {
                dimension: "trust".into()
            }
        );
        assert_eq!(e.code(), "missing_dimension");
        assert_eq!(code(&vi(vec![]), &cx()), "missing_dimension");
    }
    #[test]
    fn score_outside_range_refused() {
        for s in [0, 6, -1] {
            let mut l = line();
            l.score = s;
            assert_eq!(code(&vi(vec![l]), &cx()), "score_out_of_range");
        }
    }
    #[test]
    fn unknown_dimension_refused() {
        let mut l = line();
        l.dimension = "nope".into();
        assert_eq!(code(&vi(vec![l]), &cx()), "unknown_dimension");
    }
    #[test]
    fn anchor_must_be_a_lens_anchor_at_or_below_score() {
        for (score, anchor) in [(3, 5), (3, 2), (1, 3)] {
            let mut l = line();
            l.score = score;
            l.anchor_met = anchor;
            assert_eq!(code(&vi(vec![l]), &cx()), "bad_anchor", "{score} {anchor}");
        }
        let mut l = line();
        l.score = 4;
        l.anchor_met = 3;
        assert!(validate(&vi(vec![l]), &cx()).is_ok());
    }
    #[test]
    fn recall_lens_takes_recalled_only() {
        let mut c = cx();
        c.lens = lens(false, &[Evidence::Recalled]);
        assert_eq!(code(&vi(vec![line()]), &c), "evidence_not_allowed");
        let mut l = line();
        l.evidence = Evidence::Recalled;
        l.citations.clear();
        assert!(validate(&vi(vec![l]), &c).is_ok());
    }
    #[test]
    fn dismissed_point_repeated_refused() {
        let mut l = line();
        l.blockers = vec!["again".into()];
        let h = crate::domain::citation::citation_hash(&["H1#n1".to_string()]);
        let mut c = cx();
        c.dismissed.insert(format!("clarity\t\t{h}"));
        assert_eq!(
            validate(&vi(vec![l.clone()]), &c).unwrap_err(),
            Refusal::DismissedPointRepeated {
                persona: "ana".into(),
                citation_hash: h
            }
        );
        // no blocker => passes even with the same citation
        l.blockers.clear();
        assert!(validate(&vi(vec![l]), &c).is_ok());
    }
    fn uncited_blocker_line(text: &str) -> VerdictLine {
        let mut l = line();
        l.evidence = Evidence::Told;
        l.citations.clear();
        l.blockers = vec![text.into()];
        l
    }
    fn uncited_id(text: &str) -> String {
        let h = crate::domain::citation::citation_hash(&[]);
        crate::domain::hash::finding_id("site", "ana", "clarity", None, text, &h)
    }
    #[test]
    fn uncited_dismissed_point_repeated_by_text() {
        let mut c = cx();
        c.dismissed.insert(uncited_id("feels cold"));
        let e = validate(&vi(vec![uncited_blocker_line("feels cold")]), &c).unwrap_err();
        assert_eq!(e.code(), "dismissed_point_repeated");
    }
    #[test]
    fn uncited_new_point_not_blocked_by_dismissed_uncited() {
        let mut c = cx();
        c.dismissed.insert(uncited_id("feels cold"));
        assert!(validate(&vi(vec![uncited_blocker_line("no search")]), &c).is_ok());
    }
    fn pick_cx() -> VerdictContext {
        let mut c = cx();
        c.lens = lens(true, &[Evidence::Seen]);
        with_rival(&mut c);
        c
    }
    /// the run holds comparator `rival` (attachment `rival`, node r1), and `cmp:rival` is an option
    fn with_rival(c: &mut VerdictContext) {
        let view = AttachmentView {
            hash: "R1".into(),
            media: "aria-json".into(),
            aria_ids: BTreeSet::from(["r1".to_string()]),
            lines: 0,
            ..Default::default()
        };
        c.evidence.comparators.insert(
            "rival".into(),
            BTreeMap::from([("rival".to_string(), view)]),
        );
        c.options.push("cmp:rival".into());
    }
    fn compare_cx() -> VerdictContext {
        let mut c = cx();
        c.lens = lens(false, &[Evidence::Seen, Evidence::Told, Evidence::Recalled]);
        c.lens.compares = true;
        with_rival(&mut c);
        c
    }
    #[test]
    fn one_line_per_dimension_and_about() {
        let mut c = about_cx();
        with_rival(&mut c);
        let mut on_page = line();
        on_page.about = Some("page".into());
        let mut on_rival = line();
        on_rival.about = Some("cmp:rival".into());
        let ok = validate(&vp(vec![line(), on_page.clone(), on_rival]), &c).unwrap();
        assert_eq!(ok.lines.len(), 3);
        let e = validate(&vp(vec![on_page.clone(), on_page]), &c).unwrap_err();
        assert_eq!(e.code(), "duplicate_dimension");
        let msg = e.to_string();
        assert!(msg.contains("clarity") && msg.contains("page"), "{msg}");
        // every lens dimension still needs a line, whatever the lines are about
        c.lens.dimensions.push(Dimension {
            id: "trust".into(),
            title: "Trust".into(),
            anchors: BTreeMap::new(),
        });
        let mut l = line();
        l.about = Some("page".into());
        assert_eq!(code(&vp(vec![line(), l]), &c), "missing_dimension");
    }
    #[test]
    fn about_refused_under_a_lens_without_pick_or_compare() {
        let mut l = line();
        l.about = Some("page".into());
        let e = validate(&vi(vec![l]), &cx()).unwrap_err();
        assert_eq!(
            e,
            Refusal::UnknownAbout {
                line: 1,
                about: "page".into()
            }
        );
        assert!(e.to_string().contains("leave `about` out"), "{e}");
        let mut l = line();
        l.about = Some("page".into());
        assert!(validate(&vp(vec![l]), &about_cx()).is_ok());
    }
    #[test]
    fn unknown_about_refused() {
        for about in ["nowhere", "cmp:", "cmp:a/b", "page#n1", ""] {
            let mut l = line();
            l.about = Some(about.into());
            let e = validate(&vp(vec![l]), &about_cx()).unwrap_err();
            assert_eq!(
                e,
                Refusal::UnknownAbout {
                    line: 1,
                    about: about.into()
                },
                "{about}"
            );
            assert_eq!(e.code(), "unknown_about");
        }
        let mut l = line();
        l.about = Some("page".into());
        assert!(validate(&vp(vec![l]), &about_cx()).is_ok());
    }
    #[test]
    fn about_cmp_not_held_must_be_recalled() {
        let mut l = line();
        l.about = Some("cmp:ghost".into());
        assert_eq!(
            validate(&vp(vec![l.clone()]), &about_cx()).unwrap_err(),
            Refusal::ComparatorNotSnapshotted {
                line: 1,
                comparator: "ghost".into()
            }
        );
        l.evidence = Evidence::Told;
        l.citations.clear();
        assert_eq!(
            code(&vp(vec![l.clone()]), &about_cx()),
            "comparator_not_snapshotted"
        );
        l.evidence = Evidence::Recalled;
        assert!(validate(&vp(vec![l]), &about_cx()).is_ok());
    }
    #[test]
    fn compare_seen_line_needs_a_comparator() {
        let e = validate(&vi(vec![line()]), &compare_cx()).unwrap_err();
        assert_eq!(e, Refusal::SeenCompareWithoutComparator { line: 1 });
        assert_eq!(e.code(), "seen_compare_without_comparator");
        // about a held comparator: passes
        let mut l = line();
        l.about = Some("cmp:rival".into());
        assert!(validate(&vi(vec![l]), &compare_cx()).is_ok());
        // a told line is not a seen comparison
        let mut l = line();
        l.evidence = Evidence::Told;
        l.citations.clear();
        assert!(validate(&vi(vec![l]), &compare_cx()).is_ok());
        // about the subject's own attachment and citing it only: still nothing compared
        let mut l = line();
        l.about = Some("page".into());
        assert_eq!(
            code(&vi(vec![l]), &compare_cx()),
            "seen_compare_without_comparator"
        );
        // a lens that does not compare does not ask
        assert!(validate(&vi(vec![line()]), &cx()).is_ok());
    }
    #[test]
    fn compare_seen_line_with_cmp_citation_passes() {
        let mut l = line();
        l.citations = vec!["page#n1".into(), "cmp:rival/rival#r1".into()];
        let ok = validate(&vi(vec![l]), &compare_cx()).unwrap();
        assert_eq!(ok.lines[0].resolved, vec!["H1#n1", "R1#r1"]);
    }
    #[test]
    fn bottom_score_without_blocker_refused() {
        let mut l = line();
        l.score = 1;
        l.anchor_met = 1;
        let e = validate(&vi(vec![l.clone()]), &cx()).unwrap_err();
        assert_eq!(e, Refusal::BottomScoreWithoutBlocker { line: 1 });
        assert_eq!(e.code(), "bottom_score_without_blocker");
        assert_eq!(
            e.to_string(),
            "line 1: the bottom score needs a blocker that says what failed"
        );
        l.blockers = vec!["the price is nowhere".into()];
        assert_eq!(validate(&vi(vec![l]), &cx()).unwrap().findings.len(), 1);
        // one above the bottom needs no blocker
        let mut l = line();
        l.score = 2;
        l.anchor_met = 1;
        assert!(validate(&vi(vec![l]), &cx()).is_ok());
    }
    #[test]
    fn score_all_options_then_pick() {
        // three logo files: the persona scores each and picks one
        let mut c = cx();
        c.lens = lens(true, &[Evidence::Seen]);
        for (label, h) in [("a.png", "A"), ("b.png", "B")] {
            c.evidence.attachments.insert(
                label.into(),
                AttachmentView {
                    hash: h.into(),
                    media: "image".into(),
                    ..Default::default()
                },
            );
        }
        c.options = vec!["a.png".into(), "b.png".into(), "page".into()];
        let lines: Vec<VerdictLine> = ["a.png", "b.png", "page"]
            .iter()
            .map(|o| {
                let mut l = line();
                l.about = Some(o.to_string());
                l.citations = vec![o.to_string()];
                l
            })
            .collect();
        let mut v = vi(lines);
        v.pick = Some(PickInput {
            choice: "b.png".into(),
            reason: "it reads at a glance".into(),
        });
        let ok = validate(&v, &c).unwrap();
        assert_eq!(ok.lines.len(), 3);
        assert_eq!(ok.pick.unwrap().choice, "b.png");
    }
    #[test]
    fn same_point_keys_on_dimension() {
        let mut c = cx();
        c.lens.dimensions.push(Dimension {
            id: "trust".into(),
            title: "Trust".into(),
            anchors: BTreeMap::from([("3".into(), "ok".into())]),
        });
        let h = crate::domain::citation::citation_hash(&["H1#n1".to_string()]);
        c.dismissed.insert(format!("clarity\t\t{h}"));
        let mut clarity = line();
        clarity.blockers = vec!["the hero says nothing".into()];
        let mut trust = line();
        trust.dimension = "trust".into();
        trust.blockers = vec!["the hero has no contrast".into()];
        // the same node under another dimension is another point
        let mut quiet = line();
        quiet.blockers.clear();
        assert!(validate(&vi(vec![quiet, trust.clone()]), &c).is_ok());
        // the same node under the dismissed dimension is the same point, however worded
        assert_eq!(
            code(&vi(vec![clarity, trust]), &c),
            "dismissed_point_repeated"
        );
    }
    #[test]
    fn pick_missing_when_lens_asks() {
        assert_eq!(code(&vi(vec![line()]), &pick_cx()), "pick_missing");
    }
    #[test]
    fn pick_without_reason_refused() {
        let mut v = vi(vec![line()]);
        v.pick = Some(PickInput {
            choice: "page".into(),
            reason: "  \n".into(),
        });
        assert_eq!(code(&v, &pick_cx()), "pick_without_reason");
    }
    #[test]
    fn pick_unknown_option_refused() {
        let mut v = vi(vec![line()]);
        v.pick = Some(PickInput {
            choice: "other".into(),
            reason: "because".into(),
        });
        let e = validate(&v, &pick_cx()).unwrap_err();
        assert_eq!(
            e,
            Refusal::PickUnknownOption {
                choice: "other".into(),
                options: "page, cmp:rival".into()
            }
        );
        v.pick = Some(PickInput {
            choice: "cmp:rival".into(),
            reason: "because".into(),
        });
        assert!(validate(&v, &pick_cx()).unwrap().pick.is_some());
    }
    #[test]
    fn pick_refused_when_lens_does_not_ask() {
        let mut v = vi(vec![line()]);
        v.pick = Some(PickInput {
            choice: "anything".into(),
            reason: "why".into(),
        });
        assert_eq!(code(&v, &cx()), "pick_not_asked");
    }
    #[test]
    fn blank_blockers_are_no_blockers() {
        let mut l = line();
        l.blockers = vec!["  ".into(), "".into()];
        let ok = validate(&vi(vec![l.clone()]), &cx()).unwrap();
        assert!(ok.findings.is_empty());
        assert!(ok.lines[0].line.blockers.is_empty());
        l.score = 1;
        l.anchor_met = 1;
        assert_eq!(code(&vi(vec![l]), &cx()), "bottom_score_without_blocker");
    }
    #[test]
    fn anchor_met_is_an_anchor_of_its_own_dimension() {
        let mut c = cx();
        c.lens.dimensions.push(Dimension {
            id: "trust".into(),
            title: "Trust".into(),
            anchors: BTreeMap::from([("1".into(), "bad".into()), ("4".into(), "ok".into())]),
        });
        let mut trust = line();
        trust.dimension = "trust".into();
        trust.score = 4;
        trust.anchor_met = 3; // an anchor of clarity, not of trust
        assert_eq!(code(&vi(vec![line(), trust.clone()]), &c), "bad_anchor");
        trust.anchor_met = 4;
        assert!(validate(&vi(vec![line(), trust]), &c).is_ok());
    }
    #[test]
    fn about_is_part_of_the_finding_and_the_dismissed_key() {
        let mut c = about_cx();
        with_rival(&mut c);
        let h = crate::domain::citation::citation_hash(&["H1#n1".to_string()]);
        c.dismissed
            .insert(dismissed_key("clarity", Some("page"), &h));
        let on = |about: &str| {
            let mut l = line();
            l.about = Some(about.into());
            l.blockers = vec!["the price is hidden".into()];
            l
        };
        // the same text and citations on another option is another point
        let ok = validate(&vp(vec![line(), on("cmp:rival")]), &c).unwrap();
        assert_eq!(ok.findings[0].about.as_deref(), Some("cmp:rival"));
        let whole = crate::domain::hash::finding_id(
            "site",
            "ana",
            "clarity",
            None,
            "the price is hidden",
            &h,
        );
        assert_ne!(ok.findings[0].id, whole);
        // the dismissed option is refused
        assert_eq!(
            code(&vp(vec![line(), on("page")]), &c),
            "dismissed_point_repeated"
        );
    }
    #[test]
    fn resolved_is_sorted() {
        let mut c = cx();
        let mut l = line();
        c.evidence.attachments.insert(
            "aaa".into(),
            AttachmentView {
                hash: "A0".into(),
                media: "image".into(),
                ..Default::default()
            },
        );
        l.citations = vec!["page#n1".into(), "aaa#x".into()];
        assert_eq!(
            validate(&vi(vec![l]), &c).unwrap().lines[0].resolved,
            vec!["A0#x", "H1#n1"]
        );
    }
}
