# JuryPersona v0.3 — the jury graph — Implementation Plan (as built)

> **As built.** Written after the work, from the code, the tests and the review. Decisions of record: D11 and D12 in
> [`../decisions.md`](../decisions.md). Measured result: [`../reports/2026-10-24-v0-3-measured.md`](../reports/2026-10-24-v0-3-measured.md).

**Goal:** One static, deterministic picture of a jury (`graph RUN... --svg`) that shows personas by stratum, dimensions with a median
badge per stratum, and findings with their life across rounds; plus the small pieces a large jury needs: waves, reported usage, the
allowed `anchor_met` values in the brief, a reason for any brief above 12 seats.

**Architecture:** `application/graph.rs` builds the string, `domain/graphlayout.rs` does integer layout over ordered containers. No
force simulation, no float, no new crate. Medians come from `domain::matrix::build`, so the picture can never show a pooled number.

## Global Constraints

- The same store gives the same SVG bytes.
- One badge per stratum on a dimension node (`P S B`), never one combined number.
- `fixture_replay` stays green. Brief bytes change once (D12); only the recorded brief hashes in `fixtures/run/briefs.json` are regenerated.

## Decisions it made

| # | what |
|---|---|
| D11 | The graph: columns that descend like stairs so no edge runs behind another stratum's boxes; edge colour by score or by decision; rounds as a strip of dots; the MCP tool count goes 5 to 6 |
| D12 | Waves (`--wave N`, default 30); `usage` on a verdict (table `usage`, `invalid_usage`); the allowed anchors in the brief; a reason for any brief above 12 seats; a malformed `jurypersona.toml` is refused; comma-separated `--comparator`. Brief hashes: 14 of 14 recorded hashes changed |

## Tasks (six commits)

1. **Waves, large-brief reason, allowed anchors, comma comparators, malformed toml.** Tests `tests/v03.rs`: red 10 failed / 1 passed (the
   one that passed is the control: a verdict without usage still ingests), green 11 of 11.
2. **A verdict may report its usage.** Append-only table `usage` (`CREATE TABLE IF NOT EXISTS`, schema version stays 3); `invalid_usage`.
3. **Test: a refused usage stores nothing.**
4. **`check --history` lists each finding's life across runs** (`lives`, the same function the graph uses). 3 tests red, then green.
5. **`graph RUN... --svg [--out F]` and the MCP tool `jurypersona_graph`.** 12 graph tests red (the verb did not exist), 15 green; the MCP
   list test red (5 tools against 6), then green; an MCP call test added after (16 in the file).
6. **Docs:** D11, D12, `commands.md`, `mcp-tools.md`, `troubleshooting.md`, the changelog, the skill's waves.

Also: `docs/assets/` holds the graph of the recorded fixture run (1200 x 902); `tests/graph_asset.rs` goes red when the committed file
differs from a fresh render (seen red by changing one score in the fixture verdict, then restored).

## Review and the fix round

The review found no Critical, three Important and five Minor findings; one fix round closed all eight, seven new tests seen red first.
Details in the measured report.
