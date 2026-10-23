<!-- shape follows comparegraph docs/mcp-tools.md -->
# MCP tools

`jurygraph mcp` serves MCP over stdio. Five tools, each the same call as its CLI verb and answering with the same envelope
(including the same refusals). Their arguments carry the CLI flags' names. This is the v0.1 contract in the plan; the server
is built in plan Task 6.

| tool | CLI verb | notes |
|---|---|---|
| `jurygraph_brief` | `brief` | takes `detail` (`compact` default, `full`) |
| `jurygraph_verdict` | `verdict` | takes `run` and `verdict` as an object (lines carry `about`, as in [`usage.md`](usage.md)); a verdict without evidence is refused exactly as on the CLI |
| `jurygraph_matrix` | `matrix` | takes `detail` (`compact` default, `full`); scores per stratum |
| `jurygraph_delta` | `delta` | two runs |
| `jurygraph_should_run` | `should-run` | classifies a diff or a list of paths |

Everything else (adding personas, panels, lenses and subjects, `pick`, `check`, `decide`) is CLI only in v0.1. Register the
server with `jurygraph setup --apply`; `jurygraph doctor` shows each client's registration. The engine makes no model call
and no network call whichever way it is driven.
