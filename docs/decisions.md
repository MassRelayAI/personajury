# Decisions

Numbered from D1. A decision names who made it and why; a provisional one says what would change it.

## D1 — PersonaJury is built, under this name (2026-10-23, accepted by the maintainer)
The maintainer accepted the name PersonaJury on 2026-10-23. A model-free Rust engine with a
SQLite store, sibling of CompareGraph, Apache-2.0, binary and crate `personajury`, repository `massrelay-ai/personajury` (the
maintainer creates it and pushes). Design: the design report (kept privately by the maintainer; docs/design.md is its public summary).

## D2 — v0.1 dependencies (2026-10-23, orchestrator)
CompareGraph's set without tree-sitter (clap, rusqlite bundled, serde, serde_json, sha2, hex, thiserror) plus `toml` 0.8
(lenses and `personajury.toml` are TOML) and `globset` 0.4 (trigger paths). Both MIT/Apache or Unlicense/MIT. `deny.toml`
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
`personajury decide <finding> fixed|dismissed|row [--why] [--row] --by <who>`. Open question 3 of the v0.1 hand-back.

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
- C: `detail` is also a `personajury.toml` rule field (default compact), so the trigger rule sets the brief's weight where it
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
*Built in v0.2: [plan](plans/2026-10-24-v0-2-panels-and-the-draw.md), [measured](reports/2026-10-24-v0-2-measured.md).*
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
*Built in v0.2: [plan](plans/2026-10-24-v0-2-panels-and-the-draw.md), [measured](reports/2026-10-24-v0-2-measured.md).*
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
  (`personajury.toml`, default 15000) and `large: true` above 12 seats. The engine still counts no token and calls no model.
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
*Built in v0.3: [plan](plans/2026-10-24-v0-3-the-jury-graph.md), [measured](reports/2026-10-24-v0-3-measured.md).*
The name promises a graph and v0.2 drew none. v0.3 adds `personajury graph RUN... --svg [--out F]` and the MCP tool
`personajury_graph` (the tool count goes from 5 to 6; it answers the SVG text in `data.svg` and writes no file).
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
- **The asset.** `docs/assets/personajury-graph.svg` is the graph of the recorded fixture run; `tests/graph_asset.rs` rebuilds the
  workspace as the replay does and fails when the committed file differs from a fresh render (`PERSONAJURY_ASSET_REGEN=1` rewrites it; the replay outputs have their own `PERSONAJURY_FIXTURE_REGEN`). It looks at the SVG
  through Chrome headless at 1200 px and at 800 px.
Fix round 1 (review of 2026-10-24): above 30 personas the strata sit side by side with their own heights (stairs make the card as tall as the sum of the strata); a life strip draws the last 8 runs with a `+N` mark and a tooltip for the rest; SVG text drops the characters XML 1.0 forbids; a finding's life is rebuilt from every `fixed` decision, so a second `fixed` never hides a `fix_not_landed` step.\nWould change: a collapsed stratum for a very large jury, or a third lens that
wants its own lane (more than two lenses already stack in two lanes).

## D12 — waves, reported usage, the allowed anchors, a reason for any large brief (2026-10-24)
*Built in v0.3: [plan](plans/2026-10-24-v0-3-the-jury-graph.md), [measured](reports/2026-10-24-v0-3-measured.md).*
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
- **A malformed `personajury.toml` is refused** by `brief` and `matrix` (`invalid_config`) instead of silently ignored (M7); `doctor` reports it as a limitation item and still answers.
- `--comparator` takes a comma list as `--lens` does, and still repeats; an empty element is `invalid_arguments`.
Would change: a measured wave size (the 30 is the mirror of a rate limit, not a measurement); an agent that cannot report
usage, which leaves `reported` null.

## D13 — the split rule and `apply-check`, the verdict an agent obeys (2026-10-24, the maintainer: "decide automatically")
*Built in v0.4: [plan](plans/2026-10-24-v0-4-the-apply-check.md), [measured](reports/2026-10-24-v0-4-measured.md).*
Closes open question 3 of `docs/reports/2026-10-23-v0-1-measured.md` (what counts as a split).
- **A split** is a (lens, dimension, about) cell on which two strata's `anchor_met` ranges `[min, max]` do not overlap. Ranges
  that touch at an endpoint overlap: no split. Pure code in `domain/apply.rs`. `matrix` answers `splits` beside `strata`
  (`{lens, dimension, about, strata: [{stake, min, max}, {stake, min, max}]}`, one per disagreeing pair); it is a sibling key,
  never a field of a stratum or a cell, so `Matrix` still has no field that spans strata and the recorded fixture matrix and
  table are unchanged.
- **`personajury apply-check RUN... [--rule CLASS]`** (named `apply-check`, `verdict-of` reads badly) and the MCP tool
  `personajury_apply_check` (seven tools) answer `apply | escalate | human | advise` with the reasons as data. The latest given
  run of each lens is judged; earlier given runs are the rounds before it, and the previous round of the same subject and lens
  is found in the store.
- **The principle (the maintainer's ruling, 2026-10-24, after the v0.4 review): `apply` only when every condition is proven;
  anything unknown, missing or lowered answers `advise` (or worse), never `apply`.** The full rule:
  - **The quorum** is the higher of the quorum recorded on the judged runs, the `--rule` named to apply-check, and the highest
    `low`/`high` among the rules the subject's recorded sources fall under (what `should-run --subject` gives). `brief --rule`
    stores the run's rule class and its effective quorum (the higher of the rule's and the sources') as `run.rule_class` and
    `run.quorum` (schema v4, the same idempotent migration as D17/D18), so dropping `--rule` later never lowers it. `quorum_source`
    names where the highest came from (`sources`, `run`, `rule`). With none of the three the quorum is unknown: `quorum: null`,
    `quorum_unknown`, `advise`.
  - `human`: the rule named with `--rule` is a human class (or quorum `human`), or the subject's recorded source paths fall under one.
  - `escalate` (the answer names `next_quorum`, low to high, high to human):
    `primary_blocker`: a blocker in the primary stratum of any judged run, each run read with its own stakes (a persona primary
    in one lens run and secondary in another is primary where it was; no stakes map is shared across runs);
    `split`: two strata disagree on a cell within one run; `check_failed`: a finding of any run of this subject has no decision
    or a fix that did not land (not only the runs given); `delta_not_landed`; `human_blocker`: a blocker raised by a
    `--by-human` batch no decision has closed.
  - `advise`: `incomplete_jury` (a briefed seat of any given run has no accepted verdict; a human row does not fill a seat;
    `missing` lists the seats per run); `quorum_unknown`; `rule_below_sources` (the `--rule` named here, or the run's recorded
    rule, has a lower quorum than the sources'; at `brief` such a rule is an override, field `quorum`, that needs `--reason`);
    `overridden_down` (the run's recorded override lowers `spend` below the rule's, drops a rule lens, or sets `detail` below
    it; or replaces the rule's panel, stakes or draw, which is not proven equal; or the run's effort is below the toml's
    `effort`, `medium` when unset; or the run was briefed with no rule while a rule holds the subject); `effort_below_quorum`
    (D17); `independence_sequential` (D18); `thin_primary` (added after the v0.6-A review: a judged run whose primary stratum
    has fewer than two judged seats; one reader is not a jury, so a `--seats auto` run with one serving persona never applies);
    `stale_run` (the subject moved since the run, or a later run of the same subject and
    lens supersedes it); `delta_unavailable` (a previous round of the same subject and lens exists but the delta cannot be
    computed: another lens version, or cells the previous round does not have).
  - Raising seats, lenses, effort or detail above the rule is fine. `apply`: none of the above. `single_model` is informational.
  - Precedence: `human`, then `escalate`, then `advise`, then `apply`. Every condition is a row of `tests/apply_check_table.rs`,
    each seen red; the review's twelve probes are `tests/v04_review_probes.rs`.
- It applies nothing itself and writes nothing. The skill: the agent runs it last and obeys it. The recorded fixture run answers
  `escalate` (it has primary blockers); a test pins that.
Would change: a measured false-escalation rate; the calibration lane (v0.5), which can turn `apply` into `escalate`.

## D14 — `lens generate` (2026-10-24)
*Built in v0.4: [plan](plans/2026-10-24-v0-4-the-apply-check.md), [measured](reports/2026-10-24-v0-4-measured.md).*
`personajury lens generate --from PATH [--dimensions N]` (default 4, at most 12) writes `.personajury/generate/lens-<hash>.md`
the way `persona generate` writes its brief, with the same material hash and the same record in `material`. Two steps for the
agent: list what the material says good looks like (not part of the lens file), then draft a lens in the exact TOML shape, with
the anchors in words at 1, 3 and 5 and `standard = "material:<hash>"`. The brief's example is a valid lens (a test adds it with
`lens add`). The engine calls no model; `lens add` validates. The three sector lenses in `fixtures/lenses/` (consulting deck,
investor pitch, financial model: four dimensions each, anchors in words, generic) are loaded with `lens add`, never seeded, and a
test checks that `lens add` accepts them.

## D15 — a model per panel term (2026-10-24)
*Built in v0.4: [plan](plans/2026-10-24-v0-4-the-apply-check.md), [measured](reports/2026-10-24-v0-4-measured.md).*
A term may close with `@model`: `3 builder=primary@claude-sonnet-5-5, 2 approver=secondary@other-model`. The model is letters,
digits and `. _ : / -`, else `invalid_query`. `brief --model` stays the default and the run's `model`. Each brief records its model
(`BriefRef.model`, in `run.briefs_json`; a run made before v0.4 reads as the run's model). **A run whose panel terms name a model
states, in every one of its briefs, `Run this brief on the model <m>`; a run with no term model has the briefs it always had**, so the
recorded fixture hashes did not change. `matrix` answers `models` per stratum and `models_distinct`; `apply-check` adds the
informational reason `single_model` when every juror used one model.

## D16 — packaging: the `.mcpb` bundle, `mcp.json`, registry packages (2026-10-24)
*Built in v0.4: [plan](plans/2026-10-24-v0-4-the-apply-check.md), [measured](reports/2026-10-24-v0-4-measured.md).*
No engine change. `packaging/mcpb/manifest.json` (manifest 0.4, `server.type` `binary`, `mcp_config.command`
`${__dirname}/bin/personajury`, args `["--root", "${user_config.root}", "mcp"]`, a required `directory` setting) and
`scripts/build-mcpb.sh <platform> <binary>` (zip of the manifest for that platform and the binary; prints the sha256; no network).
`release.yml` builds four bundles (linux-x64, darwin-arm64, darwin-x64, win32-x64), validates them with a pinned
`@anthropic-ai/mcpb` (the one place the build touches the network), and gains `publish-registry` ported from CompareGraph's
(`mcp-publisher` pinned and checksum-verified, `login github-oidc`, `publish`) after the bundles' hashes are written into
`server.json`'s `packages`. `mcp.json` at the root (never `.mcp.json`, which Claude Code would load in a clone) is referenced by
the Codex (`"./mcp.json"`) and Cursor (`"mcp.json"`) manifests; `gemini-extension.json` carries `mcpServers` (the key is unverified
on a local reference). `tests/manifests.rs`: one name, one version (Cargo.toml's), one command across all of them, the `.mcpb`
args run against the binary, the script run on a stand-in binary. `tests/guards.rs` gains a no-listener scan (`bind(`, `listen(`).
Unverified against Claude Desktop until the maintainer installs a bundle by hand. A hosted connector is never built: the chat route
is files (`docs/usage.md`).

## D17 — effort: one word for how much a jury may spend (2026-10-24, the maintainer: "keep it simple")
*Built in v0.4: [plan](plans/2026-10-24-v0-4-the-apply-check.md), [measured](reports/2026-10-24-v0-4-measured.md).*
`effort = "low" | "medium" | "high" | "max"` in `personajury.toml`, `brief --effort` per run (recorded), and an optional
`budget_tokens`. The rule says what a change needs; effort says what you will spend; the run gets the smaller.
| effort | seats | lenses |
|---|---|---|
| low | 3: one per stratum present, primary first, then the next in draw order | the rule's first |
| medium | the rule's, at most 7: one per stratum present first, then primary first, in draw order | the rule's |
| high | the rule's, at most 12, the same way | the rule's |
| max | the rule's (above 12 needs `--allow-large` and `--reason`, as ever) | the rule's |
- **When it applies.** A brief under `--rule`, or with `--effort`, or with `effort` set in the toml is planned; the default under a rule
  is `medium`. A brief with none of them is as in v0.3 (nothing is recorded). An explicit `--spend` is the caller's own choice and is
  not cut by effort; an explicit `--lens` list is not cut to its first. **Consequence for existing projects:** a rule with more than
  7 seats is now cut to 7 by default; set `effort = "high"` (12) or `"max"` to keep the v0.3 behaviour. One existing test uses
  `--effort high` for this reason.
- **`budget_tokens`** fits the run under briefs × `tokens_per_brief`: lenses after the first go first, then seats from the end of the draw
  order, never the last seat of a stratum; if one seat per stratum and one lens do not fit, `over_budget` with data
  `{needed, budget, minimum_plan}` and nothing written. Every cut is recorded (`trimmed: [{what, from, to, why}]`), answered by
  `brief`, kept in the run and shown by `matrix` (JSON `effort`, `trimmed`; a header line `effort: low, trimmed: ...`, left out for the
  default `medium` with nothing cut).
- **A cap never drops a stratum** (ruling after the v0.4 review): every effort cap keeps one seat per stratum the panel has,
  then fills primary first (then secondary, then bystander) in draw order, so a split stays possible. `low` was already so.
- **A cheap jury advises.** On a `high` quorum, effort `low` or any cut makes `apply-check` answer `advise`
  (`effort_below_quorum`), never `apply`.
- `doctor` shows `effort {configured, default}` and `usage_7d`, the tokens reported over the last 7 days.
- **Schema v4** (`store/schema.sql`): `run.effort`, `run.trimmed_json`, `run.independence` (D18), `run.rule_class` and `run.quorum` (D13). A v3 store, or a v4 store made before the last two, gains them on open by an
  idempotent `ALTER TABLE ... ADD COLUMN` and its `schema_version` becomes 4; every old run reads as `independent`, no effort.
  A store of any other version is refused as before.

## D18 — independence: a word on the run (2026-10-24)
*Built in v0.4: [plan](plans/2026-10-24-v0-4-the-apply-check.md), [measured](reports/2026-10-24-v0-4-measured.md).*
Where no subagents exist (Claude Desktop, a chat window, Copilot) one context plays every juror, and the engine cannot see it. So it is
a declaration: `brief --independence independent|sequential` (default `independent`; MCP `independence`), stored as `run.independence`.
The brief files are byte-identical either way. The answer echoes it and, for `sequential`, adds `sequence` (the brief paths in panel
order) and "one context will read all of these; the run is labelled sequential". `matrix` answers `independence`
(`independent | sequential | mixed`) and opens a table with `independence: sequential` when any run is; `delta` answers
`independence {a, b}`; `apply-check` answers `advise` (`independence_sequential`, remedy: re-run the same quorum with one context per
brief) and never `apply`; `escalate` and `human` take precedence. `docs/publishing-results.md` lists it beside the hashes. Claiming
`independent` from one window is a false record.

## D19 — experts and a human verdict (2026-10-24)
*Built in v0.4: [plan](plans/2026-10-24-v0-4-the-apply-check.md), [measured](reports/2026-10-24-v0-4-measured.md).*
Expertise is a property of a persona or of a lens, never a stratum. A persona file may carry `credential` (words, at most 500
characters, else `credential_too_long`; shown in the brief, `persona list` and the matrix as `(C)`; the key is left out of an older
persona's profile and hash). A lens's `standard` is shown in every brief and in a `standard: <lens> — ...` header line of the matrix.
`verdict --by-human "<name, credential>"` ingests a person's batch under the same refusals; the file's `persona` is the seat whose
stratum the person speaks in; rows are stored as `human:<name>:<seat>` and who entered them in the new append-only table
`human_batch`. They stand beside the jury's rows (matrix JSON `human`, an `H` block under the table), never in a persona's cell, never
in a median, never in a split. The MCP verdict tool has no `by_human`: an agent never enters a person's verdict. A blocker raised by a
human batch makes `apply-check` answer `escalate` (`human_blocker`) whatever the stratum, until a recorded `decide` closes the finding;
that decision is the only way a human verdict is overruled, and `decide` takes it only with `--by-human "<name>"` (a `--by` of
the agent, the foreman or anyone is refused, `human_decision_required`; the MCP has no `decide`). `calibrate` (v0.5) will compare human batches with persona batches.

## D20 — calibration (2026-10-24)
*Built in v0.5: [plan](plans/2026-10-24-v0-5-summary-calibration-ci.md), [measured](reports/2026-10-24-v0-5-measured.md).*
`personajury calibrate <jury-run> <human-run> [--out F]`. The human run is an ordinary run whose batches were entered with `--by-human`
(`not_human_run` otherwise) on the same subject and lens version. For each stratum, the cells (lens, dimension, about) are ranked by the
stratum's median (the lower middle), high first, ties sharing a rank (1, 1, 3); `within_one_rank` per cell. **`calibrated` is true iff every
primary-stratum cell is within one rank**, and every primary cell the jury judged was compared (a cell no person judged is not proven).
Differences on the other strata are listed (`outside_primary`) and never change `calibrated`; no number spans strata. `apply-check
--calibration F` reads the `--out` file: `calibrated: false` is the advise reason `uncalibrated` (so `apply` becomes `advise`; `escalate`
and `human` still take precedence). The roadmap said "escalate"; this is `advise` by the principle of D13 (an unproven jury advises, it does
not send the change up a quorum). A file for another subject, or not a calibration, is `invalid_arguments`. Pure code in `domain/calibrate.rs`.

## D21 — CSV cell locators (2026-10-24)
*Built in v0.5: [plan](plans/2026-10-24-v0-5-summary-calibration-ci.md), [measured](reports/2026-10-24-v0-5-measured.md).*
`<label>#R<row>C<col>` on a text attachment whose label ends `.csv`, 1-based, the header is row 1, canonical digits only. Rows are records
(RFC 4180, `domain/csv.rs`: quoted fields, doubled quotes, CRLF, line breaks in quotes; no crate), so a quoted newline does not add a row.
The column must exist in that record (ragged rows are allowed). Out of range or non-canonical: `citation_unresolved`. `R` followed by a
letter stays an anchor; `#L<n>` keeps working.

## D22 — the GitHub Action (2026-10-24)
*Built in v0.5: [plan](plans/2026-10-24-v0-5-summary-calibration-ci.md), [measured](reports/2026-10-24-v0-5-measured.md).*
`action.yml` (composite) installs the released binary through `scripts/install.sh` (checksum verified), then `scripts/action-run.sh` takes
`git diff base...HEAD`, runs `should-run`, and with `store` runs `check` and `apply-check --latest`. Outputs `quorum`, `spend`, `effort`,
`answer`, `check`. The job fails on a failing `check` always, and on an answer or a `human` quorum named in `fail-on` (default `human`).
No model, no secret, no network but the download. `apply-check --latest` (new flag) judges every run of the newest run's subject, so CI
needs no run ids. The script needs `jq`. Tested against a local binary in `tests/action.rs`.
- **Amended after the v0.5 review (the maintainer's rulings, 2026-10-24; D13 "gates fail closed"):** `--latest` without `--subject`
  judges every subject that has runs, each on its own, and answers the worst (`human` > `escalate` > `advise` > `apply`), listing each
  subject's answer and reasons; `--subject S` judges S only. The Action passes no subject, and passes `--rule` when `should-run` matched
  exactly one rule. The Action fails closed: an error from `should-run`, `check` or `apply-check`, a missing binary or output that is not
  JSON answers `unavailable`, which fails the job unless `fail-on` is set without it; the default `fail-on` is
  `human,escalate,unavailable`. File names are markdown-escaped in the step summary. The `version` input defaults to the Action's own
  release (`v0.5.0`), not `latest`. A calibration file must name the subject, lens, lens version and a jury run among those given, else
  `calibration_mismatch`. CSV: lowercase and `RC1` locators are refused; a quote in the middle of an unquoted field is literal.

## D23 — the Claude Agent SDK recipe (2026-10-24)
*Built in v0.5: [plan](plans/2026-10-24-v0-5-summary-calibration-ci.md), [measured](reports/2026-10-24-v0-5-measured.md).*
Docs and one example, no runner of our own. `docs/agent-sdk.md`, `examples/agent-sdk/run_jury.py` (Python; the SDK's names are marked "check
the SDK version"). One `query()` session per brief; private folders; a coordinator session ingests through `personajury mcp`. The test only
checks the example compiles and holds no key or environment read.

## D24 — the answer display (2026-10-24, the maintainer, with a display jury of 8 mixed personas)
*Built in v0.5: [plan](plans/2026-10-24-v0-5-summary-calibration-ci.md), [measured](reports/2026-10-24-v0-5-measured.md). Display jury: [report](reports/2026-10-24-display-jury.md).*
`personajury summary RUN... [--html F] [--md F] [--save] [--width N]`, MCP `personajury_summary`. The jury preferred sentences (the non-technical
readers) and numbers (the technical readers) over bars, which got 0 of 8 picks: so **no bars anywhere**. Layout: 2 to 5 plain sentences (the
apply-check answer in words, who won with counts per stratum, the weakest dimension, the change "15 → 6", up to three quoted blockers with
persona and decision), a numbers table "for checking" (medians `P·S·B`, a dash for an empty stratum, top 5 rows), then `Next:` from a fixed
set keyed by the answer. Rows are ordered by the primary stratum's medians; nothing pools strata. `--html`: one offline file, the README
palette, the `graph` SVG inline, no scripts, deterministic. `--md` adds decisions per finding, run ids, brief hashes and the date of the latest
run (no clock: the file is the same on any day). `report_dir` (top-level toml key) names where `--save` writes `<date>-<subject>.md`; unset,
`--save` is refused (`no_report_dir`) and nothing is written. Display is free, the verdict is not: the layout is a recommendation, `apply-check`
is binding only where wired (Action, hook, own gate). Golden: `fixtures/run/summary.txt`, `summary.md`.
- **Amended after the v0.5 review (2026-10-24):** `report_dir`, and the MCP tool's `md`/`html` paths, must be relative, hold no `..`
  and resolve inside the root, else `invalid_report_dir`; a CLI path the person types may be anywhere. Everything printed is
  terminal-safe (ESC sequences, C0 controls but the newline, DEL and C1 controls removed). Quotes keep the juror's words with runs of
  whitespace folded to one space, are cut only between words with `…`, and never inside a word. The first sentence uses plain words
  ("The jury says: escalate. Primary jurors raised 15 blockers, the groups disagree on 2 scores, and 26 findings have no decision
  yet."), and a lesser reason follows in its own sentence ("Even without that, the jury could only advise: no rule was given."). The
  quoted blockers rotate across jurors, primary first, one per juror before anyone's second.

## D25 — the name: PersonaJury, renamed from JuryGraph before the first release (2026-10-24, the maintainer, "let the persona jury decide")
*Report: [the name, picked by a persona jury](reports/2026-10-24-the-name-picked-by-a-persona-jury.md).*
The working name did not say what the tool does (a persona jury scored it 2 of 5 on *understood* in every stratum) and a persona jury
picked PersonaJury 8 of 8 (`docs/reports/2026-10-24-the-name-picked-by-a-persona-jury.md`). Nothing was ever released, so the rename is
complete and carries no shim: the crate, package and binary `personajury`, the config file `personajury.toml`, the store directory
`.personajury/`, the MCP tools `personajury_*`, the environment variables `PERSONAJURY_*`, the manifests and ids, the repository
`massrelay-ai/personajury`, the release assets `personajury-<target>`, and the wordmark re-outlined ("Persona" Bold, "Jury" Regular).
Old stores and old `JURYGRAPH_*` variables are not read. The verb `graph` stays, and the asset is `docs/assets/personajury-graph.svg`
with the title "PersonaJury · <subject>". The brief text carries the product name, so the recorded brief hashes in
`fixtures/run/briefs.json` and the pinned compare-brief hash in `tests/authoring.rs` were regenerated (and the golden `fixtures/run/summary.md`, which lists them); the recorded verdicts, matrix
and picks are byte-identical. The version of every manifest is 0.5.0, and the CHANGELOG is one `[0.5.0] - unreleased` entry.

## D26 — Zlib joins the licence allow list (2026-10-24, maintainer-approved policy fix)
The first CI run on GitHub failed `cargo deny check licenses`: `foldhash` 0.2.0, pulled in by `serde_json` through
`indexmap` and `hashbrown`, is licensed Zlib, which `deny.toml` did not list. Zlib is permissive and OSI approved, and is
compatible with Apache-2.0. Instead of: replacing serde_json's features (D7 needs `preserve_order` and
`arbitrary_precision`). Seen red locally with `cargo deny --all-features check` before the change, green after.

## D27 — the name: JuryPersona, the maintainer's override of the naming jury (2026-10-24, the maintainer)
The maintainer, 2026-10-24: "JuryPersona - override go with this." This overrides D25. The naming jury picked PersonaJury 8 of 8 forced
picks in two runs; in the second run the cards' medians (understood, findable, ownable, memorable, per stratum primary, secondary,
bystander) were JuryPersona 3·3·3 / 3·3·3 / 3·3·3 / 3·3·3 against PersonaJury 4·4·3 / 3·4·4 / 5·5·5 / 4·4·4 (the table is in
[the naming report](reports/2026-10-24-the-name-picked-by-a-persona-jury.md), section "The maintainer's override"). The maintainer may
override it because a name is a human class of decision, a brand: the jury informs it and does not own it, and this one is recorded as
an override, not as a jury result.
What changes: `PersonaJury` becomes `JuryPersona` and `personajury` becomes `jurypersona` everywhere: the crate directory
`crates/jurypersona`, package and binary, `jurypersona.toml`, the store directory `.jurypersona/`, the MCP tools `jurypersona_*`, the
environment variables `JURYPERSONA_*`, the manifests, the plugin and marketplace `jurypersona` (`/plugin install jurypersona@jurypersona`),
`server.json` `io.github.massrelay-ai/jurypersona`, the repository `massrelay-ai/jurypersona`, the release assets `jurypersona-<target>`,
`docs/assets/jurypersona-graph.svg` titled "JuryPersona · <subject>", and the wordmark re-outlined ("Jury" Bold, "Persona" Regular). The
version is 0.5.1 everywhere; the CHANGELOG keeps the 0.5.0 entry and adds `[0.5.1]`.
Cost: a repository rename (GitHub redirects the old URL), release 0.5.1 with new asset names, and a plugin reinstall; v0.5.0 was public but
adopted by no one, so there is no shim and no migration (an old `.personajury/` store is not read). The brief text carries the product
name, so the recorded brief hashes in `fixtures/run/briefs.json`, the pinned compare-brief hash in `tests/authoring.rs` and the golden
`fixtures/run/summary.md` are regenerated; the recorded verdicts, matrix and picks are byte-identical.

## D28 — the final name: PersonaJury (2026-10-24, the maintainer, "PersonaJury (Recommended)")
This supersedes D27 and restores D25's name; D27 stays above as history. A third naming jury ran with 11 mixed people, three models
(Sonnet, Opus, Haiku mixed) and neutral cards, scored 1 to 10: PersonaJury took 8 of 11 picks (primary 3 of 4, secondary 3 of 3,
bystander 2 of 4) against JuryPersona's 3. Medians per stratum (primary / secondary / bystander): say-and-spell 7·7·6 against 6·6·4,
feel 6·5·6 against 5·4·4 (the run is appended to [the naming report](reports/2026-10-24-the-name-picked-by-a-persona-jury.md),
"The third run, and the final name"). The maintainer answered "PersonaJury (Recommended)". Both GitHub repositories had been deleted
and nothing was public under either name, so the rename of D27 is reverted whole: `crates/personajury`, `personajury.toml`,
`.personajury/`, `personajury_*`, `PERSONAJURY_*`, the repository `massrelay-ai/personajury`, the release assets
`personajury-<target>`, `docs/assets/personajury-graph.svg` and the wordmark ("Persona" Bold, "Jury" Regular), with no shim. The
version is 0.5.1 everywhere; the CHANGELOG adds `[0.5.1]`. The recorded brief hashes in `fixtures/run/briefs.json`, the golden
`fixtures/run/summary.md` and the pinned compare-brief hash in `tests/authoring.rs` return to their PersonaJury values.

## D29 — traits as data: the catalog, schema v5, the persona key (2026-10-24, the maintainer, with a design jury of 9)
*Built in v0.6-A. Reference: [`traits.md`](traits.md).*
A trait is a tendency or a standing fact that makes one reader decide differently from another, never a type or a diagnosis; an input,
never a score, never averaged, never a probability. **The catalog** is data: 49 files in `traits/<family>/<trait>.toml` (band 1 to 5 with
anchors at 1, 3 and 5, a category with options, or words) and eight sets in `trait-sets/<name>.toml` (at most 8 traits, a cover of at most
4). They are embedded with `include_str!` (`domain/catalog.rs`, a test keeps the list and the files equal both ways) and installed by
`trait install`, never seeded at store creation: the same hash is skipped, an earlier catalog version is updated (origin `catalog`),
and a trait the user edited (`trait add` of a shipped id with other text, origin `edited`) is never overwritten: it is listed in
`kept_local`, and `--force --reason TEXT` overwrites it, recording the reason in `trait.reason` (amended after the review, I4). Every anchor
is our own words describing a construct and follows "You tend to" (lower case, 40 to 400 characters, no "I ", no "agree"); each file says
`items_copied = false`, because published questionnaire items are copyrighted or academic-use. **Schema v5** is one migration for the three
v0.6 lanes, idempotent (`pragma_table_info` before each `ALTER`): `persona.traits_json` and `persona.serves_json`, `panel.cover_json`,
`run.traits_json` and `run.seats_json`, `subject.audience_json`, and the tables `trait` (with `reason`, the fix round) and `trait_set`; lane C adds `run.options_mode`,
`verdict.phase` and the `deliberation` table, lane B `human_batch.traits_json`. A v4 store becomes v5 on open. **The key** `traits` on a
persona file (at most 8, `deny_unknown_fields` stays) is checked against the store at `persona add`; the persona hash is over the whole
file, so a persona without the key keeps its hash. The design jury of 9 (primary, secondary and bystander seats) voted 7 of 9 to adopt the
key (primary 4 of 4, secondary 1 of 3, bystander 2 of 2) and scored clarity 8·8·8 for the "you tend to" rendering against 6·7·4 for a
number or a label; so the brief says "you tend to", never "you are", and never prints a value. **Culture is not shipped.** The jury voted 6 of
9 against a country-dimension trait (secondary 3 of 3 and bystander 2 of 2 against, primary 3 of 4 for a panel-only use) and scored safety
9·9·8 for personas written from real material against 7·4·4 for the trait; a split in which the approving stratum calls it unsafe means do
not ship. A market's view comes from `persona generate --from` material. **The ceilings** are 8 traits a persona, 8 a set and 4 in a
cover: more traits dilute the brief and make every band pair unreachable (4 traits at 3 bands is 54 pairs, 9 seats cover them; 5 is 90);
a saturation run (does a ninth trait move a verdict?) would change them. **The brief budget, measured** (the fix round, builder-ana's
critical brief on the fixture site, `--traits all`; the design's "eight traits stay under 15%" was wrong): 5 traits add 754 bytes,
+14.7% compact (5120 to 5874) and +9.6% full (7887 to 8641); 8 traits add 1225 bytes, +23.9% compact (5120 to 6345) and +15.5% full
(7887 to 9112). The budget is 15% for a typical 5-trait persona, and it holds. Eight traits stay above it, at compact and at full:
that is a documented cost of a persona that carries the ceiling, not a reason to lower it. `set` mode pays less (D31): builder-ana
under `developer-tool` renders 3 lines (+11.8%), under `b2b-purchase` 5 (+15.5%). `access.need` ships in this lane because a need is a fact the page
must serve, not a disposition, and it is the cheapest slice that finds a page failure. **Refused ids** (`refused_trait_id`, last segment of
the id or the whole id, checked before the store lookup): typologies without predictive validity (`mbti`, `myers_briggs`, `enneagram`, `disc`,
`learning_style`, `generation`, `astrology`, `zodiac`, `blood_type`); protected attributes that are not mechanisms (`age`, `gender`, `sex`,
`ethnicity`, `race`, `religion`, `disability`, `political_affiliation`, `political_lean`, `minority`, `nationality`); attributes of the persona
rather than sensitivities of the juror (`appearance`, `attractiveness`, `beauty`, `fame`, `celebrity`, `outcast`, `outsider`).
**A whitelist, not a blacklist** (the maintainer's ruling after the v0.6-A review, I2): a persona trait id is accepted only if the
store's catalog holds it, shipped or added with `trait add` (generated from material or not); every trait added that way, and every
persona trait id that is not shipped, passes a protected-attribute screen: refused (`refused_trait_id`) when its family is `demo`,
`culture` or `protected` (as `project.<f>` too, so "no country-dimension trait" holds for project traits), or when a word of its id
or title starts with a stem of `PROTECTED_STEMS` (`domain/traits.rs`, the one table, documented in `traits.md` and tested stem by
stem; **amended by D39: these are labelled, not refused**): age, gender, sex, sexual, orientation, race, racial, ethnic, religion, religio, faith, caste, nationality, national,
citizenship, country, pregnan, marital, disab (not under `access.*`, the assistive needs), politic, union, health, genetic, skin,
color, colour, tribe, tribal. The shipped catalog's own id and title are reviewed words and are not screened again
(`risk.health_safety`, "in good faith"). A free-text value (`situation.job`, a words trait) is at most 80 characters, holds no
digit, does not start "You are" or "I am", and passes the same stems, else `trait_text_refused` (I3). The catalog's
social family has eight traits (the lane brief said seven; the report's table is the authority and the total is 49). Two columns beyond the
brief's list serve `--seats auto`: `persona.serves_json` and `subject.audience_json`, with `run.seats_json`.

## D30 — cover, quotas, balance and coverage (2026-10-24, the maintainer)
*Built in v0.6-A.*
A panel term may carry a filter, `COUNT TAG[=STAKE][@MODEL][FILTER]` with `FILTER = [trait OP value, ...]`, `OP` one of `=`, `!=`, `>=`, `<=`;
`=` on a multi-option category means "contains"; a persona without the trait never matches; the filter narrows the pool before the seeded
draw and leaves the seeds as they were (a filter that excludes nobody seats the same people). `panel add --cover a,b,c` (at most 4) records
the traits the draw covers. The **cover-aware draw** (`domain/cover.rs`): for each term in query order the pool is the matching personas
in the seeded order (the tie-breaker); each seat is the unchosen persona that adds, lexicographically, the most new one-way band cells for
the term's stratum and then the most new panel-wide pairs of cover bands (correlated pairs excluded); the state carries across terms. With
an empty cover the seats are the first `count` of the seeded order: the v0.2 draw, byte for byte. The price: under a cover a persona added
to one term may move another term's seats, so coverage trades the per-term isolation D10 gave; the pool is the same between rounds, so
`delta` is unaffected. **Coverage** is counts of seats and nothing else: one-way seats per band and stratum (with `unset`), pairs
panel-wide (`covered` of `total`, `uncovered`, `excluded_correlated`), `balance` (bands no seat of a stratum holds), `collinear` (two cover
traits that sort the seated personas identically) and `confounded` (a cover trait whose band is the same for two or more seated personas of
a tag while a seated persona without the tag has another band; a persona without the trait does not count as different). Bands: low 1 to 2,
mid 3, high 4 to 5. Strata are never pooled; the pair count is panel-wide by design, and says so (`scope: "panel"`).

## D31 — traits in the brief, slices in the results (2026-10-24, the maintainer)
*Built in v0.6-A.*
`brief --traits <set>|all|off` (default: the rule's `trait_set`, else `off`; a different value under a rule is an override and needs
`--reason`, field `traits`). With `off` no byte of any brief changes; that is why the seven fixture personas can carry traits and the fixture
replay stays byte-identical: it adds the personas (after `trait install`) and briefs without a rule or `--traits`, and nothing in the replay
pins a persona hash. The block "How this reader tends to decide (told)" sits between the persona list and the stake: trait ids sorted, band
1, 3, 5 as "You tend to: <anchor>", band 2 and 4 as "You tend to lean this way: <anchor of 1 or 5>", a category's option sentence, words as
"Your work: <job>" (a words value, screened at `persona add`; never "You are", the fix round I3), and the derived route last when involvement and need for cognition are both rendered (central when both are 4 or more).
Value, id and title are never printed; the block has no digit and no "You are"; it closes with "Told, not seen: a line leaning on
one is `told`; a line about the subject still cites it." (shortened in the fix round: it is paid once per brief). `set` mode renders
the set's cover traits the persona has, and its other set traits unless they sit at 3, the middle of a band and the default reading
(the fix round, for the 15% budget of D29), and reports `trait_set_incomplete` (a warning; `apply-check` adds the informational reason). The run records `{mode, set, versions, draw}` (the
draw is `cover` or `seeded`), plus `incomplete`. **Slices** are counts: `matrix --slice TRAIT` adds `slices` beside `strata`, per stratum the
groups by band with their cells (lower-middle median, min, max, blockers), `thin` below two personas, `unset`, `separates` (cells where two
groups' `anchor_met` ranges do not overlap, the D13 rule applied to groups within a stratum) and collinearity warnings; no key is a mean, a
share, a rate or a probability. `summary --slice` adds one fixed-template sentence per stratum, ending "In this jury, not a claim about
people." The golden summaries are produced without `--slice` and do not change. **`--seats auto`** (adopter feedback: fixed panels of 8
to 14 seats are the main token waste): personas carry `serves`, subjects an `--audience`; the seats are every persona that serves the
audience (a seeded draw, or the trait set's cover) plus exactly one control that does not (a seeded draw), recorded on the
run as `seats [{persona, why}]`; refusals `audience_unknown` and `no_matching_personas`. Amended after the review: the control counts
**inside** the effort's cap (low 3 = 2 serving + 1 control, medium 7, high 12, max 12 as before), so a refusal never fires on a seat
the engine added (I1); under a rule, `--seats auto` replaces the rule's panel without an override, because the seats follow the
subject's audience the same way every time; what else the caller changes (lenses, effort, traits, spend) is an override as before
(I5). A rule may therefore leave its panel to `--seats auto`; the guard is `apply-check`'s `thin_primary` (D13): a primary stratum
of fewer than two judged seats advises.

## D32 — options by subject kind (2026-10-24, the maintainer, adopter feedback)
*Built in v0.6-C. Reference: [`commands.md`](commands.md), Options by kind.*
What a juror may score `about` and `pick` was the subject's attachment labels plus `cmp:<name>`, so a juror briefed on a page "picked"
`pricing.png`. `brief --options subject|attachments` (MCP `options`) fixes it: by default `design` and `logo` give `attachments` (each
board is a candidate) and every other kind gives `subject` (the options are the subject's name and each `cmp:<name>`). Ingest normalises
a line `about: "<subject name>"` to `about: null` before validation, so cells stay keyed as they always were, and a verdict with both
forms on one dimension is `duplicate_dimension`; a `pick.choice` of the subject name is valid. The mode is stored on the run
(`run.options_mode`, a column added on open like the other v5 columns); a run with none (made before v0.6) validates as `attachments`,
so old stores keep working. An unknown word is `invalid_options_mode`. **The fixture and the pinned hash stay byte-identical because they
pass `--options attachments`**: the recorded verdicts score `about: pricing.png`, and the option list text is identical in that mode, so
`fixtures/run/setup.sh`, `tests/fixture_replay.rs` (and the other helpers that rebuild the fixture jury) and
`compare_brief_unchanged_by_about_scope` carry that flag and a comment that they were recorded before D32; `COMPARE_BRIEF_SHA256` is not
re-recorded. What would change it: a regenerated fixture jury whose verdicts score `about: lantern-site`. The `a-technical-decision`
playbook, whose options are files, now says `--options attachments` in its command; `pick-a-name-or-logo` is a `design` subject and
is unchanged. A decision subject (D37) takes the third word, `questions`, and refuses `--options`.
**Amended after the v0.6-C review (M1).** With no `--options`, a subject briefed before keeps the mode of its latest run (a run with
no recorded mode, made before v0.6, counts as `attachments`), so a page briefed under v0.5 and again under v0.6 keeps option cells that
`delta` can compare; a subject never briefed takes its kind's default, and an explicit `--options` always wins.

## D33 — deliberation: one advise-only round (2026-10-24, the maintainer)
*Built in v0.6-C. Reference: [`commands.md`](commands.md), Deliberation.*
A second round in which each juror reads what the others saw and may change its verdict. It breaks independence by design, so it only
advises: it exists to measure persuasion, not to reach a verdict. `personajury deliberate RUN` (MCP `personajury_deliberate`) is
refused unless every briefed seat has an accepted initial verdict (`incomplete_jury_for_deliberation`) and runs once (`already_deliberated`;
one round in v0.6). It writes one deterministic file per seat: the persona, stake and (when the run renders traits) trait blocks as in the
brief, the juror's own first verdict, and the others' lines as "Juror 1 (stratum)", "Juror 2 (stratum)" in persona-id order with no
name, no model, no citation, so a juror cannot weigh a colleague by who it is. The answer is a verdict with `"phase": "deliberated"`,
ingested with `verdict --deliberated` (`deliberated_without_phase`, `phase_without_flag`, `no_deliberation`,
`deliberated_without_initial`); the evidence refusals are those of any verdict, and a score changed because of another juror is `told`
unless the juror cites what it saw itself. It is a new batch with `verdict.phase = 'deliberated'` (a column; a `deliberation` table
holds the files). The latest **initial** batch is what every read uses by default (`matrix`, `pick`, `summary`, `delta`, `graph`,
`calibrate`, `apply-check`); `--phase deliberated` reads the other batch and `matrix --phase both` answers both under `phases`.
`check --history` lists each batch with its phase. `apply-check` answers at most `advise` for a run with a deliberated verdict (reason
`deliberated`); `escalate` and `human` take precedence. Rejected: letting a deliberated batch replace the first (it would turn a
measure of persuasion into the verdict) and several rounds (a second round measures the first round's anchoring).
**Amended after the v0.6-C review (the maintainer's rulings, 2026-10-24).** The independent batch is frozen the moment `deliberate`
starts: an initial verdict for the run after that, with or without `"phase": "initial"`, is refused (`initial_closed`), because a
juror who may have read the others could otherwise answer again as the jury's first word and reach `apply`. A person's `--by-human`
verdict (D19) is no juror's first word and is still taken. `apply-check` advises (`deliberated`) as soon as the round exists, with or
without a deliberated verdict, and reads only the frozen batch. **Schema v6**: `verdict.phase` changes what "the latest batch" means,
so the store's version moved from 5 to 6 (the v0.6 columns are added on open, idempotently); a v0.5 binary refuses a v6 store by its
version rather than reading a deliberated batch as the latest, and a v5 store upgrades in place.

## D34 — the honest metrics: counts per stratum, never pooled, never a mean (2026-10-24, the maintainer)
*Built in v0.6-C. Source: the metrics note, 2026-10-24. Reference: [`commands.md`](commands.md), The counted metrics.*
Adopted, as counts inside a stratum: **agreement** (per cell `within_one`, the scores within one point of the lower-middle median;
per stratum `agreement {cells, cells_all_within_one}`); **persuasion shift** (once a deliberation was answered: per cell and persona
`{before, after, step}`, and `moved_up`, `moved_down`, `held`, `toward_first_median`, `away_from_first_median`, direction by the score,
`step` in anchors at or below the score, the first median being the initial batch's); **evidence totals** (`seen`, `told`, `recalled`
summed over a stratum's cells); **panel match** (per panel term: `asked`, `seated`, `answered`, from the terms the run was seated from,
now recorded on the run as `panel_terms_json`); **cost** (each juror's own reported tokens listed, their sum, how many jurors reported,
the estimate; nothing divided; the summary ends with a `cost:` line, a dash where nothing was reported). No key anywhere is named
`mean`, `average`, `share`, `rate`, `percent` or `probability` (a test walks the JSON). `summary` writes them as fixed sentences after
the blockers and before the table, a `panel:` line under the table and the `cost:` line last. **The goldens moved, additively**:
`fixtures/run/summary.txt` and `summary.md` gained lines only (`diff <(git show HEAD:fixtures/run/summary.txt) fixtures/run/summary.txt |
grep -c '^<'` is 0), `matrix.json` gained `within_one`, `agreement` and a stratum `evidence` and is otherwise the previous file
(`jq -S 'del(.. | .within_one?, .agreement?, .evidence?)'` of the new file equals the previous with `.evidence?` deleted, the per-cell
`evidence` existing since v0.5); `briefs.json`, `picks.json` and `matrix.md` did not change (a `--table` carries no new footer, so the
recorded table and its `ends_with` legend test stand; the footers live in the summary). The summary's line budget went from about thirty
to thirty-four for the four new sentences and lines. **Rejected**: win probability, average damages, pooled indexes, emotion signals
and simulated-time curves. A guessed number has no evidence behind it, and a pooled one breaks the strata law.
**Amended after the v0.6-C review (M2, M4).** `cost.per_juror` is one entry per juror of a run (`batches` lists its batches, the tokens
are their sum), so a juror with two batches is listed once. The agreement sentence capitalises only its first stratum ("...; secondary
jurors agree ..."); the goldens `summary.txt` and `summary.md` changed in that line only, a line this lane added, so against main they
still only gain lines.

## D35 — trait generate, and the reverse generators: traits from material, from prose, from verdicts (2026-10-24, the maintainer)
*Built in v0.6-B. The maintainer's words: "reverse traits generator based on juror". Reference: [`traits.md`](traits.md).*
Three ways to a trait that are not "pick one from the catalog", none of which calls a model and none of which writes a trait back.
- **`trait generate --from PATH [--count N]`.** As `lens generate` (D14): hash the material, write `.personajury/generate/trait-<material
  hash>.md` and record the material. `--count` is 3 by default, 1 to 8 (`invalid_arguments` outside). The brief has two steps for the
  agent: list, in the material's own words, the ways its readers are known to differ; then draft N trait files in the exact TOML shape of
  D29 with `family = "project.<slug>"` (the slug is the material's file or folder name in `[a-z0-9_]`), `evidence = "project"`,
  `source = "material:<hash>"`, `items_copied = false`, anchors or options as the predicate after "You tend to". Its two examples (a
  band trait and a category trait) are valid files, and a test adds them. The brief states the refusals in one line (no protected
  attribute, no typology, no country dimension) and prints the list. **The screen is never bypassed.** What the agent writes enters through
  `trait add --generated-from <hash>`, the path D29 already screens (family, id, title; `refused_trait_id`); `trait generate` runs the same
  `screen_id` on the slug and refuses a material named `gender-survey.md` instead of writing a brief whose every id would be refused.
  A project trait shows `(P)` in `trait list` (`badge`) and in `matrix --slice` headers, as a credential shows `(C)`.
- **`trait suggest --persona ID [--set NAME]`.** A brief, `.personajury/generate/trait-suggest-<persona id>-<hash8>.md`; `hash8` is the
  first eight hex digits of sha256(the persona's hash, then the sorted `id@version` of the candidates, one per line). It lays the
  persona's seven prose fields beside the candidates' anchors (the set's traits, `trait_set_unknown` if missing, else every trait in the
  store, in id order) and asks for one JSON object `{persona, traits, evidence}` with a verbatim quote per trait. No model; the answer
  is the path, hash and bytes. **`trait suggest --check FILE [--persona ID]`** validates the object: ids and values as `persona add`
  would (`unknown_trait`, `trait_value_off_scale`, `trait_option_unknown`, `traits_too_many`, `refused_trait_id`); every trait has a quote
  (`suggestion_without_quote`); the quote, whitespace folded, is a substring of the folded prose (`quote_not_in_prose`, case-sensitive; amended in the fix round, D39: at least 3 words and 12
  characters, `quote_too_short`, and inside one field, `quote_not_found`).
  It answers `data.persona_file` (the stored persona with `traits` replaced where suggested, others kept) and `data.evidence`, and
  **writes nothing**: the caller saves the file and runs `persona add`, a new version. Prose first, traits as its summary, each with its
  line of evidence. The MCP tool `personajury_trait_suggest_check` takes the object inline.
- **`trait observed --persona ID [--persona ID ...] [--runs A,B]`.** Counts, from the persona's accepted initial verdicts: `severity`
  (its score in each cell below, at or above its stratum's median, the lower middle as `matrix` computes it, keyed by lens and dimension),
  `blockers` and `bottom` (its count against the stratum's median count: more, same, fewer), `punished` (the cell with the most
  `below`, ties by id order, null when none), `evidence` (seen, told, recalled), `pick` (subject, comparator or attachment; with or
  against the stratum's majority, a tie counting as neither). **A comparison needs a stratum of at least three jurors** (the persona and two
  others): with fewer, severity, blockers, bottom and the majority add nothing (the evidence counts and the class of a pick are the persona's own); this refines the brief, which asked it for severity only,
  because a median of one or two is the persona itself. Then `traits/observe-map.toml`, shipped data embedded beside the catalog
  (`domain/catalog.rs` `OBSERVE_MAP`; it needs no install, `trait install` has nothing to load for it), 18 rows, and a project's own rows
  in `.personajury/observe-map.toml`: **a project row adds a rule and overrides none** (a repeated id is `invalid_arguments`). Per declared
  trait: the rules whose pole fits (a band of 4 or 5 the `high` rules, 1 or 2 the `low` rules, 3 none; a category the rules whose pole is
  the option); a rule is `consistent` when its direction's count exceeds the opposite one's, `contradicted` when the opposite exceeds it,
  `no_signal` on a tie; the trait's word is `contradicted` if a rule contradicts and none is consistent, `consistent` the other way,
  `mixed` if both, `no_signal` otherwise. **It is a bearing, not a measurement; a contradicted trait is a question for the author, not a
  verdict on the persona.** Several personas answer `{personas: [...]}`. The MCP tool `personajury_trait_observed` reads only. The tools
  are twelve now (the brief counted nine to eleven; lane C had made the base ten).

## D36 — traits proven: human traits, calibrate --slice, trait report, persona lint, label_echo, graph --slice (2026-10-24, the maintainer)
*Built in v0.6-B. Reference: [`traits.md`](traits.md).*
- **A person's own traits.** `verdict --by-human "<name, credential>" --traits T.json`: `T.json` is a `traits` map in the persona's
  shape, validated with the persona's refusals, stored in `human_batch.traits_json` (the column joined the v6 migration; no version
  bump, the column is read only by this lane) and shown by `matrix` under `human[].traits`. `--traits` without `--by-human` is
  `invalid_arguments`. The MCP has no `by_human` (D19), so no `traits` either. A person's traits are what they say of themselves;
  the engine administers no questionnaire and infers nothing from a score.
- **`calibrate JURY HUMAN --slice TRAIT [--out F]`.** In each stratum the jury's personas (with the traits they were briefed with) and
  the people (their declared traits) are grouped by band or option; a side without the trait is `unset` and left out. Per group, the D20
  rank comparison of its cells on both sides (`within_one_rank`). Then `separates_in_jury {cells, total}` and `separates_in_people`
  (two groups' `anchor_met` ranges on a cell not overlapping, the D13 rule), and the stratum's `word`: `thin` if any group held on
  either side has fewer than two (or fewer than two groups are held at all: nothing can separate); else `predicts` if both sides
  separate on at least one common cell; `missed` if the people separate and the jury does not; `inert` if neither does; else `played`
  (the jury separates and the people do not, or on other cells than they do). Written under `slices` in the calibration file;
  `calibrated` is untouched and `apply-check --calibration` reads nothing new. Nothing crosses a stratum.
- **`trait report --calibration F [F ...]`.** Per trait and stratum, the count of each word across the files and the number of files
  that sliced it; a file that is not a calibration is `not_a_calibration {path}`. No total across strata, no share.
- **`persona lint [ID ...]`.** Warnings, exit 0 always: `near_copy {a, b}` (the same non-empty trait map and a shared tag, or the same
  tags) and `confounded {trait, tag, band}` (two or more personas with the tag in the band, one or more without it in another).
  With ids, only the warnings that name one of them. `trait_without_prose` is not built: the engine cannot read prose for meaning.
- **`label_echo`.** A verdict whose note or blocker repeats a trait the persona's brief rendered (the fix round: not every trait it
  carries; a brief with no trait block gives none) gets `warnings: [{code: label_echo, line, trait}]`
  and is still accepted; the trait's title (any case) or the id's last segment with `_` as a space matches, each word by its first seven
  letters and in order, so "a marketing skeptic" echoes `marketing skepticism`. A one-word segment (`domain`) echoes by chance; it is a
  warning. `matrix` counts the echoes per stratum under `warnings` where there is one.
- **`graph --svg --slice TRAIT`.** Persona nodes take the lightest, middle or darkest step of the blue scale by band (a category's
  options cycle the three in option order), grey for unset; a legend line under the title names the trait and gives the counts per
  stratum (`P 1·0·2`, never one total). Layout and medians are unchanged; without `--slice` the bytes are v0.5's.

## D37 — decision juries (2026-10-24, the maintainer, adopter feedback from a build terminal)
*Built in v0.6-C. Numbered D37 because lane B holds D35 and D36. Reference: [`commands.md`](commands.md), Decision juries.*
The adopter replaced owner questions with a small jury: a list of questions plus the owner's point of view written in files, three
personas, and "still needs the human" for the questions that are the owner's. The walk-centred design did not cover it. A subject of
kind `decision` is a `questions.md` (`## Q<n> <question>`, `- option` lines; `invalid_questions`) plus the point-of-view files, the shipped
lens `decide` (one dimension, `confidence`, `asks_pick`) is the fifth in every store, lines are `about: "Q<n>"` and `picks` holds one
`{about, choice, reason}` per question from that question's own options (`question_pick_invalid`; the `pick` table gained `about`).
`pick` and `summary` show each question's picks per stratum. **The escalation rule is code, not judgement**: a question needs a person when its text or
options fall in one of five classes (public price, public words, legal, spending, irreversible or outward; whole-word lists in
`domain/decision.rs`), when the point-of-view files fall under a human rule of `personajury.toml`, or when the jurors of one stratum split
on it and it is about money or trust; the summary lists them and `apply-check` answers `human` (reason `human_class`, `via` `question`,
`picks` or `sources`). The lists are a first cut and miss words; a miss is a question the jury answers and a person still reads, and a
false hit costs a person a minute, so the lists lean wide.
**Amended after the v0.6-C review (the maintainer's rulings, 2026-10-24).** D13 governs: apply only when proven.
- **A strict grammar.** A question is exactly `## Q<n> [class: <class>] <text>`, `n` canonical (1, 2, 3 ..., no leading zero); options
  are `- <option>` lines; blank lines are allowed. Any other non-blank line, any heading of another level that mentions `Q<digit>`, a
  duplicate or non-canonical id, or a question with fewer than two options is `invalid_questions`, naming the line. The review found a
  `### Q2` price question silently merged into Q1; nothing is dropped now. (A `# title` line is refused too: the grammar lists only
  questions, options and blank lines.)
- **A declared class.** Every question declares `taste`, `product`, `copy`, `ops` or a human class (`money`, `permission`, `deploy`,
  `secret`, `licence`, `public-claim`, `brand`, `legal`, `spending`, `irreversible`); none is `question_class_missing`. A declared human
  class answers `human`. The keyword lists became a documented stem table (`domain/decision.rs` `STEMS`, listed in `commands.md`),
  matched by word start so plurals count: a jury class whose text or options hit a stem answers `human`, reason `class_mismatch`, with
  the stem named. Beyond the ruled stems the table keeps the earlier list's words (dollar, purchase, salary, payroll, revenue, hire,
  vendor, compliance, trademark, patent, sunset, deprecate, public, credential) and adds `turn off` and `shut down`, which the review's
  "turn off the old API for all users" needed; the headline/tagline/slogan words were dropped so a `copy` question can stay with a
  jury. The three classes a decision adds (`legal`, `spending`, `irreversible`) are question classes only; the `personajury.toml`
  human-class list (D13) is unchanged.
- **A split is between strata.** A question is split when the strata's top picks differ (a stratum tied at the top counts). A split on
  money, spending, trust, legal or public-claim (declared, or matched: money and spending by the money stems, legal by the legal stems,
  public-claim by the outward stems, trust by trust, secur, safe, confiden, audit, consent or a secret-or-permission stem) answers
  `human`, reason `split_on_sensitive`; a split on any other question answers `advise`, reason `split_on_question`.
- **Sources are informational.** The point-of-view files are scanned with the same table; hits are listed in `summary` ("Sources
  mention: ...") and as the `apply-check` reason `sources_mention` (`info`). They change no answer: what decides is the question's class
  and text and the project's path rules (a source path under a human rule still answers `human`).

## D38 — the starter config and cost by default (2026-10-24, the maintainer, adopter feedback)
*Built in v0.6-C.*
`personajury init` and `setup --apply` write a commented `personajury.toml` when the project has none, never over one: the code paths
(`crates/**`, `migrations/**`, `tools/**`, `.github/**`, `router/rules/**`, `scripts/**`, `tests/**`) at quorum `none` ("no jury"), a
`surface` class for `screens/**`, `pages/**` and `docs/**` ("a jury, surface class": the adopter made surfaces the critical class), and
the human classes as comments. It is a starting point, said so in its first line; setup reports it under `project_config`. **Cost by
default**: `summary` always ends with a cost line (estimate and reported tokens per run and per seat; reported only when verdicts carry
usage), the adopter tracking a weekly budget. No new dependency.
**Amended after the v0.6-C review (the maintainer's ruling, 2026-10-24).** The starter ships its rules active, not commented out:
human rules for pricing, prices, billing and checkout (`money`), legal, terms and privacy (`legal`), auth (`permission`) and secrets
and `.env*` paths (`secret`); `high` for the other surface paths (`screens/**`, `pages/**`, `docs/**`, `site/**`); `none` for code,
migrations, tools and CI (`crates/**`, `src/**`, `migrations/**`, `tools/**`, `.github/**`, `router/rules/**`, `scripts/**`,
`tests/**`). The highest quorum wins, so `crates/billing/src/price.rs` and `site/pricing.html` are `human`. The review had found that
adopting the old starter made `apply` reachable on prices and legal pages (`low`), where no config at all answered `advise`.

## D39 — simulated demographics are allowed, labelled and recorded (2026-10-24, the maintainer)
*Built in the v0.6-B fix round. Supersedes the refusal half of the D29 protected-attribute screen. Reference: [`traits.md`](traits.md).*
The maintainer's words: "We are keeping everything as sims, not deciding or angling … everything is supposedly technically produced as
best practice". The orchestrator's ruling: **allow, label, record.** A persona is a simulation, a composite; a jury of simulations
that names an age, a gender or a religion is a study design, and refusing the word only pushed it into prose the engine cannot see.
- **The detection is kept and widened; its outcome changes.** One table (`domain/traits.rs` `SENSITIVE_STEMS`, 64 entries, a test per
  entry) reads every text of a trait (id, family, title, caveats, source, each anchor, each option's id, title and sentence), every
  free-text value, and a persona's prose (role, context, daily tasks, fears, never reads, decides by, credential, stake rules, tags).
  Texts and stems are normalised first, without a crate: the NFKC compatibility forms the screen needs (fullwidth ASCII, the
  mathematical letters, the Latin ligatures), the Latin look-alikes of Cyrillic and Greek letters, lower case, invisible characters
  (zero-width, soft hyphen, direction marks) and combining marks removed, marked Latin letters folded (`género` is `genero`). Each
  text is read twice: word by word, and with the separators `_ - .` and space removed between words (`gen_der`, `non-binary`,
  `senior citizen`), where a joined stem must start where a word starts and run into the next, a whole-word stem must end where a
  word ends, and a short prose word (`a`, `be`, `the` …) never starts a join ("a general rule" is not `age`, "be hindered" is not
  `behinder`). Each entry matches within a word (`bisexual`, `unhealthy`), at its start (`colorful`, not `international`) or as the
  whole word for a short or ambiguous stem (`trans`, `alter`, `sikh`, `raza`, `edad`, `male`: not `transform`, `alternative`,
  `maleficent`). The entries: the D29 stems; trans, transgender, nonbinary, elderly, senior citizen, pensioner, muslim, christian,
  jewish, hindu, buddhist, sikh, islam, catholic, atheis, etni/étni/ethnie, raza, rasse, religi, nacional, nationalit, género/genero,
  geschlecht, edad, alter, schwanger, behinder, handicap; and the review's female, male, woman, women, immigra.
- **A label, never a refusal.** Each hit names one of twelve categories: age, gender, ethnicity, religion, nationality, health,
  disability, sexuality, politics, marital, pregnancy, genetics (`SENSITIVE`, the order an answer lists them). `trait add` (with or
  without `--generated-from`), `trait generate` (the material's name and slug), `persona add`, `trait suggest --check` and
  `verdict --by-human --traits` accept, and answer `sensitive: [...]`. `refused_trait_id` keeps only what means something else: the
  typologies and the persona's own beauty, fame or outsider status (the D29 list without its protected entries), and the `culture`
  family ("a country dimension, not a mechanism"); the `demo` and `protected` families are a simulation's own and are labelled.
  `trait_text_refused` keeps its length, digit and "You are" rules. The `access` exemption is gone: `disab` and `health` label where
  they hit.
- **The catalog is reviewed by hand.** A text the catalog ships unchanged carries the catalog's own label (`SHIPPED_LABELS`): only
  `risk.health_safety` names one (health); "in good faith", "colour vision" and "health" in a caveat are words by chance. An edited
  text is read again.
- **Recorded on the run.** `brief` records `uses_sensitive` (the categories its seated personas name) in `run.uses_sensitive_json`, a
  column of the v6 migration (idempotent `ALTER`, no version bump), and answers it. `matrix` answers `uses_sensitive` across its runs;
  `summary` (text, `--md`, `--html`) adds one line when there is any: "simulated demographics: age, gender — personas are
  simulations, not real-world data". No advisory cap and no human class follow from it; the decision rules of D13 and D37 are
  unchanged.
- **The cost, measured on the fixture.** newcomer-sofia's role ("a final-year student in public health") labels her `health`, so the
  fixture runs record `health` and `fixtures/run/summary.txt` and `summary.md` gained that one line (regenerated); the briefs, the
  matrix, the table and the picks are byte-identical. The table errs wide: a label is a line in a summary, not a refusal.
- **Unchanged: composites only.** A persona is never a real, named person (`PERSONAS.md`); a label records what a simulation names,
  it makes no claim about anyone.
- **The other fix-round rulings stand.** A `suggest --check` quote is at least 3 words and 12 characters and stands verbatim inside one
  prose field (`quote_too_short`, `quote_not_found`, which replaces `quote_not_in_prose`); `label_echo` reads only the traits the brief
  rendered (none when the run rendered none); the three-juror rule and the initial-batch filter of `trait observed` are held by tests.
