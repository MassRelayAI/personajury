<!-- shape follows comparegraph docs/usage.md -->
# Usage

JuryGraph runs a persona jury as an engine. It never calls a model: your agent runs the personas. This page is the loop; the
flags are in [`commands.md`](commands.md). Behaviour described here is the v0.1 contract in the plan.

## The loop
1. **Set up once.** `jurygraph setup --apply` registers the MCP server with the agents it finds (without `--apply` it prints
   a plan). `jurygraph doctor` reports the store and each registration. The store is `.jurygraph/store.db` under `--root`
   (default `.`).
2. **Personas and a panel.** `jurygraph persona add --file p.json` per persona, or `jurygraph persona generate --from
   material.md` to write a brief that asks your agent to write them. `jurygraph panel add NAME --query "3 builder, 2 company,
   2 others"` saves a query over persona tags; `jurygraph panel resolve NAME` shows who it resolves to.
3. **A subject.** `jurygraph subject add NAME --kind page --attach walk.json shot.png`. Subjects are attached files, never a
   live fetch. A comparator is the same with `jurygraph comparator add`.
4. **A brief.** `jurygraph brief --subject S --panel P --lens critical --model sonnet --spend 3 --stake a=primary,b=secondary,...`.
   Every persona of the panel needs a stake in this run. You get a run id and one deterministic, hashed brief per persona.
5. **Your agent runs the personas**: one subagent per brief, each writing one verdict JSON. The skill
   `skills/running-a-jury` is the method.
6. **Ingest.** `jurygraph verdict --run N --file v.json`. A refusal returns an error code; the persona answers again. See
   [`troubleshooting.md`](troubleshooting.md).
7. **Read.** `jurygraph matrix N --table` shows scores per stratum, side by side, never averaged (the median is the lower middle value). `jurygraph pick N` shows
   forced choices.
8. **Decide.** Each finding gets one outcome with `jurygraph decide`. `jurygraph check` exits 1 (`check_failed`, listing `unresolved`) while a finding from any batch has no decision, or was decided `fixed` and raised again by a later run (`fix_not_landed`); it lists `fixed_unverified` for fixes no later run of that lens and persona has judged.
9. **Next round.** After the fixes, brief and ingest again, then `jurygraph delta A B`.

## The verdict file
One JSON per persona, in the shape the brief shows:
```json
{ "persona": "builder-ana",
  "lines": [ { "dimension": "works", "about": "cmp:acme-pricing", "score": 2, "anchor_met": 1, "evidence": "seen",
               "citations": ["cmp:acme-pricing/pricing#btn-signup"], "blockers": ["the signup link goes nowhere"], "note": "..." } ],
  "pick": { "choice": "cmp:acme-pricing", "reason": "..." } }
```
`about` is optional: one of the subject's attachment labels or `cmp:<name>`. A verdict has one line per (dimension, `about`) and
at least one line for every dimension of the lens; scoring the same dimension twice for the same option, or leaving one out, is
refused (`duplicate_dimension`, `missing_dimension`). The table of codes is in [`troubleshooting.md`](troubleshooting.md).

## Cost knobs
Two: how many personas (`--spend N`, the first N of the panel by stake, then the lens choice) and how much each brief carries
(`--detail compact|full`, also settable as `detail` in a `jurygraph.toml` rule). Every brief reports its byte size. The engine does not estimate tokens.

## Does this change need a jury
`jurygraph should-run --diff change.diff --config jurygraph.toml` returns `none`, `low`, `high` or `human` and a spend, from
rules you write as data. A rule whose class is money, permission, deploy, secret, licence, public-claim or brand must have
quorum `human`, or the config is refused; the class is matched after trimming, lower-casing, turning spaces and underscores
into `-` and reading `license` as `licence` (so `Money`, `public_claim` and `license` count). Other spellings (`secrets`) do not.

Publishing a result outside the project: [`publishing-results.md`](publishing-results.md). Lenses: [`lenses.md`](lenses.md).

## Try it on the fixture site
The five-minute example in the README uses only files in `fixtures/`.

## What it cannot do
It checks that a citation names something the run holds; it does not check image regions or whether a persona really read
it. Colour and layout reach the jury only through screenshots.
