<!-- shape follows comparegraph docs/usage.md -->
# Usage

PersonaJury runs a persona jury as an engine. It never calls a model: your agent runs the personas. This page is the loop; the
flags are in [`commands.md`](commands.md). Behaviour described here is the v0.1 contract in the plan.

## The loop
1. **Set up once.** `personajury setup --apply` registers the MCP server with the agents it finds (without `--apply` it prints
   a plan). `personajury doctor` reports the store and each registration. The store is `.personajury/store.db` under `--root`
   (default `.`).
2. **Personas and a panel.** `personajury persona add --file p.json` per persona, or `personajury persona generate --from
   material.md` to write a brief that asks your agent to write them. `personajury panel add NAME --query "3 builder, 2 company,
   2 others"` saves a query over persona tags; `personajury panel resolve NAME` shows who it resolves to.
3. **A subject.** `personajury subject add NAME --kind page --attach walk.json shot.png`. Subjects are attached files, never a
   live fetch. A comparator is the same with `personajury comparator add`.
4. **A brief.** `personajury brief --subject S --panel P --lens critical --model sonnet --spend 3 --stake a=primary,b=secondary,...`.
   Every persona of the panel needs a stake in this run. You get a run id and one deterministic, hashed brief per persona.
5. **Your agent runs the personas**: one subagent per brief, each writing one verdict JSON. The skill
   `skills/running-a-jury` is the method.
6. **Ingest.** `personajury verdict --run N --file v.json`. A refusal returns an error code; the persona answers again. See
   [`troubleshooting.md`](troubleshooting.md).
7. **Read.** `personajury matrix N --table` shows scores per stratum, side by side, never averaged (the median is the lower middle value). `personajury pick N` shows
   forced choices.
8. **Decide.** Each finding gets one outcome with `personajury decide`. `personajury check` exits 1 (`check_failed`, listing `unresolved`) while a finding from any batch has no decision, or was decided `fixed` and raised again by a later run (`fix_not_landed`); it lists `fixed_unverified` for fixes no later run of that lens and persona has judged.
9. **Next round.** After the fixes, brief and ingest again, then `personajury delta A B`.

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
Three: how many personas (`--spend N`, the first N of the panel by stake, then the lens choice), how much each brief carries
(`--detail compact|full`, also settable as `detail` in a `personajury.toml` rule) and the one word that bounds both,
`effort = "low" | "medium" | "high" | "max"` (`brief --effort`; an optional `budget_tokens` caps the estimate). Every brief
reports its byte size and an `estimate {briefs, tokens}`; `doctor` shows the tokens your agent reported over the last 7 days.

## The last step: apply-check
`personajury apply-check RUN...` answers `apply`, `escalate`, `human` or `advise`, with the reasons. Run it last and obey it; it applies
nothing itself. A jury that cannot apply (a cheap one, or one context playing every juror) can still advise.

## Does this change need a jury
`personajury should-run --diff change.diff --config personajury.toml` returns `none`, `low`, `high` or `human` and a spend, from
rules you write as data. A rule whose class is money, permission, deploy, secret, licence, public-claim or brand must have
quorum `human`, or the config is refused; the class is matched after trimming, lower-casing, turning spaces and underscores
into `-` and reading `license` as `licence` (so `Money`, `public_claim` and `license` count). Other spellings (`secrets`) do not.

Publishing a result outside the project: [`publishing-results.md`](publishing-results.md). Lenses: [`lenses.md`](lenses.md).

## What goes to a jury
Code correctness goes to tests, so a path no rule matches gets no jury: `should-run` lists it under `unmatched`, and the Action's step
summary says so in plain words ("N files matched no rule and got no jury: ...; add a rule if one of them is something people see").
Visible surfaces (pages, decks, designs, rendered docs) go to a jury that sees the rendered result, screenshots and walks, never the diff
of code. Technical-but-read surfaces (API docs, CLI output, error messages) go to builder personas through the rule's panel. Code is a
fine subject for judgement (readability, API ergonomics, trade-offs between libraries: `fixtures/lenses/tradeoffs.toml` and
[`playbooks/a-technical-decision.md`](playbooks/a-technical-decision.md)): judgement, never correctness.

## Display is free, the verdict is not
The `summary` layout is a recommendation. Any agent may restyle the JSON an envelope returns, with two rules: never pool strata, and never
write a number the data lacks. `apply-check` is different: it is a verdict, and the engine cannot enforce it by itself. It becomes binding
where it is wired:

- **The GitHub Action** (`action.yml`, [`ci.md`](ci.md)): `fail-on: human,escalate` fails the job.
- **A Claude Code hook before merge or push**, in `.claude/settings.json`:
  ```json
  { "hooks": { "PreToolUse": [ { "matcher": "Bash", "hooks": [ { "type": "command", "command": "sh .claude/hooks/jury-gate.sh" } ] } ] } }
  ```
  with `.claude/hooks/jury-gate.sh`:
  ```sh
  #!/bin/sh
  cmd="$(jq -r '.tool_input.command // ""')"
  case "$cmd" in *"git merge"*|*"git push"*) ;; *) exit 0 ;; esac
  git diff main...HEAD > /tmp/jg.diff
  personajury should-run --diff /tmp/jg.diff | jq -r '.data.unmatched | select(length > 0) | "\(length) files matched no rule and got no jury: \(join(", ")); add a rule if one of them is something people see"' >&2
  a="$(personajury apply-check --latest | jq -r '.data.answer // "advise"')"
  case "$a" in escalate|human) echo "personajury: apply-check says $a; not merging" >&2; exit 2 ;; esac
  ```
- **A git pre-merge hook** (`.git/hooks/pre-merge-commit`, made executable; use `pre-push` for pushes):
  ```sh
  #!/bin/sh
  a="$(personajury apply-check --latest | jq -r '.data.answer // "advise"')"
  [ "$a" = escalate ] || [ "$a" = human ] && { echo "personajury: apply-check says $a" >&2; exit 1; }
  exit 0
  ```
- **Your own gate**: any script that runs `personajury apply-check --latest` and reads `.data.answer`.

## Where results live
The store (`.personajury/`) is the record: append-only, and not meant for git unless CI reads it (`store:` in the Action). Reports are views of it,
written on request (`personajury summary RUN... --md summary.md` or `--html summary.html`) or automatically into `report_dir`
(`report_dir = "docs/juries"` in `personajury.toml`; `summary --save` writes `<report_dir>/<date>-<subject>.md`, and is refused with
`no_report_dir` when it is unset). With no `report_dir` nothing is written.

## Try it on the fixture site
The five-minute example in the README uses only files in `fixtures/`.

## What it cannot do
It checks that a citation names something the run holds; it does not check image regions or whether a persona really read
it. Colour and layout reach the jury only through screenshots.

## Where it runs
- **Claude Code, Codex, Cursor, Gemini CLI and the rest:** `personajury setup --apply`, or the plugin of your harness. Where the
  harness has subagents, one per brief is a strong jury.
- **Claude Desktop:** drop the `.mcpb` bundle for your platform (from the release page) on the app and pick the project folder
  that holds `.personajury/`. Desktop has no subagents, so one context plays every juror: brief with `--independence sequential`.
  Those runs advise and never apply. This route is unverified against Claude Desktop until a maintainer installs one by hand.
  Build a bundle yourself with `scripts/build-mcpb.sh <platform> <binary>`.
- **The chat route (claude.ai, ChatGPT, any chat window):** no server, files only. Run `personajury brief` in a terminal, paste one
  brief file per chat (one chat per brief keeps the jurors independent), have the chat answer with the verdict JSON, save it and run
  `personajury verdict --run N --file v.json`. Many briefs in one chat is `--independence sequential`.
- **CI and SDKs:** the JSON envelope over the CLI and MCP is the API; there is no stable Rust API and no runner in the tree. A
  composite GitHub Action and a Claude Agent SDK recipe are planned (`roadmap.md`).

## Personas from a use-case table
A table of jobs and triggers answers what the persona brief's first step asks for (roles, sectors, objections). The example
`fixtures/material/usecases.csv` has twelve invented rows for Lantern Notes:
```
personajury persona generate --from fixtures/material/usecases.csv --count 5
```
The engine hashes the file it is given and filters nothing: cut the rows you want into a file first. A sensitivity column
(`data_sensitivity`, `clearance_note`) is material for your personas and for your own `personajury.toml`, where the paths of sensitive
surfaces belong under the human classes; the engine never learns a sector.

## The dial: what you decide, what needs a reason, what a person decides
The default is the fast one. You move freely inside the rule and the effort; to leave the rule you give a reason that is
recorded; four things stay with a person.

| choice | you decide (no reason) | needs a recorded reason (`--reason`, `--by`) | a person only |
|---|---|---|---|
| persona mix | the panel the rule names; the seeded draw picks who sits; `--subject` and `--comparator` are yours | `--panel`, `--query` or `--stake` that differ from the rule | |
| persona count | the rule's seats, trimmed by effort | `--spend` above the rule; 13 to 1000 seats with `--allow-large` | above 1000 seats: nobody |
| lens choice | any of the rule's lenses; the first under effort `low` | `--lens` that the rule does not name | |
| dimensions and anchors | `lens generate` drafts them, you accept with `lens add` | editing a shipped lens's anchors (a new version) | an anchor set claimed as a named standard carries the expert's name in `standard` |
| detail, model, seed, independence | `--model` (also per panel term), `--wave`, `--independence` | `--detail` and `--draw` under a rule | |
| effort and tokens | effort at or below the toml's; `budget_tokens` trims | `--effort` above the toml's | |
| quorum | you never set it; the rule's paths do; `--rule` names the rule (recorded on the run with its quorum) | a bigger jury than the rule asks; a rule below the sources' quorum (then `apply-check` advises, `rule_below_sources`) | lowering a quorum is a toml edit, a person's commit; a human class under a persona quorum is refused whoever edits |
| findings | the foreman decides `fixed`, `dismissed` or `row` with `--by foreman` | a `dismissed` needs `--why` | a finding raised by a human batch is decided by a person (`decide --by-human`; a plain --by is refused, `human_decision_required`) |
| apply | `apply-check` answers `apply` only when every condition is proven: every briefed seat answered, a known quorum, the run under its rule and not overridden down, effort within the quorum, independent, current, the delta computed and landed, no primary blocker or split in any run, `check` clean for the subject, no open human blocker | | `human` classes: the jury advises, a person applies |
| human verdicts | | | only a person enters one (`--by-human`) |

**The four hard stops:** the human classes; seats above 1000; a verdict edited to pass; a human-raised finding, or a human verdict
entered by an agent. Everything else is yours, with a reason where the rule said otherwise, and `apply-check` turns a weak, cheap,
incomplete, stale or overridden-down run into `advise` instead of refusing it: anything unknown, missing or lowered advises, never
applies. Raising seats, lenses, effort or detail above the rule is fine. A jury that cannot apply can still advise, which is how a
small subscription still gets a jury.
