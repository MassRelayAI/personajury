<!-- adapted from comparegraph README.md (shape only) -->
<!-- sources: docs/reports/2026-10-23-v0-1-measured.md (the fixture run); docs/reports/2026-10-24-logo-jury.md (the logo); the CompareGraph readme-jury report (linked); docs/assets/jury-graph.svg <desc> (hero alt); docs/assets/spend.json and strata.json (figures); docs/commands.md (12, 1000, 30: the seat ceilings and the wave size) -->

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/brand/lockup-dark.svg">
    <img src="docs/assets/brand/lockup.svg" alt="JuryGraph" height="64">
  </picture>
</p>

<p align="center">
  <b>For builders with a coding agent: put your page, docs, deck or diff before a jury of personas, and keep only the verdicts that cite evidence.</b>
</p>

<p align="center">
  <a href="https://github.com/massrelay-ai/jurygraph/actions/workflows/ci.yml"><img src="https://github.com/massrelay-ai/jurygraph/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0%2C%20free-blue.svg" alt="License: Apache-2.0, free"></a>
  <img src="https://img.shields.io/badge/engine-no%20model%20calls-brightgreen.svg" alt="The engine makes no model calls">
  <img src="https://img.shields.io/badge/MCP-server%20%2B%20CLI-555.svg" alt="An MCP server and a CLI">
</p>

<p align="center">
  <a href="#get-started">Get started</a> ·
  <a href="#proven-on-real-work">Proof</a> ·
  <a href="#what-a-jury-can-judge-beyond-code">Beyond code</a> ·
  <a href="docs/quickstart.md">Quickstart</a> ·
  <a href="docs/commands.md">Commands</a> ·
  <a href="docs/faq.md">FAQ</a>
</p>

<p align="center"><img src="docs/assets/jury-graph.svg" alt="The jury graph of the fixture run on the Lantern Notes site, drawn by jurygraph graph: 7 personas in three strata (primary 4, secondary 2, bystander 1) on the left, the 6 dimensions they scored across 2 lenses in the middle, each with one median badge per stratum and never one number across strata, and the 26 findings on the right, coloured by their decision: 0 open, 0 fixed, 11 dismissed, 15 sent to a row." width="100%"></p>

<p align="center"><i>The engine never calls a model; your agent's subagents are the jurors.</i></p>

**[Proven on real work](#proven-on-real-work).** A docs jury cut a sibling project's critical blockers from 15 to 6, and its README went from 4 of 7 picks to 7 of 7. A fixture jury found all four planted defects. This logo was picked by a jury, 7 of 7.

---

## Get started

One binary, SQLite at `.jurygraph/store.db`, no API key. Linux, macOS and Windows.

```bash
curl -fsSL https://raw.githubusercontent.com/massrelay-ai/jurygraph/main/scripts/install.sh | sh
# or from source: cargo install --locked --git https://github.com/massrelay-ai/jurygraph jurygraph
jurygraph setup --apply   # registers the MCP server with your agents (without --apply: a plan)
jurygraph doctor          # the store, the counts, each agent's registration
```

In Claude Code, `/plugin install jurygraph --marketplace massrelay-ai/jurygraph` adds the skills and the MCP server.

**The five-minute loop**, on the fixture site in this repository:

```bash
git clone https://github.com/massrelay-ai/jurygraph && cd jurygraph
for f in fixtures/personas/*.json; do jurygraph persona add --file "$f"; done
jurygraph panel add fixture \
  --query "3 builder=primary, 1 newcomer=primary, 2 company=secondary, 1 investor=bystander"
jurygraph subject add lantern-site --kind page --attach fixtures/site-walk/*
jurygraph brief --subject lantern-site --panel fixture --lens critical --model sonnet  # a brief per persona
# your agent: one subagent per brief, each writes one verdict JSON (skills/running-a-jury)
jurygraph verdict --run 1 --file verdicts/builder-ana.json  # refused without evidence: it answers again
jurygraph matrix 1 --table                                  # scores per stratum, never averaged across
jurygraph graph 1 --svg --out jury.svg                      # the picture above
jurygraph decide <finding> fixed --by me                    # or: dismissed --why "...", or: row --row ISSUE-12
jurygraph check                                             # exits 1 while a finding has no decision
```

Every call prints one JSON envelope (`--pretty` to read it). The full walk, with comparators and the second round, is [`docs/quickstart.md`](docs/quickstart.md).

## Why JuryGraph

An agent asked "is this page good?" answers with confidence and no evidence. One number then hides that the people who pay and the people who pass by disagree. JuryGraph is three laws, each enforced by a refusal:

1. **The engine never calls a model.** It stores personas, panels, lenses and subjects, writes the briefs and checks the verdicts. Your agent runs the personas on its own model and its own bill. No network call at brief or verdict time.
2. **Evidence, or it is labelled.** Every verdict line is `seen` (it cites an attachment of the run: an ARIA node, a screenshot, a diff line), `told` (the brief said it) or `recalled` (the model's memory, shown as such). A `seen` line with no citation is refused, and the persona answers again.
3. **Strata are never averaged.** Each persona has a stake: `primary` (would use it), `secondary` (buys, approves or supports it) or `bystander` (passes by). Scores are reported per stratum, side by side. The engine refuses the table that pools them.

<p align="center"><img src="docs/assets/strata.svg" alt="The same question, compare:clarity, asked of three strata in the recorded fixture run: the comparator leads Lantern Notes in every stratum, median 4 to 2 for primary, 4 to 1 for secondary and 4 to 2 for bystander; the secondary stratum is the harshest, and pooling every persona into one number (1.9) is the row the engine refuses." width="100%"></p>

<p align="center"><img src="docs/assets/refusal.svg" alt="Two verdict lines: evidence seen with an empty citations list is refused with the code seen_without_citation; evidence recalled is accepted and labelled recalled." width="100%"></p>

## How it works

<p align="center"><img src="docs/assets/loop.svg" alt="The loop: should-run, brief, verdict, matrix per stratum, decide, delta run inside the engine; your subagents, your model and your bill sit outside it, take the briefs, and return verdicts, and a verdict with no citation is sent back as refused." width="100%"></p>

`should-run` says whether a change needs a jury. `brief` writes one hashed brief per persona. Your agent's subagents answer, and `verdict` ingests or refuses each answer. `matrix` reports per stratum. `decide` closes each finding as `fixed`, `dismissed` (with a why) or `row` (an issue). `delta` says whether the next round landed.

- **Rules pick the panel per audience.** `brief --rule CLASS` takes the panel, lenses, spend and detail from `jurygraph.toml`, one run per lens.
- **A seeded draw.** When a pool holds more personas than seats, the seats are drawn from the subject's name, so round 2 meets the same jurors and `delta` can land.
- **Agent overrides carry a reason.** The agent may change a rule's panel, spend, lens or detail, but each change needs `--reason`, and `matrix` shows it. A brief seats 12 by default, up to 1000 with `--allow-large`.
- **Waves.** Every brief lists its jurors in waves of 30 (`--wave N`). The agent runs a wave, ingests it, then the next.
- **Reported usage beside the estimate.** A verdict may carry the tokens the agent spent. `matrix` and `doctor` show that sum, labelled "reported by the agent", beside the estimate. The engine measures no token.
- **`graph --svg`.** The jury as one deterministic picture, the hero above. Over several runs, each finding shows its life.
- **`check --history`.** Each finding's life across runs: raised, raised again after a fix, fixed, not judged, absent.

**The spend dial.** `--spend N` takes the first N personas of the panel, primary first. `--detail compact|full` sets how much each brief carries. Every brief answers its byte size and an estimate before anything is spent.

<p align="center"><img src="docs/assets/spend.svg" alt="Brief bytes on the fixture for the critical lens: spend 1 is 5,116 bytes compact and 7,879 full; spend 3 is 15,212 and 23,501; spend 7 is 34,934 and 54,275; the toml's own estimate is 15,000 tokens per brief." width="100%"></p>

Measured by `scripts/readme-assets-data.sh` ([`docs/assets/spend.json`](docs/assets/spend.json)); the token figure is the project's own estimate, not a measurement.

## Proven on real work

Three runs, each with its report. The personas are composites, not a survey: they show where to look, and they do not replace real users.

| run | what the jury judged | result |
|---|---|---|
| **The fixture** | Lantern Notes, a made-up site with four planted defects; seven personas over three strata | all four planted defects found; 14 verdicts accepted, 0 refused; 26 findings, one false positive ([report](docs/reports/2026-10-23-v0-1-measured.md)) |
| **CompareGraph's docs** | a sibling project's README, usage, FAQ and troubleshooting, under three lenses, against two other projects' READMEs | critical blockers from 15 to 6, compare blockers from 6 to 0; all three deltas landed; the forced pick went from 4 of 7 to 7 of 7 ([report](https://github.com/massrelay-ai/comparegraph/blob/main/docs/reports/2026-10-24-readme-jury.md)) |
| **This logo** | four directions, then the winner against two refinements | C won 6 of 7 picks, then 7 of 7; the refinements lost legibility at 16 px, so C ships as drawn ([report](docs/reports/2026-10-24-logo-jury.md)) |

The fixture is ours, so it proves the loop, not a benchmark. In the CompareGraph run the engine did not change; only the words did. The secondary stratum still says `blocks: 2` there, because better words cannot close a missing CI check.

<details>
<summary><b>Every score of the fixture jury</b></summary>

<p align="center"><img src="docs/assets/matrix.svg" alt="The fixture jury, every score: seven personas in three strata (four primary, two secondary, one bystander) across 21 columns of lens, dimension and option, each cell a score from 1 to 5, with a median row per stratum and a blockers count per persona; no row spans strata." width="100%"></p>

</details>

## What a jury can judge, beyond code

Anything people read or look at, as long as you can attach it as files. The jury never fetches a live page.

| subject | as files | the strata: primary / secondary / bystander | a lens |
|---|---|---|---|
| Product pages and docs | an ARIA walk and screenshots per page; the markdown | users / the buyer or approver / a passer-by | `critical`, `compare`, `taste` |
| Slides and consulting decks | a PDF export, one PNG per slide, the speaker notes | the client team who acts / the partner who signs off / an executive who reads only the titles | [`consulting-deck`](fixtures/lenses/consulting-deck.toml) |
| Investor pitches | the deck as PDF and PNGs, the one-pager | investors in your sector / their partners / a generalist angel | [`investor-pitch`](fixtures/lenses/investor-pitch.toml) |
| Financial models (readability, not correctness) | a CSV or PDF per sheet, the summary as a screenshot | the credit analyst / the credit committee / a board member | [`financial-model`](fixtures/lenses/financial-model.toml) |
| Business plans and policy briefs | the document and its charts | who must act on it / who approves it / a reader from the public | `critical` |
| Pricing | the pricing page as walk and screenshot; rivals' pages as comparators | who pays / who approves the spend / someone shopping around | `compare`, with a forced pick |
| Logos and design | one board per candidate: small and large, light and dark | the users / the approvers / a passer-by | your own lens with a forced pick, as the [logo jury](docs/reports/2026-10-24-logo-jury.md) did |

The three sector lenses ship as data in `fixtures/lenses/`, not seeded: `jurygraph lens add --file fixtures/lenses/investor-pitch.toml`. Each has four dimensions with anchors in words at 1, 3 and 5.

**What a jury must not judge:**
- **Correctness.** Tests do that. A financial-model jury reads the model; it never checks the arithmetic.
- **Money, legal, permissions and deploys.** A person decides; the jury advises. The engine refuses a config that puts these under a persona quorum.
- **Real users, when you have them.** Personas tell you where to look. Real people overrule them.

## Lenses

A lens is a set of dimensions with anchors in words: what a 1, a 3 and a 5 look like. It is versioned data in TOML; an edited anchor is a new version ([`docs/lenses.md`](docs/lenses.md)).

| lens | asks | ships as |
|---|---|---|
| `critical` | works, blocks, missing | seeded |
| `taste` | look, type, motion, voice, against a design standard the project names | seeded |
| `compare` | clarity, price, trust, against named comparators held as snapshots; the persona also picks one | seeded |
| `recall` | first-read, would-return: a cheap first impression, always `recalled` | seeded |
| `consulting-deck` | storyline, action-titles, evidence, the-ask | `fixtures/lenses/` |
| `investor-pitch` | problem, why-now, proof, the-ask | `fixtures/lenses/` |
| `financial-model` | assumptions-visible, sensitivities, risks-stated, a-lender-would-sign | `fixtures/lenses/` |
| `site` | six criteria on a 1 to 10 scale, for the held-out check | `fixtures/lenses/` |

Add your own with `jurygraph lens add --file my-lens.toml`; `jurygraph lens list` shows them.

## What the engine refuses

Each refusal is a test that was seen failing first. The codes are in [`docs/troubleshooting.md`](docs/troubleshooting.md).
- a `seen` line with no citation, or a citation that names no node, attachment or line of the run;
- a comparison with a comparator the run holds no snapshot of (it may be stored only as `recalled`);
- a score off the lens's scale or on a dimension it lacks; a dimension scored twice or not at all; a bottom score with no blocker;
- a table that averages across strata;
- a verdict for a run whose subject changed since the brief, or from a persona with no stake in the run;
- a `pick` without a reason;
- a rule override, or a brief above 12 seats, without a reason;
- reported usage in any shape but two non-negative token counts;
- a `jurygraph.toml` that puts a human class under a persona quorum.

## Not every change needs a jury

`jurygraph.toml` holds trigger rules as data ([`fixtures/jurygraph.toml`](fixtures/jurygraph.toml) is an example). `should-run` classifies a diff:

| quorum | who decides |
|---|---|
| `none` | tests and an adversarial read: internals, migrations, tests |
| `low` | three personas, one lens; applies itself with no primary blocker and no split |
| `high` | the full panel, all lenses; a foreman proposes the fixes |
| `human` | a person; the jury only advises |

The human classes are money, permission, deploy, secret, licence, public-claim and brand. A split or any primary blocker escalates one level; today the agent applies that rule (`skills/running-a-jury`).

**What a citation proves.** That the persona was given the attachment and named a node that exists in it. Not that it looked, read well or is realistic. Colour and layout reach the jury only through screenshots, and image regions are not checked. A published matrix means something only with its subject, panel, lens and brief hashes beside it ([`docs/publishing-results.md`](docs/publishing-results.md)).

## Agents and platforms

`jurygraph setup` registers the MCP server with Claude Code, Codex, Cursor, Gemini CLI, OpenCode, Hermes, Windsurf, Antigravity, Kiro, GitHub Copilot, Copilot CLI and Qwen Code; any other MCP client can run `jurygraph mcp`. The CLI does everything the tools do. Release builds: Linux x64 and arm64, macOS arm64 and x64, Windows x64. Two skills, [`running-a-jury`](skills/running-a-jury) and [`writing-a-persona`](skills/writing-a-persona), teach your agent the loop.

## Docs

[Quickstart](docs/quickstart.md) · [Usage](docs/usage.md) · [Commands](docs/commands.md) · [MCP tools](docs/mcp-tools.md) · [Lenses](docs/lenses.md) · [FAQ](docs/faq.md) · [Troubleshooting](docs/troubleshooting.md) · [Decisions](docs/decisions.md) · [Reports](docs/reports/README.md) · [Changelog](CHANGELOG.md) · [All docs](docs/README.md)

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`AGENTS.md`](AGENTS.md). Tests first; a new refusal is a test seen red before it existed. Personas are composites, never real people ([`PERSONAS.md`](PERSONAS.md)). Security: [`SECURITY.md`](SECURITY.md). Governance: [`GOVERNANCE.md`](GOVERNANCE.md). Help: [`SUPPORT.md`](SUPPORT.md).

## Licence

Apache-2.0 ([`LICENSE`](LICENSE)). Files adapted from our sibling CompareGraph are listed in [`docs/THIRD_PARTY_NOTICES.md`](docs/THIRD_PARTY_NOTICES.md).
