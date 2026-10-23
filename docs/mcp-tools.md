<!-- shape follows comparegraph docs/mcp-tools.md -->
# MCP tools

`jurygraph mcp` serves MCP over stdio. Six tools, each the same call as its CLI verb and answering with the same envelope
(including the same refusals). Their arguments carry the CLI flags' names. This is the v0.1 contract in the plan; the server
is built in plan Task 6.

| tool | CLI verb | notes |
|---|---|---|
| `jurygraph_brief` | `brief` | takes `detail` (`compact` default, `full`), and the panel in one of three ways: `panel` (saved), `query` (ad-hoc, not saved) or `rule` (a `jurygraph.toml` class: its panel, lenses, spend and detail are the defaults). `lens` is a name or an array (one run each); `stakes` is optional (a panel term carries its own); `draw` sets the seed; `allow_large` lifts the 12-seat ceiling to 1000. A value that differs from the rule, or a brief above 12 seats, needs `reason` (and may carry `by`): refused as `override_without_reason` otherwise, recorded on the run. `wave` sets the seats per wave. The answer carries `waves`, `estimate {briefs, tokens}`, `draw.seed`, `large` |
| `jurygraph_verdict` | `verdict` | takes `run` and `verdict` as an object (lines carry `about`, as in [`usage.md`](usage.md)); a verdict without evidence is refused exactly as on the CLI; it may carry `usage {input_tokens, output_tokens}` (`invalid_usage`) |
| `jurygraph_matrix` | `matrix` | takes `detail` (`compact` default, `full`); scores per stratum and `usage {reported, estimate}` |
| `jurygraph_graph` | `graph` | `runs` (one or more runs of one subject); answers the SVG text in `data.svg`; reads only, writes no file |
| `jurygraph_delta` | `delta` | two runs |
| `jurygraph_should_run` | `should-run` | classifies a diff, a list of paths or a subject's recorded source paths (`subject`) |

Everything else (adding personas, panels, lenses and subjects, `pick`, `check`, `decide`) is CLI only in v0.1. Register the
server with `jurygraph setup --apply`; `jurygraph doctor` shows each client's registration. The engine makes no model call
and no network call whichever way it is driven.
