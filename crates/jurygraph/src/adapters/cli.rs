//! adapted from comparegraph crates/comparegraph/src/adapters/cli.rs (Cli, --root, run, one envelope per call)
//! The CLI: thin. Every verb parses, calls one application function and prints one envelope.
use crate::adapters::envelope::Envelope;
use crate::application::{self as app, AppError};
use crate::domain::{Detail, Stake, SubjectKind};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "jurygraph",
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
pub enum Cmd {
    /// Personas: add one from the fixed JSON, write the brief that asks an agent to generate them, list them
    Persona {
        #[command(subcommand)]
        what: PersonaCmd,
    },
    /// Panels: a saved query over persona tags ("5 builder, 3 company, 2 others")
    Panel {
        #[command(subcommand)]
        what: PanelCmd,
    },
    /// Lenses: dimension sets with anchors in words (four ship: critical, taste, compare, recall)
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
        #[arg(long)]
        panel: String,
        #[arg(long)]
        lens: String,
        /// take the first N personas of the panel, primary → secondary → bystander (default: all)
        #[arg(long)]
        spend: Option<usize>,
        /// a stake for every panel persona: `<persona>=primary|secondary|bystander`, repeated or comma-separated
        #[arg(long = "stake", value_delimiter = ',', required = true)]
        stakes: Vec<String>,
        /// the model the agent will run the personas on (the run is pinned to it)
        #[arg(long)]
        model: String,
        /// comparator snapshots this run holds
        #[arg(long = "comparator")]
        comparators: Vec<String>,
        /// compact (attachments by path) or full (text attachments inline)
        #[arg(long, default_value = "compact")]
        detail: String,
    },
    /// Ingest one persona's verdict JSON for a run
    Verdict {
        #[arg(long)]
        run: i64,
        #[arg(long, conflicts_with = "json")]
        file: Option<PathBuf>,
        #[arg(long)]
        json: Option<String>,
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
    },
    /// The forced choices of a run, per stratum, with reasons
    Pick { run: i64 },
    /// What moved between two runs on the same subject and lens
    Delta { a: i64, b: i64 },
    /// Stale runs (the subject moved) and findings with no decision; exits 1 on an unresolved finding
    Check {
        #[arg(long)]
        history: bool,
    },
    /// A person's word on a finding: fixed, dismissed (with --why) or row (with --row)
    Decide {
        finding: String,
        outcome: String,
        #[arg(long)]
        why: Option<String>,
        #[arg(long)]
        row: Option<String>,
        #[arg(long)]
        by: String,
    },
    /// Classify a diff against jurygraph.toml: none | low | high | human, and the spend
    ShouldRun {
        #[arg(long)]
        diff: Option<PathBuf>,
        #[arg(long, value_delimiter = ',')]
        paths: Vec<String>,
        #[arg(long)]
        config: Option<PathBuf>,
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
    },
    List,
}
#[derive(Subcommand, Debug)]
pub enum PanelCmd {
    Add {
        name: String,
        #[arg(long)]
        query: String,
    },
    Resolve { name: String },
}
#[derive(Subcommand, Debug)]
pub enum LensCmd {
    Add {
        #[arg(long)]
        file: PathBuf,
    },
    List,
}
#[derive(Subcommand, Debug)]
pub enum SubjectCmd {
    Add {
        name: String,
        /// page | diff | design | logo | document | menu | prices
        #[arg(long)]
        kind: String,
        #[arg(long, num_args = 1.., required = true)]
        attach: Vec<PathBuf>,
    },
}

pub fn verb(cmd: &Cmd) -> &'static str {
    match cmd {
        Cmd::Persona { what: PersonaCmd::Add { .. } } => "persona add",
        Cmd::Persona { what: PersonaCmd::Generate { .. } } => "persona generate",
        Cmd::Persona { what: PersonaCmd::List } => "persona list",
        Cmd::Panel { what: PanelCmd::Add { .. } } => "panel add",
        Cmd::Panel { what: PanelCmd::Resolve { .. } } => "panel resolve",
        Cmd::Lens { what: LensCmd::Add { .. } } => "lens add",
        Cmd::Lens { what: LensCmd::List } => "lens list",
        Cmd::Subject { .. } => "subject add",
        Cmd::Comparator { .. } => "comparator add",
        Cmd::Brief { .. } => "brief",
        Cmd::Verdict { .. } => "verdict",
        Cmd::Matrix { .. } => "matrix",
        Cmd::Pick { .. } => "pick",
        Cmd::Delta { .. } => "delta",
        Cmd::Check { .. } => "check",
        Cmd::Decide { .. } => "decide",
        Cmd::ShouldRun { .. } => "should-run",
        Cmd::Setup { .. } => "setup",
        Cmd::Doctor { .. } => "doctor",
        Cmd::Mcp => "mcp",
    }
}

fn detail(s: &str) -> app::Result<Detail> {
    Detail::parse(s).ok_or_else(|| AppError::InvalidArguments(format!("unknown detail {s} (compact|full)")))
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
            PersonaCmd::Generate { from, count } => app::persona::generate(root, from, *count),
            PersonaCmd::List => app::persona::list(root),
        },
        Cmd::Panel { what } => match what {
            PanelCmd::Add { name, query } => app::panel::add(root, name, query),
            PanelCmd::Resolve { name } => app::panel::resolve(root, name),
        },
        Cmd::Lens { what } => match what {
            LensCmd::Add { file } => app::lens::add(root, file),
            LensCmd::List => app::lens::list(root),
        },
        Cmd::Subject { what: SubjectCmd::Add { name, kind: k, attach } } => {
            app::subject::add(root, name, kind(k)?, attach, false)
        }
        Cmd::Comparator { what: SubjectCmd::Add { name, kind: k, attach } } => {
            app::subject::add(root, name, kind(k)?, attach, true)
        }
        Cmd::Brief { subject, panel, lens, spend, stakes, model, comparators, detail: d } => app::brief::compose(
            root,
            &app::brief::BriefArgs {
                subject: subject.clone(),
                panel: panel.clone(),
                lens: lens.clone(),
                spend: *spend,
                stakes: parse_stakes(stakes)?,
                model: model.clone(),
                comparators: comparators.clone(),
                detail: detail(d)?,
            },
        ),
        Cmd::Verdict { run, file, json } => {
            let input = match (file, json) {
                (Some(f), None) => app::read_text(f)?,
                (None, Some(j)) => j.clone(),
                _ => return Err(AppError::InvalidArguments("give the verdict as --file <path> or --json <text>".into())),
            };
            app::verdict::ingest(root, *run, &input)
        }
        Cmd::Matrix { runs, across_strata, history, table, detail: d } => {
            app::matrix::show(root, runs, *across_strata, *history, *table, detail(d)?)
        }
        Cmd::Pick { run } => app::pick::show(root, *run),
        Cmd::Delta { a, b } => app::delta::show(root, *a, *b),
        Cmd::Check { history } => app::check::run(root, *history),
        Cmd::Decide { finding, outcome, why, row, by } => {
            app::decide::record(root, finding, outcome, why.as_deref(), row.as_deref(), by)
        }
        Cmd::ShouldRun { diff, paths, config } => {
            app::should_run::classify(root, diff.as_deref(), paths, config.as_deref())
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
