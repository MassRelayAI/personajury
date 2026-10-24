<!-- shape follows comparegraph docs/commands.md -->
# Commands

Every verb prints one JSON envelope (`ok, verb, schema, data, evidence, cost, error`) and exits 0 on success, 1 on error.
Global flags: `--root DIR` (default `.`, the store is `DIR/.personajury/store.db`) and `--pretty`. `personajury --version` prints
the version.

| verb | arguments and flags | what it does |
|---|---|---|
| `persona add` | `--file F` `[--generated-from HASH]` | add one persona from the fixed JSON |
| `persona generate` | `--from PATH` `[--count N]` (default 7) | write the brief that asks your agent to write personas from material |
| `persona list` | | list personas |
| `panel add` | `NAME` `--query Q` `[--allow-large]` | save a query over persona tags with a stake per term, e.g. `"3 builder=primary, 1 newcomer=primary, 2 approver=secondary, 1 others=bystander"`; more than 12 seats is refused (`panel_too_large`) unless `--allow-large`, and more than 1000 always |
| `panel resolve` | `NAME` `[--subject S]` | the personas the query resolves to now; with `--subject` the seats a draw gives that subject (`draw.seed`), without it the first by id (`draw: "none (no subject)"`) |
| `lens add` | `--file F` | add a lens (TOML) |
| `lens generate` | `--from PATH` `[--dimensions N]` (default 4, at most 12) | write the brief that asks your agent to draft a lens from a standard, a handbook or examples (see below) |
| `lens list` | | list lenses; four ship (a 1 to 10 `site` lens is fixture data: `lens add --file fixtures/lenses/site.toml`) |
| `subject add` | `NAME` `--kind K` `--attach FILE...` | a subject as attached files; kind is `page`, `diff`, `design`, `logo`, `document`, `menu` or `prices` |
| `comparator add` | same as `subject add` | a comparator, stored the same way and flagged |
| `brief` | `--subject S` `--model M` and `--panel P` or `--query Q` or `--rule R`; `[--lens L,...]` `[--stake p=stake,...]` `[--spend N]` `[--comparator C]...` `[--detail compact\|full]` `[--draw HEX]` `[--wave N]` `[--effort low\|medium\|high\|max]` `[--independence independent\|sequential]` `[--allow-large` `--reason W` `--by WHO]` | one deterministic, hashed brief per persona, one run per lens; `--lens` and `--comparator` take a comma list or repeat; the answer carries `waves`, `estimate {briefs, tokens}` and `draw.seed` |
| `verdict` | `--run N` and `--file F` or `--json J` `[--by-human "NAME, CREDENTIAL"]` | ingest one persona's verdict, or with `--by-human` a person's (see Experts); refused without evidence; may carry `usage {input_tokens, output_tokens}` (`invalid_usage` when malformed) |
| `matrix` | `RUN...` `[--history]` `[--table]` `[--detail compact\|full]` | scores per stratum, and `usage {reported, estimate}`; `--across-strata` exists only so the refusal is explicit |
| `graph` | `RUN...` `--svg` `[--out F]` | the jury as one static SVG (see below); several runs of one subject draw each finding's life; answers `data.svg`, or writes `--out` |
| `pick` | `RUN` | the forced choices of a run, per stratum, with reasons |
| `summary` | `RUN...` `[--md F]` `[--html F]` `[--save]` `[--width N]` | the answer for a person: plain sentences, a numbers table "for checking" (medians primary·secondary·bystander), then Next; no pooled number; `--save` writes into `report_dir` |
| `calibrate` | `JURY_RUN HUMAN_RUN` `[--out F]` | the jury's rank order per stratum beside a human run's (`verdict --by-human` batches); `calibrated` when every primary cell is within one rank |
| `delta` | `A B` | what moved between two runs on the same subject and lens |
| `apply-check` | `RUN...` or `--latest [--subject S]`, `[--rule CLASS]` `[--calibration F]` | the verdict to obey last: `apply`, `escalate`, `human` or `advise`, with the reasons as data (see below); applies nothing itself |
| `check` | `[--history]` | stale runs (source changed or a newer subject version), `fixed_unverified`, and findings with no decision from any batch or fixed and raised again (`fix_not_landed`); exits 1 with `check_failed` listing `unresolved`; `--history` adds `decisions` and `lives`, each finding's life across the runs (`raised`, `raised_again`, `fixed`, `unjudged`, `absent`) |
| `decide` | `FINDING OUTCOME` `[--why W]` `[--row R]` `--by WHO` or `--by-human NAME` | outcome is `fixed`, `dismissed` (with `--why`) or `row` (with `--row`); a finding a human batch raised needs `--by-human` (`human_decision_required`) |
| `should-run` | `[--diff F]` `[--paths a,b]` `[--subject S]` `[--config F]` | classify a change: `none`, `low`, `high`, `human`, and the spend; `--subject` classifies the subject's recorded source paths (a page added by hand has no diff); `warnings` names a rule's panel or lens the store lacks |
| `setup` | `[--client C]` `[--scope S]` `[--apply]` `[--force]` `[--bin B]` `[--no-rules]` `[--no-hook]` | register the MCP server; a plan unless `--apply` |
| `doctor` | `[--bin B]` `[--brief]` | the store, counts and registrations; `--brief` is two plain lines for a hook |
| `mcp` | | serve MCP over stdio |

A panel term may carry the stake its seats get (`3 builder=primary`); `brief` uses it. `--stake` takes
`persona=primary|secondary|bystander`, repeated or comma-separated: it wins over the term, and is required only for a
persona whose term has none (`persona_without_stake`). A term that resolves to no persona is refused (`panel_term_empty`).

### The seeded draw
When a term's pool of personas is bigger than its seats, `brief` draws the seats: the pool sorted by id, then a
Fisher-Yates shuffle seeded (SplitMix64) from `sha256("draw\t" + subject name + "\t" + panel name)`. The seed comes from the
subject's name, so round 2 of the same subject draws the same bystanders and `delta` can land; another subject draws
others. A term whose pool equals its seats is not shuffled. `--draw HEX` (16 hex digits) sets the seed; the run records
the one used (`run.draw_seed`).

### `brief --rule`
`--rule CLASS` reads `<root>/personajury.toml` and takes the rule's panel, lenses, spend (`0` = the whole panel) and detail
as defaults, one run per lens (the answer has `runs`). A rule with quorum `low` or `high` must name a `panel` and at least
one lens (`rule_without_panel`, `invalid_config`); a rule naming a panel or lens the store lacks is `rule_panel_unknown` /
`rule_lens_unknown`; a rule of a human class, or quorum `human`, is refused (`human_class_under_persona_quorum`), and
quorum `none` runs no jury. The caller may override with `--panel`, `--query "<terms with =stake>"` (an ad-hoc panel,
not saved), `--spend`, `--lens`, `--detail`. Every value that differs from the rule, and every brief above 12 seats,
needs `--reason "<text>"` (`override_without_reason`); the run records `{rule, fields:{name:{rule,given}}, reason, by}` in
`run.override_json` (`--by`, default `agent`), `matrix` JSON carries it as `override`, and `matrix --table` opens with
`override: <fields> — <reason>`. With no rule only the seats are bookkept (see Seat ceilings). `brief` also refuses a `personajury.toml` it reads and cannot parse (`invalid_config`) instead of ignoring it.

### Waves
Every brief answers `waves`: the persona ids of the run in the order of `briefs`, cut into waves of `--wave N` seats
(default 30). Up to 12 seats it is one wave. The engine runs nothing; the skill tells the agent to run one wave, ingest
its verdicts, then start the next, so a 1000-seat jury never floods a rate limit.

### Reported usage
A verdict may carry `usage: {"input_tokens": N, "output_tokens": M}`, non-negative integers and nothing else; any other
shape is refused as `invalid_usage`. The engine measures nothing: `matrix` and `doctor` answer
`usage {reported, estimate}`, where `reported` sums what verdicts carried (`by: "reported by the agent"`, null when none
did) and `estimate` is briefs × `tokens_per_brief`. `matrix --table` adds one line under the table when something was
reported.

### The jury graph
`personajury graph RUN... --svg [--out F]` draws one static SVG, 1200 wide, its height growing with the rows. The layout is
integer arithmetic with no force simulation, so the same store gives the same bytes. Left to right: the personas in three
stratum columns (primary, secondary, bystander, each under its title; the columns descend like stairs so no edge runs behind
another stratum's boxes), one column of dimension nodes per lens (an `@about` option is a sub-node), and the findings. An
edge from a persona to a dimension is coloured by the score on a five-step scale and carries the number as its tooltip;
every dimension node shows one median badge per stratum (P S B), never one combined number. An edge from a persona to a
finding is red when open, green when fixed, grey and dashed when dismissed, blue when it went to a row; a fix that a later
run raised again is open. The finding's label is its short id and the first words of the blocker, cut with an ellipsis; the
full text is the tooltip. Scores and medians come from the latest run of each lens. With several runs of one lens, each
finding carries a strip of dots, one per run: raised, raised again after a fix, fixed, not judged, absent. The card is the
README card (white, 1px border, `role="img"`, a `<title>` and a `<desc>` written from the data). `docs/assets/personajury-graph.svg`
is the graph of the recorded fixture jury; `tests/graph_asset.rs` fails when it differs from a fresh render
(`PERSONAJURY_ASSET_REGEN=1` rewrites it).

### Seat ceilings
A panel or a brief seats at most 12 unless `--allow-large` (on `panel add` and on `brief`), and never more than 1000
(`panel_too_large`, `error` data `{ceiling}`). A brief above 12 seats answers `large: true` and needs `--reason` with or without a rule (`override_without_reason`); with no rule the run records `override_json` as `{rule: null, fields: {seats: {rule: 12, given: N}}, reason, by}`. Every brief answers
`estimate {briefs, tokens}` = briefs × `tokens_per_brief` of `personajury.toml` (default 15000), before anything is spent.
A verdict line may carry `about` (an attachment label of the subject or `cmp:<name>`). `personajury.toml` rules may set
`detail = "compact" | "full"`. Citations in a verdict are `<label>[#locator]` or `cmp:<name>/<label>[#locator]`; `#L<n>` on a diff or a UTF-8 text attachment (a `.md`) must name a line the file has (`citation_unresolved`). The refusals and their error codes are in
the plan's "The refusals" table and in [`troubleshooting.md`](troubleshooting.md).

### `apply-check`
`personajury apply-check RUN... [--rule CLASS]` answers `answer` (`apply | escalate | human | advise`), `runs` (the runs judged: the
latest given run of each lens), `quorum` (`low`, `high`, or `null` when unknown) and `quorum_source` (`run`, `rule`, `sources` or
`unknown`), `next_quorum` when it escalates (low to high, high to human) and `reasons`, each `{code, effect, ...}`. **`apply` only
when every condition is proven; anything unknown, missing or lowered answers `advise` (or worse), never `apply`.** The quorum is the
highest of the quorum recorded on the judged runs (`brief --rule` stores the rule class and its effective quorum), the `--rule`
named here, and the rules the subject's recorded sources fall under.

| code | effect | when |
|---|---|---|
| `human_class` | human | the rule named with `--rule` is a human class (or quorum `human`), or the subject's recorded sources fall under one |
| `primary_blocker` | escalate | a blocker in the primary stratum of any judged run, with that run's own stakes |
| `split` | escalate | within one judged run, two strata's `anchor_met` ranges on a cell do not overlap (touching ranges overlap) |
| `check_failed` | escalate | a finding of any run of this subject has no decision, or a fix did not land |
| `delta_not_landed` | escalate | the delta against the previous round of the same subject and lens did not land |
| `human_blocker` | escalate | a person's batch (`--by-human`) raised a blocker no `decide --by-human` has closed, in any stratum |
| `incomplete_jury` | advise | a briefed seat of a given run has no accepted verdict; `missing` lists them per run |
| `quorum_unknown` | advise | no rule recorded on the run, none named, and no rule holds the subject's sources |
| `rule_below_sources` | advise | the `--rule` named here, or the run's recorded rule, has a lower quorum than the sources' |
| `overridden_down` | advise | the run's recorded override lowers seats (`spend`), lenses or detail below its rule, or replaced its panel, stakes or draw; its effort is below the toml's (`medium` when unset); or it was briefed with no rule while a rule holds the subject |
| `effort_below_quorum` | advise | the quorum is `high` and the run's effort was `low`, or anything was cut by effort or the budget |
| `independence_sequential` | advise | one context read every brief |
| `stale_run` | advise | the subject moved since the run, or a later run of the same subject and lens supersedes it |
| `delta_unavailable` | advise | a previous round exists but the delta cannot be computed: another lens version, or cells it does not have |
| `single_model` | info | one model judged every seat; changes no answer |

Raising seats, lenses, effort or detail above the rule is fine. `human` beats `escalate` beats `advise` beats `apply`. It reads the
store and `personajury.toml`; it writes nothing. The MCP tool `personajury_apply_check` takes `runs` and `rule`. The full rule and the
table test that pins every condition: D13, `tests/apply_check_table.rs`.

### Effort
`effort = "low" | "medium" | "high" | "max"` in `personajury.toml`, `brief --effort` over it (recorded on the run), and an optional
`budget_tokens` (a cap on briefs × `tokens_per_brief`). The rule says what a change needs, effort what you will spend; the run gets
the smaller: `low` 3 seats (one per stratum, primary first) and the rule's first lens, `medium` the rule's seats up to 7, `high` up to 12,
`max` all of them (above 12 still needs `--allow-large` and `--reason`). Applied to a brief with `--rule`, `--effort` or an `effort` in
the toml; medium is the default under a rule. An explicit `--spend` or `--lens` is not cut. A budget drops lenses after the first, then
seats from the end of the draw order, never the last seat of a stratum, else `over_budget` (`data {needed, budget, minimum_plan}`).
Every cut is answered as `trimmed: [{what, from, to, why}]`, recorded on the run and shown by `matrix`. See D17.

### Independence
`brief --independence sequential` declares that one context will read every brief (no subagents: Claude Desktop, a chat window).
The answer adds `sequence` and a sentence; `matrix`, `delta` and `apply-check` carry the word; a sequential run advises and never
applies. The briefs are byte-identical either way. See D18.

### A model per term
A panel term may close with `@model`: `3 builder=primary@claude-sonnet-5-5, 2 approver=secondary@other-model`. `--model` is the
default. A run whose terms name a model states the model in every brief of the run; `matrix` answers `models` per stratum and
`models_distinct`. See D15.

### `lens generate`
`personajury lens generate --from PATH [--dimensions N]` hashes the material (a file, or a folder in path order), writes
`.personajury/generate/lens-<hash>.md` and records the material. The brief asks the agent for two things: what the material says good
looks like (a list, not part of the lens), then one lens file of N dimensions in the exact TOML shape, anchors in words at 1, 3 and 5,
`standard = "material:<hash>"`. Then `lens add --file` validates it. The engine calls no model. See D14.

### Experts
A persona file may carry `credential` (words, at most 500 characters: `credential_too_long`); the tag `expert` is a convention, not a
stratum. A lens's `standard` is shown in every brief and in the matrix header. `verdict --by-human "NAME, CREDENTIAL"` ingests a
person's batch with the same refusals: the file's `persona` is the seat whose stratum the person speaks in. The rows stand beside the
jury's (matrix JSON `human`, an `H` block under the table), never in a cell, a median or a split. Only a person enters one: the MCP
verdict tool has no such argument. A blocker in it escalates `apply-check` until a recorded `decide --by-human` closes the finding. See D19.

### `summary`
`personajury summary RUN... [--html F] [--md F] [--save] [--width N]` (MCP `personajury_summary`, which takes `runs`, `width`, `md` and `html`) answers `text`: 2 to 5
plain sentences (the apply-check answer in words with its counts, who won with counts per stratum, the weakest dimension per stratum,
the change since the previous round as "15 → 6", and up to three blockers quoted with their persona and decision, taken in turns across jurors, primary first, one per juror before anyone's
second), then the
numbers table "for checking" (one row per option or subject, one column per dimension, each cell the medians `P·S·B`, a dash for a stratum
nobody judged, the top 5 rows by the primary stratum), then `Next:` chosen from the apply-check answer. No bars, no pooled number.
`--width` is 30 to 200 (default 80). `--html F` writes one offline file (inline CSS, the `graph --svg` picture, every per-stratum table, the
quotes; no scripts), `--md F` the same summary as markdown with the decisions per finding, the run ids and brief hashes and the date of the
latest run. `--save` writes the markdown to `<report_dir>/<date>-<subject>.md` and is refused (`no_report_dir`) when `personajury.toml` has no
top-level `report_dir`. `report_dir` must be relative, hold no `..` and resolve inside the project root (a symlink out is refused), else
`invalid_report_dir`; the MCP tool's `md` and `html` paths are checked the same way and are relative to the root. A path typed at the CLI
may be anywhere. Nothing is written without one of the three flags. **Text safety:** everything the summary prints (text, markdown,
html) has ESC sequences removed whole, and every other C0 control but the newline, DEL and the C1 controls removed (a tab becomes a
space). A quote keeps the juror's words; runs of whitespace, newlines included, fold to one space. A long quote is cut only between words
and ends with `…`; a word is never cut (one longer than the line stands whole on its own line). The golden outputs are `fixtures/run/summary.txt` and `summary.md`.

### `calibrate`
`personajury calibrate JURY_RUN HUMAN_RUN [--out F]`: the human run is an ordinary run whose verdict batches were entered with `--by-human`
(else `not_human_run`), on the same subject and lens version. Per stratum, the cells (lens, dimension, about) are ranked by the stratum's
median, high first, ties sharing a rank, for the jury and for the people; each cell answers both ranks and `within_one_rank`. `calibrated`
is true iff every primary-stratum cell is within one rank and every primary cell the jury judged was also judged by a person; bystander and
secondary differences are listed in `outside_primary`, never averaged in. `apply-check RUN... --calibration F` (MCP `calibration`) turns
`apply` into `advise` with the reason `uncalibrated` when the file says `calibrated: false`. The file must be the calibration of the
runs given: its `subject`, `lens`, `lens_version` and `jury_run` (one of the runs given) must match, else `calibration_mismatch`.
`apply-check --latest` judges the latest round of every subject that has runs, each on its own, and answers the worst (`human` >
`escalate` > `advise` > `apply`); `subjects` lists each subject's own answer and reasons, and the flat `reasons` name their `subject`.
`--latest --subject S` judges S only. This is what CI reads.

### CSV cell locators
A text attachment whose label ends `.csv` can be cited by cell: `prices.csv#R2C3`, 1-based, the header row is row 1. A row is a record
(RFC 4180: quoted fields, doubled quotes, CRLF), not a line: a quoted line break does not add a row. Out of range is `citation_unresolved`;
`R01C1`, `R+1C1`, lowercase `r9c9`, `R1c1` and `RC1` are refused like `L01`. A quote opens a quoted field only at the start of a
field; in the middle of an unquoted field (`27" screen`) it is a literal character (RFC 4180 leaves that case malformed; PersonaJury
reads it the lenient way). `#L<n>` keeps working.
