# PersonaJury v0.4 — the apply check — Implementation Plan (as built)

> **As built.** Written after the work, from the code, the tests and the review. Decisions of record: D13 to D19 in
> [`../decisions.md`](../decisions.md). Measured result: [`../reports/2026-10-24-v0-4-measured.md`](../reports/2026-10-24-v0-4-measured.md).

**Goal:** Let an agent act on a jury without a person watching, and refuse to when it should not. `apply-check` answers `apply`,
`advise`, `escalate` or `human` from the run alone; the rest of the release gives it what it needs to be honest: effort, independence,
experts and a human verdict, a model per panel term, `lens generate`, and packaging for the places an agent runs.

**Architecture:** `domain/apply.rs` (splits, the answer's precedence, pure) and `application/apply_check.rs`; schema v4 with an idempotent
migration from v3. The engine still calls no model and opens no socket (a guard test scans `src/`).

## Global Constraints

- `apply` only when every condition is proven; anything unknown, missing or lowered answers `advise` or worse (the ruling after the review, D13).
- `fixture_replay` byte-identical; `fixtures/run/*` untouched, so no brief byte changed.
- Strata never averaged; a split is a sibling key of `strata` in the matrix, never a field of a stratum.

## Decisions it made

| # | what |
|---|---|
| D13 | The split rule and `apply-check`; the full list of conditions, one table row each |
| D14 | `lens generate --from PATH [--dimensions N]`: a two-step brief; `lens add` validates |
| D15 | A model per panel term (`3 builder=primary@model`); `models` and `models_distinct` in the matrix; `single_model` is informational |
| D16 | Packaging: the `.mcpb` bundle, `mcp.json`, registry `packages`, a no-listener guard |
| D17 | Effort (`low`, `medium`, `high`, `max`) and `budget_tokens`; every cut recorded; a cap never drops a stratum; schema v4 |
| D18 | Independence: a word on the run (`independent` or `sequential`); sequential never applies |
| D19 | Experts (`credential`) and `verdict --by-human`; a human blocker escalates until a person decides it |

## Tasks

1. **Apply-check and the split rule (D13).** `tests/v04.rs`: 7 tests, `0 passed; 7 failed` red (`unrecognized subcommand`). MCP: 7 passed / 2 failed
   red (tool list, `personajury_apply_check`). The recorded fixture run answers `escalate` (a primary blocker); a test pins it.
2. **Effort (D17).** Compile-red until the `RunRow` fields existed, then 13 passed / 4 failed on behaviour.
3. **Independence (D18).** 19 passed / 6 failed red.
4. **Experts and a human verdict (D19).** 26 passed / 8 failed red; the MCP has no `by_human`, and a test says so.
5. **A model per term (D15).** Compile-red (`BriefRef.model`), then 6 failed.
6. **`lens generate` (D14).** 4 passed / 3 failed red; the three sector lenses in `fixtures/lenses/` are tested with `lens add`.
7. **Packaging (D16).** `tests/manifests.rs` seen red by bumping one manifest's version; the listener guard seen red by planting a
   `TcpListener::bind` in `src/`; both restored.
8. **Docs:** D13 to D19, commands, MCP tools (7), usage, publishing results, lenses, roadmap, the changelog, the skill, release notes.
9. **Fix round** after the review (see below): 12 probes ported to `tests/v04_review_probes.rs`, and a 27-row table `tests/apply_check_table.rs`.

## Review and the fix round

The review found five Critical findings (each let `apply-check` answer `apply` where it must not), five Important and five Minor. The fix
round closed all ten Critical and Important findings; the principle "apply only when every condition is proven" was written into D13.
Details in the measured report.
