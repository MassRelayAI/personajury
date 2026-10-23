# Plans

One plan per release, named `YYYY-MM-DD-<release>-<subject>.md`, written before the code and executed task by task. The v0.2 to v0.5 plans were written after the build and say "as built" at the top; the measured result of each is in `../reports/`.

| plan | scope | status |
|---|---|---|
| [2026-10-23-v0-1-the-engine.md](2026-10-23-v0-1-the-engine.md) | the v0.1 engine: store, verbs, refusals, MCP server, skills, fixture replay | in progress |
| [2026-10-24-v0-2-panels-and-the-draw.md](2026-10-24-v0-2-panels-and-the-draw.md) | rules pick the panel, a seeded draw, seat ceilings, overrides with a reason (D9, D10) | built, as built |
| [2026-10-24-v0-3-the-jury-graph.md](2026-10-24-v0-3-the-jury-graph.md) | the jury graph SVG, waves, reported usage, finding lives (D11, D12) | built, as built |
| [2026-10-24-v0-4-the-apply-check.md](2026-10-24-v0-4-the-apply-check.md) | apply-check, effort, independence, experts, model per term, packaging (D13 to D19) | built, as built |
| [2026-10-24-v0-5-summary-calibration-ci.md](2026-10-24-v0-5-summary-calibration-ci.md) | the summary, calibration, CSV cells, the GitHub Action, the SDK recipe (D20 to D24) | built, as built |

A plan states its global constraints and, per task, the files the task owns, the tests that go red first and the commit. The
table of refusals in the v0.1 plan is the contract that `docs/testing.md`, `docs/commands.md` and `docs/troubleshooting.md`
point to. When the build teaches something the plan did not know, write a numbered entry in `../decisions.md`; do not rewrite
history in the plan.
