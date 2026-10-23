<!-- shape follows comparegraph docs/roadmap.md -->
# Roadmap

Only what the design report names. Nothing here has a date.

## v0.1 (this plan)
The engine in [`plans/2026-10-23-v0-1-the-engine.md`](plans/2026-10-23-v0-1-the-engine.md): the store, the verbs, the
refusals, the MCP server, two skills, the fixture site and its replayed run. Status: being built; see `../CHANGELOG.md`.

## After v0.1
- **A CLI runner over an OpenAI-compatible endpoint, for CI.** Report section 2.5: "not v0.1". Not built. It would be a
  separate component; the engine would still never hold a key.
- **Split escalation computed by the engine.** Done in v0.4 (D13): `matrix` reports `splits`, `apply-check` answers.
- **Adoption by a larger product.** Report section 9: personas and panels as data in its packs, and a run as a gate. That
  work lives in that product, not here. Not built.

## v0.4 (done)
The split rule and `apply-check` (D13), `lens generate` (D14), a model per panel term (D15), packaging with the `.mcpb` bundle
(D16), effort (D17), independence (D18), experts and a human verdict (D19). See `../CHANGELOG.md`.

## v0.5 (what is left)
- **Calibration.** `calibrate <jury-run> <human-run>`: the jury's rank order against human batches per stratum, never a pooled
  number; `apply-check --calibration` turns an uncalibrated jury's `apply` into `escalate`. Needs real people's verdicts
  (`verdict --by-human` ships now so they can be collected).
- **CSV cell locators** (`<label>#R<row>C<col>`) for spreadsheets and tables.
- **A composite GitHub Action:** `should-run` on a PR diff, and `apply-check` when the store is in the checkout.
- **The Claude Agent SDK recipe** (a docs page, no code in the tree), which replaces the idea of a runner of our own.
- **Listings:** the official Claude plugin marketplace and the Codex plugin directory, after a first published result.
- A `.mcpb` bundle checked on a real Claude Desktop (v0.4 ships it unverified).
