# JuryPersona v0.2 — panels and the draw — Implementation Plan (as built)

> **As built.** This plan was written after the work, from the code, the tests and the review that followed. It records what was
> built, not what was intended. The decisions of record are D9 and D10 in [`../decisions.md`](../decisions.md); where this plan and
> a decision differ, the decision wins. Measured result: [`../reports/2026-10-24-v0-2-measured.md`](../reports/2026-10-24-v0-2-measured.md).

**Goal:** A rule picks the panel. `brief --rule <class>` takes panel, lenses, spend and detail from `jurypersona.toml`; a seeded draw
decides which personas sit when the pool is larger than the seats; the agent may override any of it with a stated reason; a brief above
12 seats needs `--allow-large`, above 1000 is always refused. v0.1 briefs stay byte-identical.

**Architecture:** unchanged rings. New pure code in `domain/` (`rng.rs`, draw in `query.rs`, a text-file citation check); one schema
bump; the bookkeeping lives in `application/brief.rs`. No dependency added, no model call, no network.

## Global Constraints

- `tests/fixture_replay.rs` and `fixtures/run/**` are not touched and stay green: the proof that v0.2 changed no v0.1 brief.
- A draw is a function of the subject's name and the panel's name, never of the subject's hash, so round 2 draws the same seats as round 1.
- Nothing is silent: every departure from the rule is recorded on the run and printed in the matrix header.
- Strata are never averaged, at any size.

## Decisions it made

| # | what |
|---|---|
| D9 | Schema v3 (`run.draw_seed`, `run.override_json`); the seeded Fisher-Yates draw with SplitMix64; a stake per panel term (`3 builder=primary`); `brief --rule`; the new refusals |
| D10 | Seat ceilings `SEAT_CEILING = 12` and `SEAT_HARD_MAX = 1000`; overrides on top of a rule need `--reason` and are recorded; the estimate is shown before the spend |

## Tasks (as committed, nine commits)

### Task 1 — domain: stake per term, the draw, SplitMix64, `draw_seed`
Files: `domain/rng.rs`, `domain/query.rs`, `domain/hash.rs`, `domain/model.rs`. Tests: `query` (7), `rng` (3), `hash` (1) unit tests,
red first; the shuffle runs only where the pool is bigger than its seats.

### Task 2 — trigger: a rule needs a panel and lenses
`rule_without_panel`, `rule_panel_unknown`, `rule_lens_unknown`; empty `lenses` is `invalid_config`. Tests: 2 unit; eight older
integration tests had to be given a panel because the spec changed them.

### Task 3 — store schema v3
`run.draw_seed`, `run.override_json`; a v2 store is refused by its version. Tests: `store` (schema_version 3, v2 refused, seed and
override round trip), `doctor` (1).

### Task 4 — text-file citation
`#L<n>` past the end of a UTF-8 text attachment is `citation_unresolved`. Tests: a citation unit test; `refusals_judging` (1).

### Task 5 — ceilings, `panel resolve --subject`, the draw in `brief`
`panel_too_large` with data `{ceiling, seats}`; `panel add --allow-large`; `panel resolve --subject`. Tests: `panel_draw` (3).

### Task 6 — `brief --rule`, overrides, estimate, ad-hoc `--query`
`--rule --query --draw --allow-large --reason --by`; one run per lens under a rule; `estimate {briefs, tokens}`; `override_without_reason`;
`matrix` JSON `override` and a header line. Tests: `panel_draw` (13 more, the matrix header test last).

### Task 7 — `should-run --subject`, warnings
Tests: 2.

### Task 8 — `persona generate`, two steps
Step 1 lists roles, sectors and objections; step 2 writes the personas (the count stays `--count`). Test: 1.

### Task 9 — docs
D9, D10, `commands.md`, `mcp-tools.md` (stakes optional on `brief`; `subject` on `should_run`), both skills, the changelog, the fixture
config (the `copy` rule has a panel).

## The refusals it added

`panel_too_large`, `rule_without_panel`, `rule_panel_unknown`, `rule_lens_unknown`, `panel_term_empty`, `override_without_reason`;
`citation_unresolved` for a line past the end of a text file.

## Review and the fix round

An independent review of the nine commits found no Critical and six Important findings (two of them gates that D10 called closed).
One fix commit closed all six Important and six of the Minor findings, tests first. See the measured report for the list.
