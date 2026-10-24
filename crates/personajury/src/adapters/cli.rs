//! adapted from comparegraph crates/comparegraph/src/adapters/cli.rs (Cli, --root, run, one envelope per call)
//! The CLI: thin. Every verb parses, calls one application function and prints one envelope.
use crate::adapters::envelope::Envelope;
use crate::application::{self as app, AppError};
use crate::domain::{Detail, Effort, Independence, Stake, SubjectKind};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "personajury",
    version,
    about = "A persona jury as an engine: evidence on every line, strata never averaged. The engine never calls a model."
)]
pub struct Cli {
    #[arg(long, global = true, default_value = ".")]
    pub root: PathBuf,
    #[arg(long, global = true)]
    pub pretty: bool,
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
// one variant per verb: `brief` carries the most flags, and a Cmd lives for one call
#[allow(clippy::large_enum_variant)]
pub enum Cmd {
    /// Personas: add one from the fixed JSON, write the brief that asks an agent to generate them, list them
    Persona {
        #[command(subcommand)]
        what: PersonaCmd,
    },
    /// Traits: the catalog of tendencies a persona may hold (49 ship, embedded; install them once), and trait sets
    Trait {
        #[command(subcommand)]
        what: TraitCmd,
    },
    /// Panels: a saved query over persona tags ("5 builder, 3 company, 2 others")
    Panel {
        #[command(subcommand)]
        what: PanelCmd,
    },
    /// Lenses: dimension sets with anchors in words (five ship: critical, taste, compare, recall, decide)
    Lens {
        #[command(subcommand)]
        what: LensCmd,
    },
    /// Subjects: what is judged, as attached files (an ARIA walk, screenshots, a diff, a document); never fetched
    Subject {
        #[command(subcommand)]
        what: SubjectCmd,
    },
    /// Comparators: the same shape as subjects, flagged; a run holds their snapshots
    Comparator {
        #[command(subcommand)]
        what: SubjectCmd,
    },
    /// Compose one deterministic, hashed brief per persona for a lens; creates the run
    Brief {
        #[arg(long)]
        subject: String,
        /// a saved panel (or `--query`, or `--rule` for the rule's panel)
        #[arg(long)]
        panel: Option<String>,
        /// an ad-hoc panel, not saved: "3 builder=primary, 2 others=bystander"
        #[arg(long)]
        query: Option<String>,
        /// one run per lens, repeated or comma-separated (default: the rule's)
        #[arg(long, value_delimiter = ',')]
        lens: Vec<String>,
        /// take the first N personas of the panel, primary → secondary → bystander (default: all)
        #[arg(long)]
        spend: Option<usize>,
        /// `<persona>=primary|secondary|bystander`, repeated or comma-separated: wins over the panel's term stake, and
        /// is needed for a persona whose term has none
        #[arg(long = "stake", value_delimiter = ',')]
        stakes: Vec<String>,
        /// the model the agent will run the personas on (the run is pinned to it)
        #[arg(long)]
        model: String,
        /// comparator snapshots this run holds
        #[arg(long = "comparator", value_delimiter = ',')]
        comparators: Vec<String>,
        /// compact (attachments by path) or full (text attachments inline); default: the rule's, else compact
        #[arg(long)]
        detail: Option<String>,
        /// take panel, lenses, spend and detail from this rule of personajury.toml; any value you change needs --reason
        #[arg(long)]
        rule: Option<String>,
        /// the draw's seed as 16 hex digits (default: from the subject's name and the panel's)
        #[arg(long)]
        draw: Option<String>,
        /// allow more than 12 seats (the most is 1000)
        #[arg(long)]
        allow_large: bool,
        /// why this brief departs from its rule (or runs more than 12 seats); recorded on the run
        #[arg(long)]
        reason: Option<String>,
        /// who departed from the rule (default: agent)
        #[arg(long)]
        by: Option<String>,
        /// seats per wave in the answer's `waves` (default 30): run one wave, ingest it, then the next
        #[arg(long)]
        wave: Option<usize>,
        /// how much this run may spend: low | medium | high | max (default: the toml's, else medium under a rule); recorded
        #[arg(long)]
        effort: Option<String>,
        /// independent (one context per brief, the default) or sequential (one context reads every brief)
        #[arg(long)]
        independence: Option<String>,
        /// render the personas' traits as "you tend to" lines: a trait set's name, `all`, or `off` (default: the rule's trait_set, else off)
        #[arg(long)]
        traits: Option<String>,
        /// `auto`: seat every persona whose `serves` meets the subject's audience (capped by effort), plus one control that does not
        #[arg(long)]
        seats: Option<String>,
        /// what a juror may score `about` and pick: `subject` (the subject and each comparator) or `attachments` (its
        /// files and each comparator); default by kind: design and logo attachments, every other kind subject
        #[arg(long)]
        options: Option<String>,
    },
    /// Ingest one persona's verdict JSON for a run
    Verdict {
        #[arg(long)]
        run: i64,
        #[arg(long, conflicts_with = "json")]
        file: Option<PathBuf>,
        #[arg(long)]
        json: Option<String>,
        /// a person entered this verdict: "<name>, <credential>". The file's persona is the seat whose stratum the
        /// person speaks in; the rows stand beside the jury's, never in its cells. Only a person enters one.
        #[arg(long)]
        by_human: Option<String>,
        /// the advise-only round (D33): the file carries "phase": "deliberated"; needs `deliberate` first
        #[arg(long)]
        deliberated: bool,
    },
    /// One advise-only round after the first verdicts: each juror reads what the others saw and may change its verdict (measures persuasion, decides nothing)
    Deliberate {
        run: i64,
        /// seats per wave in the answer's `waves` (default 30)
        #[arg(long)]
        wave: Option<usize>,
    },
    /// Scores per stratum for one or more runs; never averaged across strata
    Matrix {
        #[arg(required = true)]
        runs: Vec<i64>,
        /// refused: a table that averages across strata (report §5); the flag exists so the refusal is explicit
        #[arg(long)]
        across_strata: bool,
        #[arg(long)]
        history: bool,
        /// add the markdown table as data.table
        #[arg(long)]
        table: bool,
        #[arg(long, default_value = "compact")]
        detail: String,
        /// also group each stratum's scores by this trait's bands (counts per group, never a mean, never across strata)
        #[arg(long)]
        slice: Option<String>,
        /// initial (default: the jury's first word), deliberated (the advise-only round) or both (under `phases`)
        #[arg(long, default_value = "initial")]
        phase: String,
    },
    /// The forced choices of a run, per stratum, with reasons
    Pick { run: i64 },
    /// Draw the jury as one static SVG: personas by stake, the dimensions they scored, the findings; several runs draw each finding's life
    Graph {
        #[arg(required = true)]
        runs: Vec<i64>,
        /// the only format for now
        #[arg(long)]
        svg: bool,
        /// write the SVG to this file instead of answering it
        #[arg(long)]
        out: Option<PathBuf>,
        /// initial (default) or deliberated: which batch of each persona is drawn
        #[arg(long, default_value = "initial")]
        phase: String,
    },
    /// What moved between two runs on the same subject and lens
    Delta {
        a: i64,
        b: i64,
        /// initial (default) or deliberated: which batch of each run is read
        #[arg(long, default_value = "initial")]
        phase: String,
    },
    /// Stale runs (the subject moved) and findings with no decision; exits 1 on an unresolved finding
    Check {
        #[arg(long)]
        history: bool,
    },
    /// The verdict to obey last: apply | escalate | human | advise, with the reasons as data; applies nothing itself
    ApplyCheck {
        runs: Vec<i64>,
        /// instead of naming runs: every subject that has runs, each judged on its own; the worst answer wins (what CI reads)
        #[arg(long)]
        latest: bool,
        /// with --latest: only this subject
        #[arg(long)]
        subject: Option<String>,
        /// the rule class of personajury.toml this change falls under (its quorum is the current level)
        #[arg(long)]
        rule: Option<String>,
        /// a file written by `calibrate --out`: `calibrated: false` turns apply into advise (`uncalibrated`)
        #[arg(long)]
        calibration: Option<PathBuf>,
    },
    /// Set the jury's rank order beside a human run's, per stratum (a human run holds `verdict --by-human` batches)
    Calibrate {
        jury_run: i64,
        human_run: i64,
        /// also write the answer to this file, for `apply-check --calibration`
        #[arg(long)]
        out: Option<PathBuf>,
        /// initial (default) or deliberated: which batch of the jury is set beside the people
        #[arg(long, default_value = "initial")]
        phase: String,
    },
    /// The answer display: sentences, the numbers "for checking", then Next; --html / --md / --save write views
    Summary {
        #[arg(required = true)]
        runs: Vec<i64>,
        /// one self-contained html file (inline CSS, the jury graph, every per-stratum table)
        #[arg(long)]
        html: Option<PathBuf>,
        /// the same summary as markdown, with the decisions per finding, the run ids and brief hashes, and the date
        #[arg(long)]
        md: Option<PathBuf>,
        /// write the markdown to <report_dir>/<date>-<subject>.md (refused when personajury.toml has no report_dir)
        #[arg(long)]
        save: bool,
        /// the text's width in characters (30 to 200, default 80)
        #[arg(long)]
        width: Option<usize>,
        /// set by the MCP adapter: --html/--md must be relative paths inside the root (I2); a typed CLI path may be anywhere
        #[arg(skip)]
        confined: bool,
        /// after the sentences, one sentence per stratum on how the jurors of each band of this trait scored the weakest dimension
        #[arg(long)]
        slice: Option<String>,
    },
    /// A person's word on a finding: fixed, dismissed (with --why) or row (with --row)
    Decide {
        finding: String,
        outcome: String,
        #[arg(long)]
        why: Option<String>,
        #[arg(long)]
        row: Option<String>,
        /// who decided (the foreman, an agent, a person)
        #[arg(long)]
        by: Option<String>,
        /// a person's name: required to decide a finding a human batch raised (D19)
        #[arg(long)]
        by_human: Option<String>,
    },
    /// Classify a diff against personajury.toml: none | low | high | human, and the spend
    ShouldRun {
        #[arg(long)]
        diff: Option<PathBuf>,
        #[arg(long, value_delimiter = ',')]
        paths: Vec<String>,
        #[arg(long)]
        config: Option<PathBuf>,
        /// classify the recorded source paths of this subject (a page added by hand has no diff)
        #[arg(long)]
        subject: Option<String>,
    },
    /// Register the MCP server with agent clients: a plan by default, `--apply` writes (additive, backed up)
    Setup {
        #[arg(long, default_value = "all")]
        client: String,
        #[arg(long, default_value = "user")]
        scope: String,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        bin: Option<String>,
        #[arg(long)]
        no_rules: bool,
        #[arg(long)]
        no_hook: bool,
    },
    /// Write a starter personajury.toml (human rules active for prices, legal, auth and secrets; high for screens, pages and docs; no jury for code); never overwrites
    Init,
    /// Report the store, the counts and each agent client's registration
    Doctor {
        #[arg(long)]
        bin: Option<String>,
        /// one or two plain lines for a session-start hook (not an envelope)
        #[arg(long)]
        brief: bool,
    },
    /// Serve MCP over stdio
    Mcp,
}

#[derive(Subcommand, Debug)]
pub enum PersonaCmd {
    Add {
        #[arg(long)]
        file: PathBuf,
        /// the material hash a `persona generate` brief recorded
        #[arg(long)]
        generated_from: Option<String>,
    },
    Generate {
        #[arg(long)]
        from: PathBuf,
        #[arg(long, default_value_t = 7)]
        count: usize,
        /// a trait set: the brief also lists its traits and gives each persona a Latin-square row of cover bands
        #[arg(long)]
        traits: Option<String>,
    },
    List,
}
#[derive(Subcommand, Debug)]
pub enum TraitCmd {
    /// Add every embedded catalog trait and set the store lacks; an earlier catalog version is updated; a local edit is
    /// kept (`kept_local`) unless --force
    Install {
        /// overwrite local edits with the catalog's text (needs --reason, which the new version records)
        #[arg(long)]
        force: bool,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Add one trait file (traits/<family>/<trait>.toml)
    Add {
        #[arg(long)]
        file: PathBuf,
        /// the material hash a generated trait came from
        #[arg(long)]
        generated_from: Option<String>,
    },
    List,
    Show { id: String },
    /// Write the brief that asks the agent to draft trait files from a material (D35); no model is called
    Generate {
        #[arg(long)]
        from: PathBuf,
        /// how many trait files to ask for (1 to 8)
        #[arg(long, default_value_t = 3)]
        count: usize,
    },
    /// From a juror's own words: write the brief (--persona), or verify the answer (--check FILE); no model is called (D35)
    Suggest {
        #[arg(long)]
        persona: Option<String>,
        /// the candidates are this trait set's traits; without it, every trait in the store
        #[arg(long)]
        set: Option<String>,
        /// verify an answer file; writes nothing, answers the merged persona file as data
        #[arg(long)]
        check: Option<PathBuf>,
    },
    /// What a persona did in its accepted initial verdicts, counted, and what that bears on the traits it declares (D35)
    Observed {
        #[arg(long = "persona", required = true)]
        persona: Vec<String>,
        /// run ids separated by commas; default every run the persona sat in
        #[arg(long)]
        runs: Option<String>,
    },
    /// Trait sets: up to 8 traits an audience is judged by, up to 4 of them the cover of a panel
    Set {
        #[command(subcommand)]
        what: TraitSetCmd,
    },
}
#[derive(Subcommand, Debug)]
pub enum TraitSetCmd {
    Add {
        #[arg(long)]
        file: PathBuf,
    },
    List,
}
#[derive(Subcommand, Debug)]
pub enum PanelCmd {
    Add {
        name: String,
        /// "3 builder=primary, 1 newcomer=primary, 2 approver=secondary, 1 others=bystander"; a stake per term is optional
        #[arg(long)]
        query: String,
        /// allow more than 12 seats (the most is 1000)
        #[arg(long)]
        allow_large: bool,
        /// up to 4 traits the seats must cover across their bands (the draw picks seats that add new band cells)
        #[arg(long, value_delimiter = ',')]
        cover: Vec<String>,
    },
    Resolve {
        name: String,
        /// draw the seats for this subject (the seed comes from its name); without it the seats are the first by id
        #[arg(long)]
        subject: Option<String>,
    },
}
#[derive(Subcommand, Debug)]
pub enum LensCmd {
    Add {
        #[arg(long)]
        file: PathBuf,
    },
    /// Write the brief that asks an agent to draft a lens from a standard, a handbook or examples (no model is called)
    Generate {
        /// a file or a folder of the material the lens is drawn from
        #[arg(long)]
        from: PathBuf,
        /// how many dimensions the lens has (1 to 12)
        #[arg(long, default_value_t = 4)]
        dimensions: usize,
    },
    List,
}
#[derive(Subcommand, Debug)]
pub enum SubjectCmd {
    Add {
        name: String,
        /// page | diff | design | logo | document | menu | prices | decision (a questions.md plus the files a point of view is read from)
        #[arg(long)]
        kind: String,
        #[arg(long, num_args = 1.., required = true)]
        attach: Vec<PathBuf>,
        /// who the subject speaks to, as tags (for `brief --seats auto`): "docs,developers"
        #[arg(long, value_delimiter = ',')]
        audience: Vec<String>,
    },
}

pub fn verb(cmd: &Cmd) -> &'static str {
    match cmd {
        Cmd::Persona { what: PersonaCmd::Add { .. } } => "persona add",
        Cmd::Persona { what: PersonaCmd::Generate { .. } } => "persona generate",
        Cmd::Persona { what: PersonaCmd::List } => "persona list",
        Cmd::Trait { what: TraitCmd::Install { .. } } => "trait install",
        Cmd::Trait { what: TraitCmd::Add { .. } } => "trait add",
        Cmd::Trait { what: TraitCmd::List } => "trait list",
        Cmd::Trait { what: TraitCmd::Show { .. } } => "trait show",
        Cmd::Trait { what: TraitCmd::Generate { .. } } => "trait generate",
        Cmd::Trait { what: TraitCmd::Suggest { .. } } => "trait suggest",
        Cmd::Trait { what: TraitCmd::Observed { .. } } => "trait observed",
        Cmd::Trait { what: TraitCmd::Set { what: TraitSetCmd::Add { .. } } } => "trait set add",
        Cmd::Trait { what: TraitCmd::Set { what: TraitSetCmd::List } } => "trait set list",
        Cmd::Panel { what: PanelCmd::Add { .. } } => "panel add",
        Cmd::Panel { what: PanelCmd::Resolve { .. } } => "panel resolve",
        Cmd::Lens { what: LensCmd::Add { .. } } => "lens add",
        Cmd::Lens { what: LensCmd::Generate { .. } } => "lens generate",
        Cmd::Lens { what: LensCmd::List } => "lens list",
        Cmd::Subject { .. } => "subject add",
        Cmd::Comparator { .. } => "comparator add",
        Cmd::Brief { .. } => "brief",
        Cmd::Verdict { .. } => "verdict",
        Cmd::Deliberate { .. } => "deliberate",
        Cmd::Matrix { .. } => "matrix",
        Cmd::Graph { .. } => "graph",
        Cmd::Pick { .. } => "pick",
        Cmd::Delta { .. } => "delta",
        Cmd::Check { .. } => "check",
        Cmd::ApplyCheck { .. } => "apply-check",
        Cmd::Summary { .. } => "summary",
        Cmd::Calibrate { .. } => "calibrate",
        Cmd::Decide { .. } => "decide",
        Cmd::ShouldRun { .. } => "should-run",
        Cmd::Setup { .. } => "setup",
        Cmd::Init => "init",
        Cmd::Doctor { .. } => "doctor",
        Cmd::Mcp => "mcp",
    }
}

fn detail(s: &str) -> app::Result<Detail> {
    Detail::parse(s).ok_or_else(|| AppError::InvalidArguments(format!("unknown detail {s} (compact|full)")))
}
fn parse_effort(s: &str) -> app::Result<Effort> {
    Effort::parse(s).ok_or_else(|| AppError::InvalidArguments(format!("unknown effort {s} (low|medium|high|max)")))
}
fn parse_independence(s: &str) -> app::Result<Independence> {
    Independence::parse(s)
        .ok_or_else(|| AppError::InvalidArguments(format!("unknown independence {s} (independent|sequential)")))
}
fn kind(s: &str) -> app::Result<SubjectKind> {
    SubjectKind::parse(s).ok_or_else(|| {
        let all: Vec<_> = SubjectKind::ALL.iter().map(|k| k.name()).collect();
        AppError::InvalidArguments(format!("unknown kind {s} ({})", all.join("|")))
    })
}
/// `<persona>=<stake>` pairs; a persona named twice is refused (it would keep the last stake silently).
pub fn parse_stakes(v: &[String]) -> app::Result<Vec<(String, Stake)>> {
    let mut out: Vec<(String, Stake)> = Vec::new();
    for s in v.iter().filter(|s| !s.trim().is_empty()) {
        let (p, k) = s
            .split_once('=')
            .ok_or_else(|| AppError::InvalidArguments(format!("--stake {s}: expected <persona>=<stake>")))?;
        let k = Stake::parse(k.trim()).ok_or_else(|| {
            AppError::InvalidArguments(format!("--stake {s}: stake is primary, secondary or bystander"))
        })?;
        let p = p.trim().to_string();
        if out.iter().any(|(q, _)| *q == p) {
            return Err(AppError::InvalidArguments(format!("--stake names {p} twice; one stake per persona")));
        }
        out.push((p, k));
    }
    Ok(out)
}

fn dispatch(cli: &Cli) -> app::Result<serde_json::Value> {
    let root = cli.root.as_path();
    match &cli.cmd {
        Cmd::Persona { what } => match what {
            PersonaCmd::Add { file, generated_from } => app::persona::add(root, file, generated_from.as_deref()),
            PersonaCmd::Generate { from, count, traits } => app::persona::generate(root, from, *count, traits.as_deref()),
            PersonaCmd::List => app::persona::list(root),
        },
        Cmd::Trait { what } => match what {
            TraitCmd::Install { force, reason } => app::traits::install(root, *force, reason.as_deref()),
            TraitCmd::Add { file, generated_from } => app::traits::add(root, file, generated_from.as_deref()),
            TraitCmd::List => app::traits::list(root),
            TraitCmd::Show { id } => app::traits::show(root, id),
            TraitCmd::Generate { from, count } => app::traits::generate(root, from, *count),
            TraitCmd::Observed { persona, runs } => app::trait_reverse::observed(root, persona, runs.as_deref()),
            TraitCmd::Suggest { persona, set, check } => match check {
                Some(f) => app::trait_reverse::suggest_check(root, f, persona.as_deref()),
                None => app::trait_reverse::suggest(root, persona.as_deref(), set.as_deref()),
            },
            TraitCmd::Set { what: TraitSetCmd::Add { file } } => app::traits::set_add(root, file),
            TraitCmd::Set { what: TraitSetCmd::List } => app::traits::set_list(root),
        },
        Cmd::Panel { what } => match what {
            PanelCmd::Add { name, query, allow_large, cover } => app::panel::add(root, name, query, *allow_large, cover),
            PanelCmd::Resolve { name, subject } => app::panel::resolve(root, name, subject.as_deref()),
        },
        Cmd::Lens { what } => match what {
            LensCmd::Add { file } => app::lens::add(root, file),
            LensCmd::Generate { from, dimensions } => app::lens::generate(root, from, *dimensions),
            LensCmd::List => app::lens::list(root),
        },
        Cmd::Subject { what: SubjectCmd::Add { name, kind: k, attach, audience } } => {
            app::subject::add(root, name, kind(k)?, attach, false, audience)
        }
        Cmd::Comparator { what: SubjectCmd::Add { name, kind: k, attach, audience } } => {
            app::subject::add(root, name, kind(k)?, attach, true, audience)
        }
        Cmd::Brief {
            subject, panel, query, lens, spend, stakes, model, comparators, detail: d, rule, draw, allow_large, reason, by, wave, effort, independence, traits, seats, options,
        } => app::brief::compose(
            root,
            &app::brief::BriefArgs {
                subject: subject.clone(),
                panel: panel.clone(),
                query: query.clone(),
                lenses: lens.clone(),
                spend: *spend,
                stakes: parse_stakes(stakes)?,
                model: model.clone(),
                comparators: comparators.clone(),
                detail: d.as_deref().map(detail).transpose()?,
                rule: rule.clone(),
                draw: draw.clone(),
                allow_large: *allow_large,
                reason: reason.clone(),
                by: by.clone(),
                wave: *wave,
                effort: effort.as_deref().map(parse_effort).transpose()?,
                independence: independence.as_deref().map(parse_independence).transpose()?.unwrap_or_default(),
                traits: traits.clone(),
                seats: seats.clone(),
                options: options.clone(),
            },
        ),
        Cmd::Deliberate { run, wave } => app::deliberate::run(root, *run, *wave),
        Cmd::Verdict { run, file, json, by_human, deliberated } => {
            let input = match (file, json) {
                (Some(f), None) => app::read_text(f)?,
                (None, Some(j)) => j.clone(),
                _ => return Err(AppError::InvalidArguments("give the verdict as --file <path> or --json <text>".into())),
            };
            app::verdict::ingest_with(root, *run, &input, by_human.as_deref(), *deliberated)
        }
        Cmd::Matrix { runs, across_strata, history, table, detail: d, slice, phase } => {
            app::matrix::show(root, runs, *across_strata, *history, *table, detail(d)?, slice.as_deref(), phase)
        }
        Cmd::Graph { runs, svg, out, phase } => app::graph::show(root, runs, *svg, out.as_deref(), phase),
        Cmd::Pick { run } => app::pick::show(root, *run),
        Cmd::Delta { a, b, phase } => app::delta::show_phase(root, *a, *b, phase),
        Cmd::Check { history } => app::check::run(root, *history),
        Cmd::ApplyCheck { runs, latest, subject, rule, calibration } => {
            let groups = app::apply_check::runs_to_judge(root, runs, *latest, subject.as_deref())?;
            if *latest {
                app::apply_check::worst(root, &groups, rule.as_deref(), calibration.as_deref())
            } else {
                app::apply_check::run(root, &groups[0], rule.as_deref(), calibration.as_deref())
            }
        }
        Cmd::Calibrate { jury_run, human_run, out, phase } => app::calibrate::show(root, *jury_run, *human_run, out.as_deref(), phase),
        Cmd::Summary { runs, html, md, save, width, confined, slice } => {
            app::summary::show(root, runs, html.as_deref(), md.as_deref(), *save, *width, *confined, slice.as_deref())
        }
        Cmd::Decide { finding, outcome, why, row, by, by_human } => app::decide::record(
            root,
            finding,
            outcome,
            why.as_deref(),
            row.as_deref(),
            by.as_deref(),
            by_human.as_deref(),
        ),
        Cmd::ShouldRun { diff, paths, config, subject } => {
            app::should_run::classify(root, diff.as_deref(), paths, config.as_deref(), subject.as_deref())
        }
        Cmd::Setup { client, scope, apply, force, bin, no_rules, no_hook } => app::setup::run(
            root,
            &app::setup::SetupArgs {
                client: client.clone(),
                scope: scope.clone(),
                apply: *apply,
                force: *force,
                bin: bin.clone(),
                no_rules: *no_rules,
                no_hook: *no_hook,
            },
        ),
        Cmd::Init => app::init::run(root),
        Cmd::Doctor { bin, .. } => app::doctor::report(root, bin.as_deref()),
        Cmd::Mcp => unreachable!("mcp is served by run()"),
    }
}

/// The envelope for one parsed call; the MCP server calls this too.
pub fn envelope_for(cli: &Cli) -> Envelope {
    let v = verb(&cli.cmd);
    match dispatch(cli) {
        Ok(data) => Envelope::ok(v, data),
        Err(e) => Envelope::err(v, &e),
    }
}

pub fn run() -> i32 {
    let cli = Cli::parse();
    if matches!(cli.cmd, Cmd::Mcp) {
        crate::adapters::mcp::serve(&cli.root, std::io::stdin().lock(), std::io::stdout().lock());
        return 0;
    }
    if let Cmd::Doctor { bin, brief: true } = &cli.cmd {
        print!("{}", app::doctor::brief_lines(&cli.root, bin.as_deref()));
        return 0;
    }
    let env = envelope_for(&cli);
    let json = env.to_json();
    if cli.pretty {
        let v: serde_json::Value = serde_json::from_str(&json).expect("envelope is JSON");
        println!("{}", serde_json::to_string_pretty(&v).expect("serialises"));
    } else {
        println!("{json}");
    }
    if env.ok {
        0
    } else {
        1
    }
}
