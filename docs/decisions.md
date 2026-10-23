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
    (D17); `independence_sequential` (D18); `stale_run` (the subject moved since the run, or a later run of the same subject and
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
