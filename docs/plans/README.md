# Plans

One plan per release, named `YYYY-MM-DD-<release>-<subject>.md`, written before the code and executed task by task.

| plan | scope | status |
|---|---|---|
| [2026-10-23-v0-1-the-engine.md](2026-10-23-v0-1-the-engine.md) | the v0.1 engine: store, verbs, refusals, MCP server, skills, fixture replay | in progress |

A plan states its global constraints and, per task, the files the task owns, the tests that go red first and the commit. The
table of refusals in the v0.1 plan is the contract that `docs/testing.md`, `docs/commands.md` and `docs/troubleshooting.md`
point to. When the build teaches something the plan did not know, write a numbered entry in `../decisions.md`; do not rewrite
history in the plan.
