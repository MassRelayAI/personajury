---
date: 2026-10-23
valid_until: 2026-11-06
verdict: "Ready for the maintainer's merge read. All 12 Critical and Important findings are addressed, and each reviewer reproduction flips on the fixed binary. No new Critical or Important breakage. One new Minor (should-run now refuses a diff that removes a line starting with `-- \"`) and one C1 residual (decision recorded between brief and ingest) are listed below."
status: done
reviewer: scoped re-review of the single fix wave (Opus), read-only
scope: final-fixes-package.diff (lane/final-fixes, 4e72c51..f6f8d31 on e2ba78d), against final-review.md and the progress.md rulings
---

# JuryPersona v0.1: re-review of the final fix wave

## How this was checked
- `cargo test -p jurypersona` in the final-fixes worktree (HEAD f6f8d31): **244 passed, 0 failed**.
- `cargo build -p jurypersona`. The fixed binary was copied to `scratchpad/rr/jgnew` and the reviewer's binary to
  `scratchpad/rr/jgold`. Every C and I reproduction was run on both binaries in fresh workspaces under
  `/tmp/claude-1001/-mnt-extra-disk-repos/10cc8f43-f927-415f-a4e9-2945df54356e/scratchpad/rr/`, built with `rr/mk.sh` and `rr/lib.sh`
  (the same base case as `rv/mk.sh`).
- `git diff main...lane/final-fixes -- fixtures/run/verdicts` is **empty**. `briefs.json` and `picks.json` are unchanged as well.

## Critical and Important: verdicts

| id | verdict | decisive output, old binary → fixed binary |
|---|---|---|
| C1 | ADDRESSED | `check` `ok:true, unresolved:[]` → `ok:false … "reason":"fix_not_landed"`, run 2, `check_failed`, exit 1 |
| C2 | ADDRESSED | `delta 1 2` with no verdicts: `landed: True` → `landed: False`, `incomplete` = 6 cells (ana and cy × 3) |
| C3 | ADDRESSED | clean re-ingest: `ok:true` → `ok:false … "reason":"no_decision"`, exit 1 |
| I1 | ADDRESSED | `blockers 4→3, landed: True` → `blockers_by_stratum {primary 1→3, bystander 3→0}, landed: False` |
| I2 | ADDRESSED | verdict on run 1 after v2 accepted → `subject_moved` "new subject version 2"; `check` stale lists run 1 |
| I3 | ADDRESSED | one merged row `ana \| 5 \| 3 \| 3` → `invalid_arguments` "a matrix is over one subject: run 1 is on site, run 2 is on other". The lens-version arm is covered by `i3_matrix_refuses_mixed_subjects_and_lens_versions` |
| I4 | ADDRESSED | `\| bystander \| cy \| 5 \|` → `\| bystander \| cy \| 5r \|` plus the legend line; `evidence` counts on each cell |
| I5 | ADDRESSED | an unjudged recall brief: `fixed_unverified` 2 → 0 (old) → still 6 (new; the new binary counts every batch's findings, so the totals differ) |
| I6 | ADDRESSED | `persona add` of `../../../../ESCAPED` → `invalid_arguments` "must match ^[a-z0-9][a-z0-9._-]{0,63}$"; no ESCAPED file written. Brief-time re-check and lens/subject/panel slugs are in tests |
| I7 | ADDRESSED (per ruling) | `should-run --paths LICENSE,prices/a.json` with classes `license`/`Money` under low: `quorum:"low"` → `human_class_under_persona_quorum` |
| I8 | ADDRESSED | the quoted diff: `quorum:"low"`, brand path missing → `quorum:"human"`, hits `brand/logo-é.svg` |
| I9 | ADDRESSED | `i9_about_separates_findings_and_dismissals`: two findings for two options; dismissing one option does not refuse the other; the dismissed option is still refused |

**Addressed 12 / 12. Not addressed: 0.**

Residual on C1 (Minor, not new breakage). The fix orders by run creation (`run.id > decision.after_run`), not by
when the verdict was ingested. Two cases still pass `check`:
1. Run 2 is briefed, then `fixed` is recorded, then run 2's verdict raises the same point.
2. `fixed` is recorded, then run 1 is re-ingested with the same point.

Both were reproduced in `rr/c1b` and `rr/c1c`: `check` returns `ok:true` and lists the finding under `fixed_unverified`.
The point is visible, but it is labelled "not yet verified" when it was in fact raised again. The review's alternative
(compare `decision.id` with the raising verdict row's id) would close both cases. This can wait for v0.2.

## Minors

Addressed (11 of 12), each with a test in `tests/final_review.rs` or a domain unit test:
- blank blocker
- stray pick (`pick_not_asked`)
- lens validation, including anchor_met checked per dimension
- `std::net` network guard
- MCP delta text
- error codes (`not_found`)
- attachment labels
- `--model` and `--stake`
- query under-resolution shown (`data.panel.terms`)
- `brief` refuses a subject that moved
- column renamed to `verdict.about`

Not addressed: the R12/B3 dismissed-key loopholes. The progress.md ruling parks them as provisional for the maintainer, so
leaving them open follows the ruling.

Release deferrals: the brief orphan run, `about` in the finding id (I9) and `lens list` showing `compares` are all
addressed.

## The three checks the brief asked for

**Schema v2 without a migration.** I built a v1 store with the old binary (personas, a panel, a subject, a run and a
verdict) and ran the new binary on it with `check`, `matrix 1`, `persona list`, `decide`, `subject add` and `doctor`.
Every verb was refused with `store_error` "… has schema_version Ok(1), this binary needs 2", exit 1. A sha256 of the
SQL dump was identical before and after: **refused cleanly, never half-migrated**. `Store::open` runs `SCHEMA` before
the version check, but every statement is `IF NOT EXISTS` and v2 adds no table, index or trigger, so nothing is written.
Two nits, both Minor:
- No test plants a v1 store. The guard has been seen to fail only by this manual run.
- The message prints `Ok(1)`, which is Debug formatting.

**The re-keyed `fixtures/run/decisions.jsonl`.** I replayed the fixture jury on both binaries (`rr/map.py`):
- Both give 26 unresolved findings, and the old and new files both have 26 decisions.
- All 26 old ids are present in the old `check` output, and they are distinct.
- Mapped through the old `check` output, the old decisions give the same multiset of (persona, dimension, text, args)
  as the new file. The outcomes and the D-row targets are unchanged.
- On the new binary, the (persona, dimension, about, text) keys are distinct, and each decision hits exactly one
  unresolved finding.

The replay asserts one hit per decision and `unresolved == 0` after the decisions are applied, so a decision that
mapped to the wrong finding would turn it red. It still proves 26 findings, each resolved by exactly one recorded
decision.

**Recorded verdicts.** The diff over `fixtures/run/verdicts` is empty. `matrix.json` equals main's once the new
`evidence` key is stripped (checked structurally), so no score moved. `matrix.md` only gains the legend line.

## New breakage in the fix diff

No new Critical or Important breakage.

New Minor: `paths_from_diff` (domain/trigger.rs) now reads `--- ` lines and fails on a quoted path, but it does not
track whether it is inside a hunk. A removed line whose content starts with `-- "` produces the diff line
`--- "quoted" comment`. Examples are an SQL, Lua or Haskell comment opening with a quote. The whole diff is then
refused with `invalid_diff` (reproduced in `rr/i8b/sql.diff`; the old binary returned `low`). A removed line
`-- a/brand/x.svg` adds a phantom path, which escalates to `human`. Both failures are fail-closed: the gate stops
rather than passing, and `+++ b/` had the same phantom-path weakness before this wave. Fix: read `---`, `+++` and
`rename` lines only between a `diff --git` header and the first `@@`.

## Out of scope (not part of this review)
- The fixer's report already lists docs that describe the old `check`, `fixed_unverified` and `landed` rules
  (troubleshooting, commands, usage:24, the running-a-jury skill, quickstart), and CHANGELOG has no schema v2 entry.
- The I7 plurals and synonyms (`secrets`, `claims`) are parked by ruling as a design limit.
- The median row of the matrix table has no evidence mark, even when every score under it is recalled. This follows
  the ruling ("a median never is"); the `evidence` counts in the JSON carry it.

## Verdict
**Ready for the maintainer's merge read.** 12 of 12 Critical and Important findings are addressed, 11 of 12 Minors are
addressed and the 12th is parked by ruling. The v1 store is refused cleanly, the decision re-key is one-to-one and
faithful, and no recorded verdict changed. The trigger parser Minor and the C1 ordering residual can wait for v0.2.
