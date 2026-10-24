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

## v0.5 (done)
Calibration (D20), CSV cell locators (D21), the GitHub Action (D22), the Claude Agent SDK recipe (D23), and the answer display `summary`
(D24). See `../CHANGELOG.md`.

## v0.6 (in progress, three lanes)
- **A, traits as data (D29 to D31):** the trait catalog and sets, schema v5, the persona `traits` key, `brief --traits`, panels with
  cover and quotas, slices and `summary --slice`, `--seats auto`. Built.
- **C, options, deliberation and honest metrics:** not built here.
- **B, traits proven (calibrate per trait, `trait observed`):** not built here.

## After v0.5 (what is left)
- **Calibration with real people.** `calibrate` ships; it needs a person's verdicts (`verdict --by-human`) on a real subject to say anything.
- **Listings:** the official Claude plugin marketplace and the Codex plugin directory, after a first published result.
- A `.mcpb` bundle checked on a real Claude Desktop (v0.4 ships it unverified).
