<!-- shape follows comparegraph docs/commands.md -->
# Commands

Every verb prints one JSON envelope (`ok, verb, schema, data, evidence, cost, error`) and exits 0 on success, 1 on error.
Global flags: `--root DIR` (default `.`, the store is `DIR/.jurygraph/store.db`) and `--pretty`. `jurygraph --version` prints
the version.

| verb | arguments and flags | what it does |
|---|---|---|
| `persona add` | `--file F` `[--generated-from HASH]` | add one persona from the fixed JSON |
| `persona generate` | `--from PATH` `[--count N]` (default 7) | write the brief that asks your agent to write personas from material |
| `persona list` | | list personas |
| `panel add` | `NAME` `--query Q` `[--allow-large]` | save a query over persona tags with a stake per term, e.g. `"3 builder=primary, 1 newcomer=primary, 2 approver=secondary, 1 others=bystander"`; more than 12 seats is refused (`panel_too_large`) unless `--allow-large`, and more than 1000 always |
| `panel resolve` | `NAME` `[--subject S]` | the personas the query resolves to now; with `--subject` the seats a draw gives that subject (`draw.seed`), without it the first by id (`draw: "none (no subject)"`) |
| `lens add` | `--file F` | add a lens (TOML) |
| `lens list` | | list lenses; four ship (a 1 to 10 `site` lens is fixture data: `lens add --file fixtures/lenses/site.toml`) |
| `subject add` | `NAME` `--kind K` `--attach FILE...` | a subject as attached files; kind is `page`, `diff`, `design`, `logo`, `document`, `menu` or `prices` |
| `comparator add` | same as `subject add` | a comparator, stored the same way and flagged |
| `brief` | `--subject S` `--model M` and `--panel P` or `--query Q` or `--rule R`; `[--lens L,...]` `[--stake p=stake,...]` `[--spend N]` `[--comparator C]...` `[--detail compact\|full]` `[--draw HEX]` `[--wave N]` `[--allow-large` `--reason W` `--by WHO]` | one deterministic, hashed brief per persona, one run per lens; `--lens` and `--comparator` take a comma list or repeat; the answer carries `waves`, `estimate {briefs, tokens}` and `draw.seed` |
| `verdict` | `--run N` and `--file F` or `--json J` | ingest one persona's verdict; refused without evidence; may carry `usage {input_tokens, output_tokens}` (`invalid_usage` when malformed) |
| `matrix` | `RUN...` `[--history]` `[--table]` `[--detail compact\|full]` | scores per stratum, and `usage {reported, estimate}`; `--across-strata` exists only so the refusal is explicit |
| `graph` | `RUN...` `--svg` `[--out F]` | the jury as one static SVG (see below); several runs of one subject draw each finding's life; answers `data.svg`, or writes `--out` |
| `pick` | `RUN` | the forced choices of a run, per stratum, with reasons |
| `delta` | `A B` | what moved between two runs on the same subject and lens |
| `check` | `[--history]` | stale runs (source changed or a newer subject version), `fixed_unverified`, and findings with no decision from any batch or fixed and raised again (`fix_not_landed`); exits 1 with `check_failed` listing `unresolved`; `--history` adds `decisions` and `lives`, each finding's life across the runs (`raised`, `raised_again`, `fixed`, `unjudged`, `absent`) |
| `decide` | `FINDING OUTCOME` `[--why W]` `[--row R]` `--by WHO` | outcome is `fixed`, `dismissed` (with `--why`) or `row` (with `--row`) |
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
`--rule CLASS` reads `<root>/jurygraph.toml` and takes the rule's panel, lenses, spend (`0` = the whole panel) and detail
as defaults, one run per lens (the answer has `runs`). A rule with quorum `low` or `high` must name a `panel` and at least
one lens (`rule_without_panel`, `invalid_config`); a rule naming a panel or lens the store lacks is `rule_panel_unknown` /
`rule_lens_unknown`; a rule of a human class, or quorum `human`, is refused (`human_class_under_persona_quorum`), and
quorum `none` runs no jury. The caller may override with `--panel`, `--query "<terms with =stake>"` (an ad-hoc panel,
not saved), `--spend`, `--lens`, `--detail`. Every value that differs from the rule, and every brief above 12 seats,
needs `--reason "<text>"` (`override_without_reason`); the run records `{rule, fields:{name:{rule,given}}, reason, by}` in
`run.override_json` (`--by`, default `agent`), `matrix` JSON carries it as `override`, and `matrix --table` opens with
`override: <fields> — <reason>`. With no rule only the seats are bookkept (see Seat ceilings). `brief` also refuses a `jurygraph.toml` it reads and cannot parse (`invalid_config`) instead of ignoring it.

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
`jurygraph graph RUN... --svg [--out F]` draws one static SVG, 1200 wide, its height growing with the rows. The layout is
integer arithmetic with no force simulation, so the same store gives the same bytes. Left to right: the personas in three
stratum columns (primary, secondary, bystander, each under its title; the columns descend like stairs so no edge runs behind
another stratum's boxes), one column of dimension nodes per lens (an `@about` option is a sub-node), and the findings. An
edge from a persona to a dimension is coloured by the score on a five-step scale and carries the number as its tooltip;
every dimension node shows one median badge per stratum (P S B), never one combined number. An edge from a persona to a
finding is red when open, green when fixed, grey and dashed when dismissed, blue when it went to a row; a fix that a later
run raised again is open. The finding's label is its short id and the first words of the blocker, cut with an ellipsis; the
full text is the tooltip. Scores and medians come from the latest run of each lens. With several runs of one lens, each
finding carries a strip of dots, one per run: raised, raised again after a fix, fixed, not judged, absent. The card is the
README card (white, 1px border, `role="img"`, a `<title>` and a `<desc>` written from the data). `docs/assets/jury-graph.svg`
is the graph of the recorded fixture jury; `tests/graph_asset.rs` fails when it differs from a fresh render
(`JURYGRAPH_ASSET_REGEN=1` rewrites it).

### Seat ceilings
A panel or a brief seats at most 12 unless `--allow-large` (on `panel add` and on `brief`), and never more than 1000
(`panel_too_large`, `error` data `{ceiling}`). A brief above 12 seats answers `large: true` and needs `--reason` with or without a rule (`override_without_reason`); with no rule the run records `override_json` as `{rule: null, fields: {seats: {rule: 12, given: N}}, reason, by}`. Every brief answers
`estimate {briefs, tokens}` = briefs × `tokens_per_brief` of `jurygraph.toml` (default 15000), before anything is spent.
A verdict line may carry `about` (an attachment label of the subject or `cmp:<name>`). `jurygraph.toml` rules may set
`detail = "compact" | "full"`. Citations in a verdict are `<label>[#locator]` or `cmp:<name>/<label>[#locator]`; `#L<n>` on a diff or a UTF-8 text attachment (a `.md`) must name a line the file has (`citation_unresolved`). The refusals and their error codes are in
the plan's "The refusals" table and in [`troubleshooting.md`](troubleshooting.md).
