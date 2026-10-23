# PersonaJury v0.5 — the summary, calibration and CI — Implementation Plan (as built)

> **As built.** Written after the work, from the code, the tests and the review. Decisions of record: D20 to D24 in
> [`../decisions.md`](../decisions.md); the display choice is in [`../reports/2026-10-24-display-jury.md`](../reports/2026-10-24-display-jury.md).
> Measured result: [`../reports/2026-10-24-v0-5-measured.md`](../reports/2026-10-24-v0-5-measured.md).

**Goal:** Make a jury's answer readable by the person who asked, and make the gate bind where a team already works: `summary` (sentences
first, then a numbers table), `calibrate` against a human run, CSV cell citations, a GitHub Action, and a recipe for the Claude Agent SDK.

**Architecture:** `domain/summary.rs` and `application/summary.rs`; `domain/calibrate.rs`; `domain/csv.rs`; `action.yml` and
`scripts/action-run.sh`. No new crate. The engine still has no network code.

## Global Constraints

- Strata never pooled: no combined row, no combined number in the summary.
- The summary is deterministic and terminal-safe; the golden files `fixtures/run/summary.txt` and `summary.md` pin it.
- `fixture_replay` untouched and green.

## Decisions it made

| # | what |
|---|---|
| D20 | `calibrate <jury-run> <human-run>`; `calibrated` iff every primary cell is within one rank; `apply-check --calibration`; `uncalibrated` is `advise`, not `escalate` |
| D21 | `<label>#R<row>C<col>` on a `.csv` attachment; a small RFC 4180 parser |
| D22 | The GitHub Action and `apply-check --latest` |
| D23 | The Claude Agent SDK recipe (docs and one example, no runner) |
| D24 | The answer display: `summary`, no bars anywhere (display jury, 0 of 8 picks for bars) |

## Tasks

1. **`summary` (D24).** CLI, MCP `personajury_summary` (eight tools), `--html`, `--md`, `--save`, `--width`. 12 of 12 tests red first.
2. **`calibrate` and `apply-check --calibration` (D20).** 8 of 8 tests red first.
3. **CSV cell locators (D21).** 6 of 6 unit tests red first, plus a mutation proving the citation guard.
4. **The Action (D22).** `action.yml`, `scripts/action-run.sh`, `docs/ci.md`, and the new flag `apply-check --latest`. 8 of 8 tests red first.
5. **The Agent SDK recipe (D23).** `docs/agent-sdk.md`, `examples/agent-sdk/run_jury.py`; the test only byte-compiles the example and checks it holds no key.
6. **Docs:** a tradeoffs lens, a technical-decision playbook, "display is free, the verdict is not", what goes to a jury, where results live,
   commands, MCP tools, the changelog, the roadmap, D20 to D24.
7. **Fix round** after the review: 18 probe tests in `tests/v05_review.rs`.

## Review and the fix round

The review found one Critical, three Important and seven Minor findings (the Critical and one Important both made the CI gate fail open).
The fix round closed the Critical, all three Important and five of the Minor findings. Details in the measured report.
