# PersonaJury in five minutes

PersonaJury keeps the jury; your agent runs it. The engine never calls a model: it writes a brief per persona, your agent's
subagents answer in JSON, and the engine refuses any answer without evidence and reports scores per stratum.

Before you start: install the binary ([README, Install](../README.md#install)). The words (persona, stake, lens, brief,
verdict) are explained in [README, Words used here](../README.md#words-used-here). To see a finished jury before running your
own, replay the recorded one ([README, Try it](../README.md#try-it)). Every command prints one JSON envelope; add `--pretty`
to read it, and `personajury summary RUN --html jury.html` writes a page you open in a browser.

```bash
# 0. a workspace (any folder) and the four shipped lenses, seeded on first write: critical, taste, compare, recall
cd my-project

# 1. personas: the fixed JSON (see skills/writing-a-persona); tags feed panel queries
personajury persona add --file personas/ana.json          # once per persona
personajury panel add devs --query "2 builder, 1 others"   # "<N> <tag>, ..."; `others` = none of the named tags
personajury panel resolve devs

# 2. the subject is what people see, as files: an ARIA walk, screenshots, a diff, a document. Never a live URL.
personajury subject add pricing --kind page --attach walk/pricing.walk.json shots/pricing.png
personajury comparator add rival --kind prices --attach rival/pricing.walk.json rival/pricing.png   # optional

# 3. brief: one hashed file per persona; every panel persona needs a stake; --spend N takes the first N,
#    primary first. --detail compact (attachments by path) or full (text attachments inline) sets the token weight.
personajury brief --subject pricing --panel devs --lens critical --spend 3 --model <your-model> \
  --stake ana=primary,ben=secondary,cleo=bystander
#    -> .personajury/runs/1/<persona>--critical.md

# 4. your agent: one subagent per brief, each in a private folder, each writes one verdict JSON; then
personajury verdict --run 1 --file verdicts/ana--critical.json      # refused without evidence, e.g. seen_without_citation

# 5. read it per stratum (never averaged across), decide each finding, check the gate
personajury matrix 1 --table
personajury decide <finding-id> fixed --by you        # or: dismissed --why "...", or: row --row ISSUE-12
personajury check                                      # exits 1 while a finding has no decision, or was fixed and raised again

# 6. next round after fixes: a new brief on the changed subject, then
personajury delta 1 2                                  # landed: every cell judged again, no stratum's blockers up, none down by > one anchor

# when to run at all: classify a diff against your personajury.toml (none | low | high | human, and the spend)
git diff main > /tmp/d.diff && personajury should-run --diff /tmp/d.diff

# once: register the MCP server (brief, verdict, matrix, delta, should_run) with your agents; doctor shows the state
personajury setup            # a plan; add --apply to write
personajury doctor
```

Evidence classes: `seen` (cites an attachment of the run), `told` (the brief said it), `recalled` (memory, labelled).
Strata: `primary` would use it, `secondary` buys or approves it, `bystander` passes by. A worked run with seven personas
is in `fixtures/run/` (`setup.sh`, the verdicts, `matrix.md`); `skills/running-a-jury` is the full loop.
