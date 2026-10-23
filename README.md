<!-- adapted from comparegraph README.md (shape only) -->
# JuryGraph

**A persona jury as an engine: your agent runs the personas, the engine keeps the jury honest.** Already installed? `jurygraph doctor`

[![ci](https://github.com/massrelay-ai/jurygraph/actions/workflows/ci.yml/badge.svg)](https://github.com/massrelay-ai/jurygraph/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/massrelay-ai/jurygraph?include_prereleases)](https://github.com/massrelay-ai/jurygraph/releases)
[![licence](https://img.shields.io/badge/licence-Apache--2.0-blue)](LICENSE)
![MCP tools](https://img.shields.io/badge/MCP%20tools-5-blue)
![Rust](https://img.shields.io/badge/Rust-1.98-orange)
![local](https://img.shields.io/badge/100%25%20local-no%20model%20calls-brightgreen)
![platforms](https://img.shields.io/badge/platforms-linux%20%7C%20macOS%20%7C%20windows-lightgrey)
![agents](https://img.shields.io/badge/agents-Claude%20Code%20%7C%20Codex%20%7C%20any%20MCP%20client-lightgrey)

It stores personas, panels, lenses and subjects, composes the briefs, ingests the verdicts your agent's subagents write,
refuses a verdict without evidence, and reports scores per stratum. The engine never calls a model.

<p align="center"><img src="docs/assets/loop.svg" alt="The loop: should-run, brief, verdict, matrix per stratum, decide, delta run inside the engine; your subagents, your model and your bill sit outside it, take the briefs, and return verdicts, and a verdict with no citation is sent back as refused." width="100%"></p>

The engine stores, composes, checks and refuses. Your agent (Claude Code, Codex, any MCP client) runs the personas with
its own subagents and its own bill: `brief`, then its subagents, then `verdict`. No API key, no network call at verdict
time. Apache-2.0. Rust, one binary, SQLite at `.jurygraph/store.db`. Status: v0.1, no benchmarks published yet.

**Three evidence classes.** Every verdict line carries one:
- `seen`: the persona looked at an attachment of this run, and cites it (an ARIA node, a screenshot, a diff line).
- `told`: text the brief itself supplied.
- `recalled`: the model's memory, shown as such. A comparison with no comparator snapshot can only be `recalled`.

## Why JuryGraph
Three laws: the engine never calls a model, a verdict line has evidence or is labelled, and strata are never averaged.

**The strata rule.** A persona's stake in a subject is `primary` (would use it), `secondary` (buys, approves or supports it)
or `bystander` (passes by). Scores are reported per stratum, side by side, and never averaged across them; the engine
refuses the table that does.

<p align="center"><img src="docs/assets/strata.svg" alt="The same question, compare:clarity, asked of three strata: the comparator leads Lantern Notes in every stratum, median 4 to 2 for primary, 4 to 1 for secondary and 4 to 2 for bystander; the secondary stratum is the harshest, and pooling every persona into one number (1.9) is the row the engine refuses." width="100%"></p>

The picture is the recorded fixture run (`fixtures/run/matrix.json`, [report](docs/reports/2026-10-23-v0-1-measured.md)).

The engine never calls a model. Your agent (Claude Code, Codex, any MCP client) runs the personas with its own subagents
and its own bill: `brief`, then its subagents, then `verdict`. No API key, no network call at verdict time.

Apache-2.0. Rust, one binary, SQLite at `.jurygraph/store.db`. Status: v0.1, no benchmarks published yet.

## Install
```
curl -fsSL https://raw.githubusercontent.com/massrelay-ai/jurygraph/main/scripts/install.sh | sh
cargo install --locked --git https://github.com/massrelay-ai/jurygraph jurygraph     # or from source
jurygraph setup --apply       # registers the MCP server with the agents it finds; without --apply it prints a plan
jurygraph doctor              # the store, the counts, each client's registration
```
In Claude Code: `/plugin install jurygraph --marketplace massrelay-ai/jurygraph` adds the skills and the MCP server.

## A five-minute example (the fixture site)
Everything below uses the files in `fixtures/`. Output is one JSON envelope per call (`--pretty` to read it).
```
# 1. personas (seven, fixed JSON) and a saved panel query
for f in fixtures/personas/*.json; do jurygraph persona add --file $f; done
jurygraph panel add fixture --query "3 builder, 2 company, 2 others"
jurygraph panel resolve fixture

# 2. what is judged: attached files, never a live fetch; and one comparator
jurygraph subject add pricing --kind page --attach fixtures/site-walk/pricing.walk.json fixtures/site-walk/pricing.png
jurygraph comparator add acme-pricing --kind page --attach fixtures/comparators/acme-pricing/pricing.walk.json \
  fixtures/comparators/acme-pricing/pricing.png

# 3. one brief per persona; the stake is declared per run, for every persona of the panel
jurygraph brief --subject pricing --panel fixture --lens critical --model sonnet --spend 3 --detail compact \
  --stake builder-ana=primary,builder-ravi=primary,builder-mei=primary,company-hanna=primary,company-george=secondary,newcomer-sofia=bystander,investor-claire=secondary

# 4. the agent loop: one subagent per persona reads its brief file and writes one verdict JSON
#    (the skill skills/running-a-jury says how), then:
jurygraph verdict --run 1 --file verdicts/builder-ana.json      # refused without evidence: the persona answers again

# 5. read it, per stratum
jurygraph matrix 1 --table
jurygraph pick 1                                                # forced choices, when the lens asks for one

# 6. one decision per finding, then the next round
jurygraph decide <finding> fixed --by me
jurygraph decide <finding> dismissed --why "out of scope" --by me
jurygraph decide <finding> row --row TRK-12 --by me
jurygraph check                                                 # stale runs; exits 1 on an unresolved finding
jurygraph delta 1 2                                             # what moved between two runs after the fixes

# 7. which changes need a jury at all
jurygraph should-run --diff change.diff --config fixtures/jurygraph.toml
```
`--spend N` takes the first N personas of the panel, primary then secondary then bystander. `--detail compact|full` sets
how much each brief carries: `compact` lists attachments by path and hash, `full` inlines text attachments (an ARIA walk as
lines, a diff, a document) up to 64 KiB each. Every brief reports its byte size. Two knobs, how many and how much each.
`persona generate --from <material>` writes a brief that asks your agent to write the personas; see `skills/writing-a-persona`.

<p align="center"><img src="docs/assets/spend.svg" alt="Brief bytes on the fixture for the critical lens: spend 1 is 5,116 bytes compact and 7,879 full; spend 3 is 15,212 and 23,501; spend 7 is 34,934 and 54,275; the toml's own estimate is 15,000 tokens per brief." width="100%"></p>

Measured with `scripts/readme-assets-data.sh` (the command is recorded in [`docs/assets/spend.json`](docs/assets/spend.json)); the token figure is the project toml's estimate, not a measurement.

<p align="center"><img src="docs/assets/refusal.svg" alt="Two verdict lines: evidence seen with an empty citations list is refused with the code seen_without_citation; evidence recalled is accepted and labelled recalled." width="100%"></p>

Coming in v0.2: a seeded panel draw, a 12-seat default ceiling, and agent overrides that carry a reason.

## The four lenses
Dimension sets with anchors in words (what a 1, a 3 and a 5 look like), shipped as data in `lenses/`.

| lens | asks | notes |
|---|---|---|
| `critical` | works, blocks, missing | does it do the job, what stops me, what is absent |
| `taste` | look, type, motion, voice | against a design standard the project points at |
| `compare` | clarity, price, trust | against named comparators, each a snapshot in the run; the persona also picks one |
| `recall` | first-read, would-return | a first impression from general memory, cheap, always `recalled` |

Add your own with `jurygraph lens add --file my-lens.toml`; `jurygraph lens list` shows them.
A lens is a versioned standard; `docs/lenses.md` says how, and a 1 to 10 `site` lens ships as fixture data
(`fixtures/lenses/site.toml`, loaded with `lens add`, not seeded).

## What the engine refuses
Each refusal is a test that was seen failing first.
- a `seen` verdict line with no citation;
- a `compare` line naming a comparator the run holds no snapshot of (it may be stored only as `recalled`);
- a score outside the lens's range, or on a dimension the lens lacks; a dimension scored twice for the same option or not at
  all; a score at the bottom of the scale with no blocker;
- a table that averages across strata;
- a verdict for a run whose subject changed since the brief;
- a persona with no stake in the run;
- a `pick` without a reason;
- a panel query that resolves to fewer personas than `--spend`;
- a `jurygraph.toml` that puts a `human` class (money, permission, deploy, secret, licence, public-claim, brand) under a
  persona quorum.

## How good is it
One fixture run: a seven-persona jury over three strata read the Lantern Notes site and found all four planted defects. Verdicts: 14 accepted, 0 refused. Findings: 26, with one false positive. This is a fixture we wrote, not a benchmark. The report is
[`docs/reports/2026-10-23-v0-1-measured.md`](docs/reports/2026-10-23-v0-1-measured.md); every score is below.

<p align="center"><img src="docs/assets/matrix.svg" alt="The fixture jury, every score: seven personas in three strata (four primary, two secondary, one bystander) across 21 columns of lens, dimension and option, each cell a score from 1 to 5, with a median row per stratum and a blockers count per persona; no row spans strata." width="100%"></p>

## Not every change needs a jury
`jurygraph.toml` holds trigger rules as data (`fixtures/jurygraph.toml` is an example). `should-run` classifies a diff:

| quorum | who decides |
|---|---|
| `none` | tests and the adversarial read (internals, migrations, tests) |
| `low` | three personas, one lens; applies itself with no primary blocker and no split |
| `high` | the full panel, all lenses, a foreman proposes the fixes |
| `human` | a person; the jury only advises |

A split verdict or any primary blocker escalates one level. In v0.1 the engine classifies and reports; the agent applies the
escalation rule (`skills/running-a-jury`), and the engine does not compute a split yet. `tokens_per_brief` in the toml is
the project's own estimate, not a measurement.

## What a citation proves and what it does not
A citation proves the persona was **given** the attachment and **named a node that exists** in it (a walk node, an image
label, a diff line). It does not prove the persona looked, read what it cites or judged well; `seen` is the persona's claim
about what it opened. A published matrix is meaningful only with its subject, panel, lens and brief hashes beside it
(`docs/publishing-results.md`).

## What it cannot see
It checks that a citation names a real walk node, attachment or diff line; it does not check image regions, and it cannot
tell whether a persona read what it cites or is realistic. Colour and layout reach the jury only through screenshots.

## Layout
`crates/jurygraph` the engine · `lenses/` the shipped lenses · `skills/` the agent's method · `fixtures/` the Lantern Notes
site, its answer key and its personas · `docs/` decisions, plans, release steps. Contributing: `CONTRIBUTING.md`, `AGENTS.md`.
Security: `SECURITY.md`. Governance: `GOVERNANCE.md`. Help: `SUPPORT.md`. Personas: `PERSONAS.md`. All the docs: `docs/README.md`.

## Licence
Apache-2.0 (`LICENSE`). Files adapted from our sibling CompareGraph are listed in `docs/THIRD_PARTY_NOTICES.md`.
