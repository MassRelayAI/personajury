# PersonaJury v0.6 — traits, decision juries and walks — Implementation Plan (as built)

> **As built.** Written after the work, from the code, the tests, the lane reports and the reviews. Decisions of record: D29 to D39
> and D42 in [`../decisions.md`](../decisions.md) (lane E's export and chat pack are D40 and D41). Measured result:
> [`../reports/2026-10-24-v0-6-measured.md`](../reports/2026-10-24-v0-6-measured.md).

**Goal:** Let a jury stand for any audience, and keep it honest. Personas carry research-based traits, and a panel can cover them.
`--seats auto` seats who the subject serves. A jury can answer questions, not only judge a surface. One deliberation round is
allowed, and it only advises. The numbers stay counts per stratum. A trait can be checked against what the persona did. A page walk
can be captured without writing it by hand.

**Architecture:** traits are data (`traits/`, `trait-sets/`, embedded in the binary) and a store table; `domain/traits.rs`,
`domain/observed.rs`, `domain/lint.rs`, `domain/calibrate.rs`, `domain/decision.rs`; one application module per verb as before.
The walk exporter is `tools/walk.mjs`, outside the engine. No new crate. Schema version 6.

## Global Constraints

- The engine calls no model and no network. `tools/walk.mjs` is a helper the engine never runs.
- Strata never pooled: slices, metrics and `trait observed` count per stratum; no mean anywhere (a test walks the JSON for one).
- `fixture_replay` green; the recorded briefs, matrix and picks byte-identical. New keys and summary lines are additive.
- Every new refusal is a test seen red first, or, where the code came first, a mutation seen red.
- No copied questionnaire item: every trait anchor is our own words, and each file says `items_copied = false`.

## Decisions it made

| # | lane | what |
|---|---|---|
| D29 | A | traits as data: the catalog of 49 traits in 14 families, 8 sets, the persona key (at most 8), the refused ids |
| D30 | A | cover, quotas, balance and coverage on a panel |
| D31 | A | traits in the brief ("you tend to"), slices in the results |
| D32 | C | options by subject kind (`--options subject|attachments`) |
| D33 | C | deliberation: one advise-only round; schema 6 |
| D34 | C | the honest metrics: agreement, shift, panel match, cost, counted per stratum |
| D35 | B | `trait generate`, `trait suggest`, `trait observed` |
| D36 | B | human traits, `calibrate --slice`, `trait report`, `persona lint`, `label_echo`, `graph --slice` |
| D37 | C | decision juries |
| D38 | C | the starter config and cost by default |
| D39 | B | simulated demographics are allowed, labelled and recorded (amends D29 and D35) |
| D42 | D | the walk exporter ships upstream |

## Tasks

1. **Lane A: traits, cover, slices (D29 to D31).** The catalog and `trait install | add | list | show`; schema 5; `brief --traits`;
   panel filters, `--cover` and `coverage`; `matrix --slice` and `summary --slice`; `brief --seats auto`, persona `serves`, `subject
   add --audience`; MCP `personajury_trait_list`. Tests 471 → 526.
2. **Lane C: options, deliberation, metrics, decision juries (D32 to D34, D37, D38).** `--options`; `deliberate`, `verdict
   --deliberated`, `--phase`; the counted metrics in `matrix` and `summary`; subject kind `decision` with `questions.md` and the
   `decide` lens; `personajury init`. Tests 542 → 590.
3. **Lane B: traits proven (D35, D36).** `trait generate --from`; `trait suggest` and `--check`; `trait observed` with
   `traits/observe-map.toml` (18 rows); `verdict --by-human --traits`, `calibrate --slice`, `trait report`; `persona lint`,
   `label_echo`, `graph --slice`; two MCP tools. Tests 605 → 660.
4. **Lane D: the walk exporter (D42).** `tools/walk.mjs` (Chrome DevTools Protocol over Node's `WebSocket`, no npm dependency),
   `docs/walks.md`, `tools/test/walk.test.mjs` (6 tests), one CI step.
5. **Docs** in each lane: `docs/traits.md`, commands, MCP tools, both skills, usage, the changelog, the decisions.

## Review and the fix rounds

Lanes A, B and C were each reviewed read-only by an independent reviewer, then fixed on the same branch before merge.

- **A:** 0 Critical, 5 Important (effort caps under `--seats auto`, near-synonym ids, free text in the `words` trait, install
  reverting an edit, auto never reaching `apply`). Fixed with 11 probe tests; 542 tests.
- **C:** 4 Critical (an initial verdict after deliberation, no grammar for questions, a money question declared a jury class, a split
  that could apply), 3 Important. Fixed with 19 red tests; 605 tests.
- **B:** 1 Critical (options and anchors unscreened), 4 Important. Mid-round the maintainer ruled "allow, label, record" (D39): the
  screen labels instead of refusing. 678 tests.

Details and the open items are in the measured report.

## What the plan did not know

- The brief budget: eight traits add 23.9% to a compact brief, above the 15% detector. Kept as a documented cost (D29).
- MCP tools grew in parallel lanes: eight before v0.6, nine after A, ten after C, twelve after B, thirteen after E.
- The fixture's walk ids are hand-written and differ from the generated ones; re-cite from a captured walk (D42).
- The store went to schema 6, not one migration for all lanes (D33).
