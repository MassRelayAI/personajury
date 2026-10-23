# JuryGraph in five minutes

JuryGraph keeps the jury; your agent runs it. The engine never calls a model: it writes a brief per persona, your agent's
subagents answer in JSON, and the engine refuses any answer without evidence and reports scores per stratum.

```bash
# 0. a workspace (any folder) and the four shipped lenses, seeded on first write: critical, taste, compare, recall
cd my-project

# 1. personas: the fixed JSON (see skills/writing-a-persona); tags feed panel queries
jurygraph persona add --file personas/ana.json          # once per persona
jurygraph panel add devs --query "2 builder, 1 others"   # "<N> <tag>, ..."; `others` = none of the named tags
jurygraph panel resolve devs

# 2. the subject is what people see, as files: an ARIA walk, screenshots, a diff, a document. Never a live URL.
jurygraph subject add pricing --kind page --attach walk/pricing.walk.json shots/pricing.png
jurygraph comparator add rival --kind prices --attach rival/pricing.walk.json rival/pricing.png   # optional

# 3. brief: one hashed file per persona; every panel persona needs a stake; --spend N takes the first N,
#    primary first. --detail compact (attachments by path) or full (text attachments inline) sets the token weight.
jurygraph brief --subject pricing --panel devs --lens critical --spend 3 --model <your-model> \
  --stake ana=primary,ben=secondary,cleo=bystander
#    -> .jurygraph/runs/1/<persona>--critical.md

# 4. your agent: one subagent per brief, each in a private folder, each writes one verdict JSON; then
jurygraph verdict --run 1 --file verdicts/ana--critical.json      # refused without evidence, e.g. seen_without_citation

# 5. read it per stratum (never averaged across), decide each finding, check the gate
jurygraph matrix 1 --table
jurygraph decide <finding-id> fixed --by you        # or: dismissed --why "...", or: row --row ISSUE-12
jurygraph check                                      # exits 1 while a finding has no decision, or was fixed and raised again

# 6. next round after fixes: a new brief on the changed subject, then
jurygraph delta 1 2                                  # landed: every cell judged again, no stratum's blockers up, none down by > one anchor

# when to run at all: classify a diff against your jurygraph.toml (none | low | high | human, and the spend)
git diff main > /tmp/d.diff && jurygraph should-run --diff /tmp/d.diff

# once: register the MCP server (brief, verdict, matrix, delta, should_run) with your agents; doctor shows the state
jurygraph setup            # a plan; add --apply to write
jurygraph doctor
```

Evidence classes: `seen` (cites an attachment of the run), `told` (the brief said it), `recalled` (memory, labelled).
Strata: `primary` would use it, `secondary` buys or approves it, `bystander` passes by. A worked run with seven personas
is in `fixtures/run/` (`setup.sh`, the verdicts, `matrix.md`); `skills/running-a-jury` is the full loop.
