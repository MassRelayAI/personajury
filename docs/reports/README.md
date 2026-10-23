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

No report exists yet. The first is expected from the held-out check: a jury run on the fixture site compared with the
answer key in `fixtures/README.md` (see `../testing.md`).
