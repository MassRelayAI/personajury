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
| `panel add` | `NAME` `--query Q` | save a query over persona tags, e.g. `"5 builder, 3 company, 2 others"` |
| `panel resolve` | `NAME` | the personas the query resolves to now |
| `lens add` | `--file F` | add a lens (TOML) |
| `lens list` | | list lenses; four ship (a 1 to 10 `site` lens is fixture data: `lens add --file fixtures/lenses/site.toml`) |
| `subject add` | `NAME` `--kind K` `--attach FILE...` | a subject as attached files; kind is `page`, `diff`, `design`, `logo`, `document`, `menu` or `prices` |
| `comparator add` | same as `subject add` | a comparator, stored the same way and flagged |
| `brief` | `--subject S` `--panel P` `--lens L` `--model M` `--stake p=stake,...` `[--spend N]` `[--comparator C]...` `[--detail compact\|full]` | one deterministic, hashed brief per persona; creates the run |
| `verdict` | `--run N` and `--file F` or `--json J` | ingest one persona's verdict; refused without evidence |
| `matrix` | `RUN...` `[--history]` `[--table]` `[--detail compact\|full]` | scores per stratum; `--across-strata` exists only so the refusal is explicit |
| `pick` | `RUN` | the forced choices of a run, per stratum, with reasons |
| `delta` | `A B` | what moved between two runs on the same subject and lens |
| `check` | `[--history]` | stale runs (source changed or a newer subject version), `fixed_unverified`, and findings with no decision from any batch or fixed and raised again (`fix_not_landed`); exits 1 with `check_failed` listing `unresolved` |
| `decide` | `FINDING OUTCOME` `[--why W]` `[--row R]` `--by WHO` | outcome is `fixed`, `dismissed` (with `--why`) or `row` (with `--row`) |
| `should-run` | `[--diff F]` `[--paths a,b]` `[--config F]` | classify a change: `none`, `low`, `high`, `human`, and the spend |
| `setup` | `[--client C]` `[--scope S]` `[--apply]` `[--force]` `[--bin B]` `[--no-rules]` `[--no-hook]` | register the MCP server; a plan unless `--apply` |
| `doctor` | `[--bin B]` `[--brief]` | the store, counts and registrations; `--brief` is two plain lines for a hook |
| `mcp` | | serve MCP over stdio |

`--stake` takes `persona=primary|secondary|bystander`, repeated or comma-separated, for every persona in the panel.
A verdict line may carry `about` (an attachment label of the subject or `cmp:<name>`). `jurygraph.toml` rules may set
`detail = "compact" | "full"`. Citations in a verdict are `<label>[#locator]` or `cmp:<name>/<label>[#locator]`. The refusals and their error codes are in
the plan's "The refusals" table and in [`troubleshooting.md`](troubleshooting.md).
