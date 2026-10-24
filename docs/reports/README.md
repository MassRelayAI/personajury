<!-- adapted from comparegraph docs/reports/README.md (shape) -->
# Reports

Every measurement, proof and bake-off ends in a dated file here, with `date`, `valid_until`, `verdict` and `status` in its
front matter, written before anyone claims the number. The README quotes numbers only from these files. A report past its
`valid_until` is re-run or marked stale, never silently kept.

## Publishing rule
A jury result shared outside a project carries the date, the model as declared (`--model`, unverified), the subject, panel,
lens and brief hashes, and the per-stratum table (primary, secondary, bystander side by side, never averaged). A result
without them is prose and is labelled as such. A verdict about a named comparator is a fictional persona's opinion on a
dated, hashed snapshot, not a claim about the comparator. Full rule: [`../publishing-results.md`](../publishing-results.md).

```yaml
---
date: 2026-11-01
valid_until: 2026-12-01
verdict: "one sentence a reader can act on"
status: measured        # measured | decided | stale | superseded
---
```

| date | report | verdict in one line | status |
|---|---|---|---|
| 2026-10-24 | [The README jury](2026-10-24-the-readme-jury.md) | 10 composite personas, 4 lenses, 2 rounds: 32 findings in round 1 (27 fixed); critical blockers 22 → 9 and docs 5 → 3, landed; compare not landed (per-screenshot price notes, an open question); picks for this README 4 of 10 → 7 of 10 | decided |
| 2026-10-24 | [The name, picked by a persona jury](2026-10-24-the-name-picked-by-a-persona-jury.md) | JuryPersona, 8 of 8 forced picks over seven other names | decided |
| 2026-10-24 | [v0.2 measured: panels, the draw and overrides](2026-10-24-v0-2-measured.md) | 292 tests green after a review that found 6 Important findings, all fixed; the ceiling of 12 is still a guess | measured |
| 2026-10-24 | [v0.3 measured: the jury graph, waves and usage](2026-10-24-v0-3-measured.md) | 332 tests green; the graph is byte-identical and goes red when one score moves; 3 Important findings fixed | measured |
| 2026-10-24 | [v0.4 measured: the apply check](2026-10-24-v0-4-measured.md) | 409 tests green; the review found 5 Critical cases where apply-check said apply, all fixed; the .mcpb bundle is unverified | measured |
| 2026-10-24 | [v0.5 measured: summary, calibration, CSV cells, CI](2026-10-24-v0-5-measured.md) | 471 tests green; the CI gate now judges every subject and fails closed; the Action has not run on GitHub | measured |
| 2026-10-24 | [How to show a jury's answer](2026-10-24-display-jury.md) | sentences first, numbers for checking, no bars: bars got 0 of 8 picks, numbers and sentences tied 4 to 4 by reader | decided |

The held-out check is still to be reported: a jury run on the fixture site compared with the
answer key in `fixtures/README.md` (see `../testing.md`).
