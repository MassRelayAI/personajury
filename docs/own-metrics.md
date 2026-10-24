# Your own metrics

PersonaJury reports a fixed set of numbers: medians, blockers, agreement, shift, panel match and cost, each per stratum
([D34](decisions.md)). You may want another one. You do not need a new verb for it. Every verb prints JSON, the store is a plain
SQLite file, and `personajury export` writes the raw verdict rows ([D40](decisions.md)). So any metric for any use case is a short
script.

One rule carries over: **count per stratum, never pooled across strata.** The engine refuses to pool; your script should too.

## An example: who gave the low scores?

The question: of the lines scored 2 or below, how many came from a juror who declared `trait.trust.marketing_skepticism` 4 or 5?
The data source is the CSV export. The script loads it into SQLite in memory and counts, grouped by stratum. It uses the Python
standard library only.

```bash
personajury export 1 2 --format csv --out rows.csv
python3 skeptic_low.py rows.csv
```

`skeptic_low.py`:

```python
# Per stratum: of the lines scored 2 or below, how many came from a juror
# who declared trust.marketing_skepticism 4 or 5. Never pooled across strata.
import csv, sqlite3, sys

db = sqlite3.connect(":memory:")
rows = list(csv.DictReader(open(sys.argv[1], newline="")))
db.execute("CREATE TABLE r (stratum, persona, score INT, skeptic INT)")
db.executemany("INSERT INTO r VALUES (?, ?, ?, ?)",
    [(x["stratum"], x["persona"], int(x["score"]), int(x["trait.trust.marketing_skepticism"] or 0)) for x in rows])
q = """SELECT stratum, COUNT(*), SUM(skeptic >= 4), COUNT(DISTINCT persona)
       FROM r WHERE score <= 2 GROUP BY stratum
       ORDER BY instr("primary secondary bystander", stratum)"""
for stratum, low, by_skeptics, jurors in db.execute(q):
    print(f"{stratum:10} {low:3} low scores, {by_skeptics:3} by skeptics, from {jurors} jurors")
```

Run on the fixture jury (`fixtures/run/setup.sh`, then the recorded verdicts of runs 1 and 2, as in the
[README](../README.md#try-it)), the export has 147 rows and the script prints:

```text
primary     50 low scores,  29 by skeptics, from 4 jurors
secondary   32 low scores,  16 by skeptics, from 2 jurors
bystander   12 low scores,  12 by skeptics, from 1 jurors
```

Read it with care. The bystander stratum is one juror, so "12 of 12" says one persona is a skeptic, not that skeptics score low.
A count is a bearing, not a finding. The fixture is small: seven personas, two runs.

To change the metric, change the `WHERE` and the `SUM`. Any `trait.<id>` column of the export works the same way; it is empty
when the persona does not hold that trait.

## What is stable to read

**The export is the stable interface.** Its columns and their order are fixed by D40 and held by the golden files
`fixtures/run/export.csv` and `fixtures/run/export.ndjson`: `run`, `subject`, `subject_version`, `lens`, `lens_version`, `persona`,
`stratum`, `model`, `dimension`, `about`, `score`, `anchor_met`, `evidence`, `citations`, `blockers` (a count), `phase`, `by_human`,
then one `trait.<id>` column per trait (sorted), then `sensitive`. A new column would be named in
the [changelog](../CHANGELOG.md). Prefer the export: it already joins each line to its stratum, which the store keeps in the run.

**The store, read only.** `.personajury/store.db` holds schema version 6 in v0.6.0 (`SELECT value FROM meta WHERE key =
'schema_version'`). These tables and columns are safe to read in schema 6:

| table | columns |
|---|---|
| `run` | `id`, `subject_name`, `subject_version`, `lens_name`, `lens_version`, `model`, `stakes_json` (persona to stratum), `independence`, `quorum`, `created_at` |
| `verdict` | `run_id`, `batch`, `persona_id`, `dimension`, `score`, `anchor_met`, `evidence`, `about`, `citations_json`, `blockers_json`, `note`, `phase`, `created_at` |
| `pick` | `run_id`, `batch`, `persona_id`, `choice`, `reason`, `about` |
| `finding` | `id`, `run_id`, `persona_id`, `dimension`, `about`, `text` |
| `decision` | `finding_id`, `outcome`, `why`, `row_ref`, `by`, `after_run` |
| `persona` | `id`, `version`, `tags_json`, `traits_json` |
| `usage` | `run_id`, `persona_id`, `input_tokens`, `output_tokens` (reported by the agent, not measured) |

Verdicts, picks, usage and decisions are append-only: the latest `batch` of a persona in a run is its answer, and the earlier
batches are history. Other tables and columns are internal and may change. A store from a newer version may have more columns;
read columns by name, never by position.

**Writing to the store directly is unsupported.** The verbs check what goes in: citations, scales, strata, the refusals. A row
written by hand skips every one of those checks, and the append-only tables refuse an update or a delete. Use the verbs to write.

## Other sources

- `personajury matrix RUN... --pretty` gives the per-stratum numbers the engine computes, as JSON.
- `personajury export RUN...` without `--out` prints ndjson, one row per line, for `jq` or any pipeline.
- `personajury export RUN... --phase all` adds the deliberated rows; each row names its phase.
