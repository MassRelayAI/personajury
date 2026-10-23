# Decisions

Numbered from D1. A decision names who made it and why; a provisional one says what would change it.

## D1 — JuryGraph is built, under this name (2026-10-23, accepted by the maintainer)
The maintainer accepted the name JuryGraph on 2026-10-23. A model-free Rust engine with a
SQLite store, sibling of CompareGraph, Apache-2.0, binary and crate `jurygraph`, repository `massrelay-ai/jurygraph` (the
maintainer creates it and pushes). Design: the design report (kept privately by the maintainer; docs/design.md is its public summary).

## D2 — v0.1 dependencies (2026-10-23, orchestrator)
CompareGraph's set without tree-sitter (clap, rusqlite bundled, serde, serde_json, sha2, hex, thiserror) plus `toml` 0.8
(lenses and `jurygraph.toml` are TOML) and `globset` 0.4 (trigger paths). Both MIT/Apache or Unlicense/MIT. `deny.toml`
bans every HTTP client crate. Would change: a need the standard library cannot meet.

## D3 — spend has two knobs: how many, and how much each carries (2026-10-23, maintainer, mid-build)
The requirement: spend must be adjustable from low to high, and the material can range from technical to external to feature docs. The report gives the first knob (`--spend N` personas × the lenses chosen);
this adds `--detail compact|full` on `brief`, `matrix` and their MCP tools, as CompareGraph's `show` has. `compact` (the
default) lists attachments by path and hash; `full` inlines text attachments (an ARIA walk as lines, a diff, a document) up
to 64 KiB each. Every brief reports its byte size. "External" stays a snapshot attached as a comparator — never a live fetch
(report §2.2, §4). Would change: a measurement that compact briefs cost more in agent reads than full ones save.

## D4 — `decide` writes the decisions table (2026-10-23, orchestrator; open to the maintainer)
The build prompt's verb list has no writer for `decisions`, but report §4 stores them and §10(1) and §10(3) need them (a
finding with no decision blocks the finish; a dismissed point cannot return without new evidence). Added
`jurygraph decide <finding> fixed|dismissed|row [--why] [--row] --by <who>`. Open question 3 of the v0.1 hand-back.

## D5 — the shapes the report leaves to the build (2026-10-23, orchestrator, provisional)
| choice | alternative | what would change it |
|---|---|---|
| scores are integers on the lens scale, shipped lenses 1–5 with anchors in words at 1, 3, 5 | 1–10 like the hand juries | the maintainer's answer to open question 1 |
| `anchor_met` must be an anchor at or below the score | free | a lens with anchors that are not ordered |
| within a stratum the matrix shows every score, median, min, max; no arithmetic mean anywhere | a mean per stratum | none: §10 "nothing averages" |
| "one anchor" in delta = the lens's anchor spacing (2 on 1/3/5); `landed` = no cell down by more than that and blockers down (or none before and after) | one point | a run where 1-point drops matter |
| a finding = one blocker; id = 16 hex of sha256(subject, persona, dimension, text, citation hash); "the same point" = same persona, same citation hash, dismissed | text similarity | personas rewording a dismissed point to slip past |
| citation grammar `<label>[#locator]`, `cmp:<name>/<label>[#locator]`; ARIA node ids and diff lines `L<n>` are checked, image regions are not | check image regions | screenshots with region ids |
| pick options = the subject's attachment labels and `cmp:<name>` per comparator in the run | subjects as options | open question 2 |
| panel query tags match literally (lower-cased); `others` = none of the query's other tags; first term wins a persona | stemming ("builders" = "builder") | personas tagged in both forms |
| `--stake` is required for every panel persona (the spend order needs all stakes) | only the spent ones | panels of 15+ where typing stakes is a burden — then a persona's `stake_rules` could default it |
| `check` exits 1 on an unresolved finding; a stale run is listed, not failed | fail on stale | a lane that finishes on a stale run |
| the four shipped lenses are seeded into every new store | `lens add lenses/*.toml` by hand | a project that wants none of them |
| human classes fixed in the engine: money, permission, deploy, secret, licence, public-claim, brand | a list in the toml | none: the toml cannot weaken the human classes |

## D6 — the design author's second read: six seams closed in v0.1 (2026-10-23, Fable read, orchestrator ruling)
Fable read the plan against the report (the maintainer's notes, copied to
`docs/reports/` with the v0.1 measurement). Adopted for v0.1:
- A2: under a lens that asks for a pick (`compare`), a `seen` line must name a comparator the run holds or carry a `cmp:`
  citation — "a comparison with no comparator snapshot cannot be seen" (report §2.2).
- A3: a line may be `about` one option (an attachment label or `cmp:<name>`); the rule of D5 "one line per dimension"
  becomes "one line per (dimension, about)", and every dimension still needs at least one line. Matrix and delta cells are
  keyed (lens, dimension, about). This is what lets a persona "pick one and score all" (report §3, three logo files).
- A4: `delta` refuses two runs on different lens versions, not only different lens names (report §2.6 pins the version).
- A5: a score at the bottom of the scale with no blocker is refused (`bottom_score_without_blocker`): otherwise the worst
  verdict produces no finding and escapes the §10(1) gate.
- A6: the median of an even count is the lower middle value, an integer — no arithmetic mean anywhere (§10 "nothing averages").
- B3: "the same point" (§10(3)) = same persona, same dimension, same citation hash (an uncited point: the same finding id).
  `check` also lists `fixed_unverified`: findings decided `fixed` whose subject has no later run (§10(2)).
- C: `detail` is also a `jurygraph.toml` rule field (default compact), so the trigger rule sets the brief's weight where it
  sets the run's size.
Deferred: A1 (a `brief#<section>` locator for `told` lines) — the report does not require told lines to cite.
Not built, left to the maintainer: the detector's target and the split rule (Fable recommends a hand-scored jury table the maintainer keeps privately (six criteria on a 1–10 scale) with a 1–10
`site` lens shipped as fixture data, and "two strata whose anchor_met ranges do not overlap" — open questions 1 and 2).

## D7 — serde_json `arbitrary_precision` (2026-10-23, orchestrator)
`setup --apply` rewrites the user's own agent configs (additive, backed up). Without `arbitrary_precision` a large integer
in such a file is rewritten as a float (`1e+23`): a lossy edit of a file we do not own. CompareGraph enables the feature
for the same reason. A feature of an existing dependency, no new crate. Would change: serde_json keeping integers exact by
default.

## D8 — the final review's fixes: store schema v2 and what the gates trust (2026-10-23, orchestrator rulings)
The final review of v0.1 (`docs/reports/2026-10-23-v0-1-final-review.md`, re-review `docs/reports/2026-10-23-v0-1-final-rereview.md`) found gates that could report a
pass that never happened. The orchestrator's rulings, built in lane `final-fixes`:
- `check` reads the findings of every batch (overturns the Task 2 ruling): a re-ingest without the blocker never clears it.
  A finding whose latest decision is `fixed` and that a run created after the decision raises again is unresolved
  (`reason: fix_not_landed`). `fixed_unverified` clears only on a later run on the same subject and lens in which the
  persona that raised it has verdict lines. Each unresolved item carries `about` and `reason` (`no_decision` or
  `fix_not_landed`).
- `delta`: blockers per stratum (`blockers_by_stratum`); `landed` needs no regression, no stratum whose blockers rose,
  every (persona, lens, dimension, about) cell of the earlier run present in the later one (else `incomplete`), and
  blockers down or none either side.
- A run is stale also when its subject has a later version with another hash (`new subject version <n>`).
- The finding id is `sha256(subject \t persona \t dimension \t about \t text \t citation_hash)[..16]`, about empty for the
  subject as a whole; the dismissed key of a cited point is `dimension \t about \t citation_hash` (supersedes D5's id).
- Schema v2 (`store/schema.sql`): `verdict.comparator` renamed `about`; `finding.about` added; `decision.after_run` (the
  highest run id when the decision was recorded: a run above it was created after the decision). No store predates v0.1,
  so there is no migration: a v1 store is refused by its `schema_version`.
- Ids and names (persona, lens, subject, comparator, panel) match `^[a-z0-9][a-z0-9._-]{0,63}$` without `..`.
Would change: the maintainer's answer on the "same point" key (R12/B3), which stays provisional.
