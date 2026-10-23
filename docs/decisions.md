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

## D9 — schema v3, the seeded draw, a stake per panel term, `brief --rule` (2026-10-24, the maintainer's call on the design author's read)
The persona-structure read (kept privately by the maintainer) found that a rule already carries `panel`, `lenses`, `spend` and
`detail`, but `brief` did not read it, the panel resolved in persona-id order (the same bystanders on every subject) and
every stake was typed by hand. v0.2 builds option A1:
- **Schema v3** (`store/schema.sql`): `run.draw_seed TEXT` (the hex seed the draw used) and `run.override_json TEXT`
  (D10). A v2 store is refused by its `schema_version`; no store predates v0.2, so there is no migration.
- **The seeded draw.** For every panel term whose pool is bigger than its seat count, the seats are drawn by a Fisher-Yates
  shuffle of the pool (sorted by id) seeded from the first 8 bytes of `sha256("draw\t" + subject name + "\t" + panel name)`.
  The seed comes from the subject's NAME, not its hash: a hash moves with every edit, and `delta` (D8) needs the same
  (persona, lens, dimension, about) cells in both rounds. The generator is SplitMix64 in `domain/rng.rs`, standard library
  only, no new dependency. A term whose pool equals its seats is not shuffled, so every v0.1 brief is byte-identical
  (`tests/fixture_replay.rs` is unchanged and green). `brief --draw HEX` overrides the seed; the run records the one used.
  `panel resolve` without a subject keeps the id order and says `draw: none (no subject)`.
- **A stake per panel term** (`3 builder=primary`): the stake belongs to the audience, not the persona. `--stake` stays,
  wins over the term, and is needed only where a term has none. Supersedes the D5 row "`--stake` is required for every
  panel persona".
- **`brief --rule <class>`** takes panel, lenses, spend and detail from the rule and writes one run per lens. A rule with
  quorum `low` or `high` must name a `panel` and lenses (`rule_without_panel`); the other new refusals are
  `rule_panel_unknown`, `rule_lens_unknown` and `panel_term_empty` (a "public" panel that silently became an insiders'
  panel).
- `#L<n>` on a UTF-8 text attachment is checked against its line count, as a diff's already was; `should-run --subject`
  classifies a subject's recorded source paths and warns about a panel or lens the store lacks; the `persona generate`
  brief asks for the roles, sectors and objections first (an entity pass; the count stays the caller's `--count`).
Would change: a saturation run (brief 12, count new findings per added persona) that moves the ceiling of D10; a need to
rotate bystanders between rounds, which would break `delta` and so needs its own decision.

## D10 — seat ceilings 12 and 1000, and the agent's overrides with a reason (2026-10-24, the maintainer)
The maintainer, 2026-10-24: "allow llm to override the persona count, type, etc on top of us for flexibility … if they want
to run 1000 maximum also can right? in a crazy sense". Built as:
- **Two ceilings** (`domain/model.rs`): `SEAT_CEILING = 12` and `SEAT_HARD_MAX = 1000`. A panel (`panel add`) or a brief
  (`brief`) above 12 seats is refused as `panel_too_large` unless `--allow-large`; above 1000 it is refused always, with
  `error` data `{ceiling: 1000}`. The 12 is a guess until the saturation run of D9 measures it.
- **Overrides on top of the rule.** With `--rule`, the caller (the agent) may change the panel (`--panel`, or an ad-hoc
  `--query` that is not saved), the lenses, the spend and the detail. Any value that differs from the rule, and any brief
  above 12 seats, needs `--reason` (`override_without_reason`) and is recorded on every run it made as
  `run.override_json = {rule, fields:{name:{rule,given}}, reason, by}` (`--by`, default `agent`). Nothing is silent:
  `matrix` JSON carries `override`, and `matrix --table` opens with `override: <fields> — <reason>`. With no `--rule` there
  is no override bookkeeping (v0.1 behaviour).
- **The cost is shown before it is spent.** Every brief answers `estimate {briefs, tokens}` = briefs × `tokens_per_brief`
  (`jurygraph.toml`, default 15000) and `large: true` above 12 seats. The engine still counts no token and calls no model.
- **The human classes are untouched.** No override can run a jury on a human class (or quorum `human`) as if it decided:
  the refusal of D5 stands. Strata are never averaged at any size.
Would change: the maintainer's word; or a measured ceiling that is not 12.
Fix round 1 (review of 2026-10-24, the maintainer's rulings):
- Under `--rule`, `--stake` (when it differs from the panel's term) and `--draw` (when it differs from the derived seed) are
  overrides like the others: they need `--reason` and are recorded as `stakes` / `draw` fields of `override_json`. The order
  of lenses is not a departure.
- Each term draws from its own stream, seeded from the run's seed and the term's tag (`hash::term_seed`), so a persona
  added to one term never moves another term's seats. The ad-hoc query is seeded by its normalised terms.
- `brief --rule` classifies the subject's recorded source paths: a path under a human class is refused
  (`human_class_under_persona_quorum`, data `{class, path}`) under any rule and any override.
- A rule's `spend = 0` is the whole panel capped at 12 (1000 with `--allow-large`, which then needs a reason above 12); the
  seats beyond the cap are cut in draw order and the answer says `capped: true`. It is not a refusal.
- Seat totals and the estimate use checked arithmetic; an overflow is `panel_too_large`, never a panic and never a saved 0.
- Only `L<n>` locators of a text file are checked (canonical form: no sign, no leading zero; a diff's too); any other
  anchor behaves as in v0.1, and a file with a NUL byte is not text.
- The ceiling is applied to an ad-hoc `--query` by its total seats, to a saved panel at `panel add` and then by the seats a
  brief selects. A brief above 12 seats without `--rule` records nothing (no rule, no bookkeeping): the maintainer has not
  yet confirmed that reading of the amendment.

## D11 — the jury graph: one deterministic SVG, and its layout rule (2026-10-24, the maintainer asked for it by name)
The name promises a graph and v0.2 drew none. v0.3 adds `jurygraph graph RUN... --svg [--out F]` and the MCP tool
`jurygraph_graph` (the tool count goes from 5 to 6; it answers the SVG text in `data.svg` and writes no file).
- **One static picture, no simulation.** The layout is integer arithmetic over ordered containers (`domain/graphlayout.rs`,
  pure and unit-tested; the string building is `application/graph.rs`, no new crate): the same store gives byte-identical
  output, and `tests/graph.rs` says so. The chosen alternative was a force layout, which needs floats and an iteration
  count and moves whenever a node is added.
- **The layout rule.** The card is 1200 wide and grows in height with the rows. Left to right: three stratum columns of
  persona nodes (name, id, a stake pill), a column per lens of dimension nodes with the `@about` options as sub-nodes, and
  the findings (short id, the first words of the blocker with an ellipsis, the full text as the tooltip). The stratum
  columns descend like stairs (each starts under the one before it, below its own title) so that an edge from one
  stratum never runs behind another stratum's boxes; row heights are computed from the content, labels are cut to the
  width they have, and a test lays out 30 personas and asserts no two boxes touch. Every dimension node carries one
  median badge per stratum (P S B) and never one combined number; the medians are the matrix's, taken from
  `domain::matrix::build`, not recomputed.
- **Colour.** An edge persona to dimension takes one of five steps of the README's blue scale (and a heavier line for a
  higher score), the number is its tooltip. An edge persona to finding is red (open), green (fixed), grey and dashed
  (dismissed) or blue (row); a `fixed` finding that a later run raised again is open, as in `check`.
- **Rounds.** Several runs of one subject: scores and medians come from the latest run of each lens, and when a lens was run
  more than once every finding carries a strip of dots, one per run of its lens: raised, raised again after a fix
  (`fix_not_landed`), fixed (absent after a fix, the persona judged), not judged, absent. The lives come from
  `check --history` (`data.lives`, one entry per finding with `status`, `decision` and `life`), which is the same function.
  Runs of two subjects are refused.
- **The asset.** `docs/assets/jury-graph.svg` is the graph of the recorded fixture run; `tests/graph_asset.rs` rebuilds the
  workspace as the replay does and fails when the committed file differs from a fresh render (`JURYGRAPH_ASSET_REGEN=1` rewrites it; the replay outputs have their own `JURYGRAPH_FIXTURE_REGEN`). It looks at the SVG
  through Chrome headless at 1200 px and at 800 px.
Fix round 1 (review of 2026-10-24): above 30 personas the strata sit side by side with their own heights (stairs make the card as tall as the sum of the strata); a life strip draws the last 8 runs with a `+N` mark and a tooltip for the rest; SVG text drops the characters XML 1.0 forbids; a finding's life is rebuilt from every `fixed` decision, so a second `fixed` never hides a `fix_not_landed` step.\nWould change: a collapsed stratum for a very large jury, or a third lens that
wants its own lane (more than two lenses already stack in two lanes).

## D12 — waves, reported usage, the allowed anchors, a reason for any large brief (2026-10-24)
- **Waves.** Every brief answers `waves`: its persona ids in brief order, cut into waves of `--wave N` seats (default 30;
  one wave up to 12 seats; `--wave 0` is `invalid_arguments`). The engine runs nothing, so the throttle is advice the skill
  gives: run a wave, ingest it, then the next. A 100-seat brief answers 30/30/30/10.
- **Reported usage.** A verdict may carry `usage {input_tokens, output_tokens}`: two non-negative integers and nothing else,
  else `invalid_usage` (red first); each value must also fit in i64 (a larger one is refused, sums saturate). It is stored in a new append-only table `usage` (`CREATE TABLE IF NOT EXISTS`, so a v3
  store gains it on open and the schema version stays 3; no column of `verdict` changed). `matrix` and `doctor` answer
  `usage {reported, estimate}`: `reported` sums every verdict that carried one and is labelled "reported by the agent",
  `estimate` is briefs × `tokens_per_brief`. The engine still measures no token.
- **The allowed anchors are in the brief.** `bad_anchor` was the commonest refusal in two real runs (the logo jury). The
  answer-shape rules now say, per lens, `anchor_met: one of 1, 3, 5, at or below your score`. This changes the bytes of
  every brief: the 14 recorded hashes of `fixtures/run/briefs.json` were regenerated (14 of 14 changed) and so was the
  one compare-brief hash in `tests/authoring.rs`; the recorded verdicts, matrix and picks are untouched and
  `fixture_replay` is unchanged and green.
- **A large brief needs a reason, rule or not (closes review M9 of v0.2).** A brief above 12 seats without `--rule` now needs
  `--reason` (`override_without_reason`) and records `override_json = {rule: null, fields: {seats: {rule: 12, given: N}},
  reason, by}`; `matrix` shows it (`override: seats 12→N — reason: …`).
- **A malformed `jurygraph.toml` is refused** by `brief` and `matrix` (`invalid_config`) instead of silently ignored (M7); `doctor` reports it as a limitation item and still answers.
- `--comparator` takes a comma list as `--lens` does, and still repeats; an empty element is `invalid_arguments`.
Would change: a measured wave size (the 30 is the mirror of a rate limit, not a measurement); an agent that cannot report
usage, which leaves `reported` null.
