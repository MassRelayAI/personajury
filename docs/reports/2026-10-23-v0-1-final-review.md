---
date: 2026-10-23
valid_until: 2026-11-06
verdict: "Not ready for the maintainer's merge read: 3 Critical and 10 Important findings, each reproduced on the debug binary. The refusals of report §5 hold line by line, but the gates built on top of them (check, delta, fixed_unverified, matrix, should-run) can each report a trustworthy-looking result that is not."
status: open
reviewer: final whole-branch review (Opus), read-only
scope: final-review-package.diff (engine src + lenses; setup.rs excluded), decisions D1–D7, progress.md rulings and deferred minors, docs/reports/2026-10-23-v0-1-measured.md
---

# PersonaJury v0.1: final review

`cargo test -p personajury`: 207 passed, 0 failed, the same as the measurement. Every Critical and Important finding below
was reproduced with `target/debug/personajury` in throwaway workspaces under
`/tmp/claude-1001/-mnt-extra-disk-repos/10cc8f43-f927-415f-a4e9-2945df54356e/scratchpad/rv/`. The repo was not modified.
The workspace helper `mk.sh` there builds the base case: personas `ana` and `cy` tagged `t`, panel `p = "2 t"`, subject
`site` with one `walk.json` (node `n1`), and the briefs use `--stake ana=primary,cy=bystander`.

## What holds

- `refuse::validate` (domain/refuse.rs:111–289) applies every §5 refusal in the order the plan fixes: a seen line must
  have a citation and every citation must resolve; an unheld `cmp:` may only be recalled; the scale, the dimension, the
  anchor and the bottom score are checked; there is one line per (dimension, about); stake, pick and reason are checked.
  I found no ingest path that skips `validate` (CLI and MCP both go through `envelope_for` → `verdict::ingest`).
- Citations resolve only against the run's own blobs and comparators (domain/citation.rs:14–63). ARIA ids and `L<n>` are
  checked; image and file locators are not, which D5 allows.
- Briefs are deterministic. Two runs on the same inputs give identical hashes (checked: run 1 and run 2 in `w6` have equal
  brief hashes). A brief carries no run id, no time and no model.
- The matrix has no field that spans strata, and `--across-strata` is refused. Medians are the lower middle, never a mean.
- Verdict, pick and decision are append-only through triggers (store/schema.sql:40–45), and no API path issues
  UPDATE or DELETE on them.
- No network crate is in the lock, and no secret env read appears outside setup.rs (which reads only HOME,
  XDG_CONFIG_HOME, PERSONAJURY_BIN, PATH and PERSONAJURY_NOW). The engine never spawns a process.

## Critical

### C1: `check` passes when a point decided `fixed` is raised again by a later run
- **Where:** application/check.rs:32–50 with store/mod.rs:667–675 (`INSERT OR IGNORE` on `finding.id`) and
  domain/hash.rs:235 (the finding id has no run in it).
- **Scenario (reproduced, `w2`):** run 1 has ana raise "the Go button does nothing" citing `walk.json#n1`. Someone
  records `decide <id> fixed --by agent` without changing anything. Run 2 is briefed on the unchanged subject and ana
  raises the same point with the same citation. `check` returns `ok:true`, `unresolved: []`, `fixed_unverified: []`,
  exit 0.
- **Why it matters:** this is the "fix did not land" signal of §10(2). The finding id is the same, so the old `fixed`
  decision covers the new raise, and the later run counts as having verified the fix. An uncited (told or recalled)
  blocker has the same id across subject versions too, so this case does not even need an unchanged subject.
- **Smallest fix:** in `check`, when the latest decision is `fixed` and the finding is raised by a run later than the
  run the decision was made on, list it as unresolved (`fix_not_landed`). Store the decision's run, or compare
  `decision.id` against the raising verdict row's `id`.

### C2: `delta` reports `landed: true` when the later run has no verdicts, or only some
- **Where:** domain/delta.rs:23–60.
- **Scenario (reproduced, `w2`):** `brief` creates run 3 and no verdict is ingested. `delta 1 3` gives
  `blockers_before: 1, blockers_after: 0, regressions: [], landed: true`. A cell that is missing after the fix becomes
  `after: null` and is never a regression, and a missing persona adds no blockers. The same happens when one of seven
  subagents answered.
- **Why it matters:** `landed` is the engine's verdict on whether a fix worked (§10(2)). Here it says yes with no
  evidence at all.
- **Smallest fix:** `landed` requires every (persona, lens, dimension, about) key of `before` to exist in `after`.
  Otherwise set `landed: false` and add `incomplete: [keys]`. Refusing a delta whose run b has no verdict rows is also
  reasonable.

### C3: re-ingesting a clean verdict removes a finding from the `check` gate
- **Where:** application/check.rs:26 (`findings_of(.., verdict_rows(r.id, false))`, latest batch only).
- **Scenario (reproduced, `w1`):** ana's verdict has a blocker and `check` fails with 1 unresolved. The same persona's
  verdict is then ingested again with score 3 and `blockers: []` (batch 2). `check` returns ok and exits 0. No decision
  was ever recorded. The `finding` row still exists, but `check` does not read it.
- **Why it matters:** §10(1) says "a finding with none of the three blocks the lane finish". Here a finding is cleared by
  overwriting it, which is the rewrite that append-only was meant to prevent, done one level up. The agent that wants
  the lane finished is the same agent that ingests verdicts.
- **Smallest fix:** derive `check`'s findings from every batch (`verdict_rows(id, true)`) or from the `finding` table
  for the run. A superseded batch's finding then still needs a decision (the "dismissed: superseded" decision is cheap
  and visible).

(C3 needs the ingesting agent to resubmit, so it could be argued down to Important. I keep it Critical because the
gate's whole value is that the agent cannot clear it without a recorded decision.)

## Important

### I1: `delta` sums blockers across strata: primary blockers can rise while `landed` is true
- **Where:** domain/delta.rs:50–53. The MCP description (adapters/mcp.rs:65) and the report both say "per stratum".
- **Scenario (reproduced, `w3`):** before, primary ana has 1 blocker and bystander cy has 3. After, ana has 3 and cy has
  0. Result: `blockers_before 4, blockers_after 3, landed: true`.
- **Why it matters:** this is a cross-strata number deciding the outcome (§2.3), and a primary blocker is exactly what
  §10 escalates on.
- **Fix:** count blockers per stratum (take the stakes from the runs). `landed` requires that no stratum's count rises
  and that primary does not rise. Return `blockers_by_stratum`.

### I2: a new subject version does not make old runs stale
- **Where:** application/verdict.rs:36–48 (`moved_why` rehashes only the pinned version's own source files).
- **Scenario (reproduced, `w4`):** run 1 is on `site` v1 (walk.json). `subject add site` with walk.json plus a new
  pricing.json creates v2 with a new hash. A verdict on run 1 is still accepted, and `check` lists no stale run. The same
  happens when v2 drops an attachment or changes `kind`.
- **Why it matters:** §5 refuses "a run on a subject whose hash moved since the brief". The subject hash did move.
- **Fix:** in `moved_why`, also compare `run.subject_hash` with `store.subject(name).hash` and report
  `subject has version vN` when they differ.

### I3: `matrix` over several runs merges different subjects and lens versions into one cell
- **Where:** application/matrix.rs:192–215 and domain/matrix.rs:35–43 (a later row replaces an earlier one, keyed by
  persona and lens name only).
- **Scenario (reproduced, `w4`):** run 1 is critical on `site`, where ana scores works=3. Run 2 is critical on `other`,
  where ana scores works=5. `matrix 1 2 --table` prints a single row, `ana | 5 | 3 | 3`, and the data names no subject.
  With two versions of one lens, each cell is emitted twice.
- **Fix:** refuse (invalid_arguments) a multi-run matrix whose runs differ in subject, or that holds two versions of a
  lens name. Alternatively, key cells by (subject, lens, version).

### I4: the matrix drops the evidence class: a `recalled` score shows as a plain number
- **Where:** domain/model.rs:467–480 (`Cell` has no evidence) and domain/matrix.rs:44–64 and :75–120.
- **Scenario (reproduced, `w6` run 3):** cy's `recalled` works=5 appears as `| bystander | cy | 5 |` and in the median,
  formatted exactly like a seen score.
- **Why it matters:** §2.2 says a recalled line is "shown as such". The `--table` the maintainer reads cannot tell memory
  from evidence.
- **Fix:** add `evidence: {seen, told, recalled}` counts to `Cell`, and suffix a recalled score in `render_table`
  (`5ʳ` or `5 (recalled)`).

### I5: `fixed_unverified` is cleared by any later run on the subject, even one never judged
- **Where:** application/check.rs:17–21 and :41–44 (`latest` is the max run id per subject, across lenses, with or
  without verdicts).
- **Scenario (reproduced, `w3`):** 4 findings are decided `fixed`, so `fixed_unverified` has 4. A `recall` brief on the
  same subject, with no verdict ingested, brings `fixed_unverified` to 0.
- **Fix:** a fix counts as verified only by a later run with the same lens that has verdict rows from the persona that
  raised the finding.

### I6: a persona id is used as a path, and a brief can be written outside the workspace
- **Where:** application/brief.rs:491–493 (`{dir_rel}/{id}--{lens}.md`). application/persona.rs:196–234 does not
  validate `id`, and application/lens.rs:8 does not validate `name`.
- **Scenario (reproduced, `w5`):** add a persona with id `../../../../ESCAPED`, then brief. The run writes
  `ESCAPED--critical.md` in the parent of the workspace (I removed the file afterwards). Persona files are written by a
  model under `persona generate`, so a model-chosen string decides where the file goes. An id or lens name containing
  `/` instead makes the write fail after the run row is inserted, which is the deferred minor about no rollback.
- **Fix:** at `persona add` and `lens add`, refuse an id or name that does not match `^[a-z0-9][a-z0-9._-]{0,63}$`.
  Panel names are not used in paths but take the same rule.

### I7: `should-run` matches the human list by exact spelling, so `license`, `Money` or `secrets` pass under `low`
- **Where:** domain/trigger.rs:27–33. docs/usage.md:45 states that these classes "always return `human`".
- **Scenario (reproduced, `w5`):** a rule `class = "license"` with paths `LICENSE*` and `quorum = "low"` is accepted, and
  so is `class = "Money"` on `prices/**`. `should-run --paths LICENSE,prices/a.json` returns `low`.
- **Why it matters:** D5 says "the toml cannot weaken the human classes", but spelling the class differently weakens
  it. US "license" is the common spelling.
- **Fix:** normalise the class before the lookup (trim, lower-case, `_`→`-`, drop a trailing `s`, map `license`→`licence`
  and `claim`→`public-claim`). Correct docs/usage.md. A further step is a test that walks a list of synonyms.

### I8: `should-run` drops git's quoted paths, so a non-ASCII file under a human class is not seen
- **Where:** domain/trigger.rs:47–64 (`strip_prefix("diff --git a/")` and `"+++ b/"` never match `"a/…"` in quotes).
- **Scenario (reproduced, `w5` and a real `git diff` in `gq/`):** git quotes `brand/logo-é.svg` by default
  (`core.quotePath`). Given a diff with that file plus `apps/web/a.ts`, the answer is `quorum: low`. The brand path
  appears in neither `hits` nor `unmatched`. With the brand file alone, the answer is `invalid_arguments`, which at least
  is not a pass.
- **Fix:** unquote C-style quoted paths (`"a/…"`, `"b/…"`, octal escapes). Refuse the diff when a `diff --git` header
  yields no path rather than dropping it silently.

### I9: `about` is in neither the finding id nor the dismissed key (deferred minor, raised)
- **Where:** domain/refuse.rs:212–250 and domain/hash.rs:235.
- **Scenario:** under `compare`, the same blocker text with the same citations on `about: home.png` and
  `about: cmp:acme` is one finding. One decision covers both, and dismissing the point on option A refuses it on
  option B.
- **Fix:** add `about` to `finding_id` and to the dismissed key. This must happen before the first real store exists,
  because finding ids are a persisted format.

## Minor

- **Blank blocker.** A bottom score accepts `blockers: ["  "]`, so the A5 "says what failed" check is met by whitespace
  (refuse.rs:156; reproduced in `w6`). Fix: treat blank blockers as absent.
- **Stray pick.** A pick under a lens that asks none is stored and shown by `pick` with an unchecked choice
  (refuse.rs:267–281; reproduced: `choice: "not-an-option"`). Fix: refuse `pick` when `!asks_pick`.
- **No lens validation at `lens add`.** A lens with zero dimensions is accepted, and a verdict with `lines: []` is then
  accepted (reproduced in `w6`). Also accepted: non-integer anchor keys (silently dropped), anchors outside `scale`,
  `scale[0] > scale[1]`, duplicate dimension ids, an empty `evidence` list. `anchor_met` is checked against the union of
  all dimensions' anchors (model.rs:203). Fix: validate in `Lens::from_toml`.
- **Dismissed key, both directions (ruling R12/B3).** Adding an unchecked citation (`shot.png#anything`) or citing the
  whole attachment instead of the node gives a new citation hash. The dismissed point is then accepted again with no new
  evidence (reproduced). In the other direction, a *different* blocker on the same dimension and citations is refused as
  "repeated" (seen in `w6` run 3). The ruling is provisional, so record this as evidence for the maintainer's open
  question. Fix: key on resolved citations of checked kinds only, plus the finding text.
- **Network guard covers crates only.** `tests/guards.rs` scans for network crates. `std::net::TcpStream` needs no crate
  and would pass. Fix: add a grep for `std::net` and `TcpStream` to the guard, with a planted test.
- **MCP descriptions overstate what delta checks.** `personajury_delta` says it "refuses runs that differ in lens or
  panel", but panel is not checked, and it says "per stratum", which it is not (see I1). Fix the text, or implement both.
- **Wrong error codes.** A missing `--attach`, `--file` or `verdict --file` returns `io_error` rather than `not_found`
  (should-run already maps NotFound).
- **Attachment labels are file names, unchecked.** A file named `a#b.json` cannot be cited. A file named `cmp:x.png`
  becomes an option starting `cmp:` and satisfies the A2 test at refuse.rs:206 without any comparator. Fix: refuse labels
  containing `#` or starting `cmp:`.
- **`--model` and `--stake` are not checked.** `--model ""` is accepted, so the run is pinned to an empty model.
  `--stake a=primary,a=bystander` keeps the last one silently. Fix: refuse both.
- **Query under-resolution is silent.** A query asking for 5 builders that resolves to 2 is accepted when `--spend` is
  omitted (the law only names `--spend`). Showing wanted and got in `brief` output would cost nothing.
- **`brief` does not check whether the subject already moved.** It composes from blobs even when the sources have
  changed, and the stale run is only refused at verdict time. A `moved_why` call at brief time is cheap.
- **Column name.** `verdict.comparator` stores `about`. Renaming it is free now and costs a migration after the first
  store.

## Deferred minors and rulings: triage

| item (progress.md) | verdict |
|---|---|
| walk link targets in node names, no `href` | can wait (v0.2); data shape only |
| `brief` inserts the run before writing files, no rollback | **fix before release**: with I6 an id makes the write fail, and an orphan run with no briefs still clears `fixed_unverified` (I5). Write the files first, or wrap in one transaction |
| `about` in neither finding id nor dismissed key | **fix before release** (I9): finding ids are persisted |
| `lens list` omits `compares` | fix before release (one line; agents choose lenses from this list) |
| A1 `brief#<section>` locator | can wait; the report does not require it |
| Task 3b doc minors | resolved: no doc outside plans and reports names `tests/refusals.rs` |
| Ruling: findings derived from latest-batch verdict rows (Task 2) | **overturn**: this is C3 |
| Ruling: check fails on unresolved findings of every run | right; keep it. C1 and C3 are the gaps in it |
| Ruling R12/B3 "same point" key | keep as provisional; add the two reproduced cases above to the maintainer's question |
| Ruling: delta mismatch is `invalid_arguments` | fine |
| Ruling R13 `check_failed` with data | fine |
| Ruling: no-network guard via lock scanner + deny.toml | fine for crates; add the `std::net` grep (Minor) |
| Ruling: `about` only under pick or compares lenses | fine; it removed the round-1 noise |
| Ruling: column `verdict.comparator` keeps its name | fine, but renaming now is free (Minor) |

## Verdict

**Not ready for the maintainer's merge read.** Line-level trust is sound: every §5 refusal holds and briefs are
deterministic. The gates above it are not: `check` passes after a failed fix (C1) or a resubmission (C3), and `delta`
says "landed" with no verdicts (C2) or with primary blockers up (I1). All Critical and Important fixes are local
(check.rs, delta.rs, verdict.rs `moved_why`, matrix guards, id validation, trigger normalisation). Each one needs a
red-first test from the scenarios above.
