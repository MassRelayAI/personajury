---
title: "How far can a jury go: any sector, any agent, and how much of the human it can replace"
date: 2026-10-24
valid_until: 2027-04-24
verdict: "The structure is not about code. A run is files plus personas plus a lens, all of them data, so it covers any sector. Three sector lenses ship as examples, not as the way to cover sectors; the general answer is a `lens generate` brief (v0.4), the same two-step method as `persona generate`. The engine runs wherever an agent can call a local MCP server or a CLI; a chat without subagents can run a jury, but only sequentially and with weaker independence. A jury can take the human out of most loops, but not all of them: a change is applied automatically under the rule table below, and a person stays on the human classes, on escalations and on calibration. Building toward that: the split rule (open question 3), juror-model diversity, and a calibration run against real people."
status: open
---

# How far can a jury go

## 1. Is it only for code?
No. The engine knows three things, and all three are data:
- **a subject** as files;
- **personas** with stakes;
- **a lens** with anchors written in words.

It has never parsed code to judge it. The fixture is a product site, and the two real runs so far were a README
([CompareGraph's docs](https://github.com/massrelay-ai/comparegraph/blob/main/docs/reports/2026-10-24-readme-jury.md))
and a logo ([2026-10-24-logo-jury.md](2026-10-24-logo-jury.md)).

**Citations.**
- Images and slides are cited by label.
- Text, notes and CSV rows are cited by `#L<n>`.
- A spreadsheet exported to CSV can therefore be cited row by row. A cell locator such as `Sheet!B12` does not exist
  yet.

## 2. Are sector lenses too specific? Do we need them?
We do not need a lens per sector in the product. A lens is a short TOML file, and the people who know a sector are the
ones who can write its anchors.
- **What ships:** three examples as fixture data, loaded with `lens add` and not seeded: a consulting deck, an investor
  pitch, and a financial model judged for readability. They show the shape: four dimensions with anchors in words.
- **The general answer:** `lens generate --from <material>` (v0.4). Like `persona generate`, the engine writes a brief.
  The agent drafts a lens from the material, for example a style guide, a rubric, or a partner's review notes. A person
  or the agent accepts it with `lens add`, which validates it. The engine still calls no model.
- **The cost of being wrong:**
  - Too many shipped lenses become a catalogue nobody maintains.
  - Too few, and every user writes anchors from scratch.
  - Three examples plus a generator sits between the two.

## 3. Does it work in Codex, Claude Code, Claude Desktop, a chat?
| where | how | independence of jurors |
|---|---|---|
| Claude Code, Codex, Cursor, Gemini CLI | the plugin / MCP server (`jurygraph mcp`) plus the skill; one subagent per brief | strong: each juror runs in its own context, in a private folder |
| Claude Desktop (local) | the same stdio MCP server; a `.mcpb` bundle would make installing one click (not built) | depends on whether the client runs subagents |
| a chat with no subagents and no local tools | the model plays each persona in turn from the briefs | **weak**: one context sees every juror, so the scores contaminate each other; label such a run as such |
| a hosted chat connector (remote MCP) | would need a hosted store, which breaks "local, no network" | not planned |

The rule that makes a jury worth anything is independence: one juror, one context, no view of the others' scores.
Wherever that holds, the jury holds.

## 4. Can it answer instead of a human?
For most changes, yes. JuryGraph's own two real runs went from brief to `delta landed` with no human decision. Every
finding was decided by the foreman with `--by foreman`. The question is not whether it can, but where it must not, and
how we would know it went wrong.

**Applied automatically (no person):**
- `quorum` is `none`, `low` or `high`;
- no blocker in the primary stratum;
- no split between strata (the split rule, open question 3, must be decided and built first);
- `check` passes and `delta` lands;
- every finding is decided by the foreman, with `--by` naming it.

**Escalated one level:**
- any primary-stratum blocker;
- a split;
- a delta that does not land after two rounds.

**Always a person:** the human classes. These are money, permission, deploy, secret, licence, public claim and brand.
The jury may advise. No configuration can lower this; the engine refuses it.

**What makes it robust rather than hopeful:**
1. **Independence:** private folders, one context per juror. The engine cannot see a juror's folder, so the skill and
   the agent enforce this.
2. **Different models for jurors.** Seven Sonnet jurors share one model's blind spots. The run already records the model
   per brief. Mixing models per stratum turns correlated errors into disagreement you can see. v0.4: `brief --model`
   per term.
3. **Calibration against people.** Once a quarter, the same subject goes to a few real people under the same lens. The
   held-out detector (open question 1) says how close the jury's ranks must come. If the jury drifts, its applying
   rights drop to "advise only" until it is recalibrated.
4. **Evidence or refusal.** Every `seen` line cites a file, and every decision is recorded and append-only, so an
   automatic outcome can be audited after the fact.
5. **Personas from real material:** support tickets, interviews, sales notes, through `persona generate`. They should
   not come from the model's imagination.

**What it cannot do:** know your market better than the material you give it, or replace the first real user who says
"I would not pay for this". A jury removes the human from the routine loop and keeps them where judgement and
accountability live.

## 5. The v0.4 work this implies
- the split rule and automatic escalation;
- `lens generate`;
- `brief --model` per term;
- a calibration run (`jurygraph calibrate`: human scores entered as verdicts by a person, compared with the jury per
  stratum);
- a cell locator for spreadsheets;
- an `.mcpb` bundle.

Not built; listed in `docs/roadmap.md`.
