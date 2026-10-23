<!-- shape follows comparegraph docs/roadmap.md -->
# Roadmap

Only what the design report names. Nothing here has a date.

## v0.1 (this plan)
The engine in [`plans/2026-10-23-v0-1-the-engine.md`](plans/2026-10-23-v0-1-the-engine.md): the store, the verbs, the
refusals, the MCP server, two skills, the fixture site and its replayed run. Status: being built; see `../CHANGELOG.md`.

## After v0.1
- **A CLI runner over an OpenAI-compatible endpoint, for CI.** Report section 2.5: "not v0.1". Not built. It would be a
  separate component; the engine would still never hold a key.
- **Split escalation computed by the engine.** v0.1 reports strata and the agent applies the escalation rule; the engine
  does not compute a split. The rule for what counts as a split is not decided yet. Not built.
- **Adoption by a larger product.** Report section 9: personas and panels as data in its packs, and a run as a gate. That
  work lives in that product, not here. Not built.
