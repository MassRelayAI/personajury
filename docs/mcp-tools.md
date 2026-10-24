<!-- shape follows comparegraph docs/mcp-tools.md -->
# MCP tools

`personajury mcp` serves MCP over stdio. Eight tools, each the same call as its CLI verb and answering with the same envelope
(including the same refusals). Their arguments carry the CLI flags' names. This is the v0.1 contract in the plan; the server
is built in plan Task 6.

| tool | CLI verb | notes |
|---|---|---|
| `personajury_brief` | `brief` | takes `detail` (`compact` default, `full`), and the panel in one of three ways: `panel` (saved), `query` (ad-hoc, not saved) or `rule` (a `personajury.toml` class: its panel, lenses, spend and detail are the defaults). `lens` is a name or an array (one run each); `stakes` is optional (a panel term carries its own); `draw` sets the seed; `allow_large` lifts the 12-seat ceiling to 1000. A value that differs from the rule, or a brief above 12 seats, needs `reason` (and may carry `by`): refused as `override_without_reason` otherwise, recorded on the run. `wave` sets the seats per wave. `effort` (`low`, `medium`, `high`, `max`) says how much the run may spend; `independence` (`independent`, `sequential`) says whether one context per brief runs the jurors (a sequential answer carries `sequence`). A panel term may name its model (`3 builder=primary@model`). `traits` (a trait set, `all` or `off`) renders the personas' traits as "you tend to" lines and `seats: "auto"` seats the personas that serve the subject's audience plus one control (answered as `seats`); a panel with a cover answers `coverage`. The answer carries `waves`, `estimate {briefs, tokens}`, `draw.seed`, `large` |
| `personajury_verdict` | `verdict` | takes `run` and `verdict` as an object (lines carry `about`, as in [`usage.md`](usage.md)); a verdict without evidence is refused exactly as on the CLI; it may carry `usage {input_tokens, output_tokens}` (`invalid_usage`) |
| `personajury_matrix` | `matrix` | takes `detail` (`compact` default, `full`) and `slice` (a trait id: `data.slices`, per stratum, counted); scores per stratum and `usage {reported, estimate}` |
| `personajury_graph` | `graph` | `runs` (one or more runs of one subject); answers the SVG text in `data.svg`; reads only, writes no file |
| `personajury_delta` | `delta` | two runs |
| `personajury_apply_check` | `apply-check` | `runs`, `rule` and `calibration` (a file from `calibrate --out`); answers `apply`, `escalate`, `human` or `advise` with the reasons as data; reads only |
| `personajury_trait_list` | `trait list` | the trait catalog the store holds (`data.traits`), or with `sets: true` the trait sets; reads only; install the catalog with the CLI |
| `personajury_summary` | `summary` | `runs`, `width`, `slice` (one sentence per stratum on a trait), and optional `md` / `html` (relative paths inside the project root, else `invalid_report_dir`); answers the text in `data.text` (sentences, numbers for checking, Next); writes a file only when `md` or `html` is given |
| `personajury_should_run` | `should-run` | classifies a diff, a list of paths or a subject's recorded source paths (`subject`) |

There is no tool that enters a person's verdict (`--by-human`) or records a decision: a person does that. Everything else (adding personas, panels, lenses and subjects, `lens generate`, `pick`, `check`, `decide`) is CLI only. Register the
server with `personajury setup --apply`; `personajury doctor` shows each client's registration. The engine makes no model call
and no network call whichever way it is driven.
