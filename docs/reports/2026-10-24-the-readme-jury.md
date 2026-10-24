---
title: "JuryPersona's own README, judged by a persona jury, fixed and judged again"
date: 2026-10-24
valid_until: 2027-04-24
verdict: "Ten composite personas (4 primary, 3 secondary, 3 bystander, one a deliberate critic) judged the README and quickstart under four lenses, against two other READMEs. Round 1 raised 32 findings: 27 fixed, 4 to the backlog, 1 dismissed. In round 2, critical blockers fell from 22 to 9 and the delta landed (primary 8 → 3, secondary 10 → 3, bystander 4 → 3). Docs blockers fell from 5 to 3 (landed), and taste stayed at 0 (landed). Compare did not land: blockers rose from 5 to 8, 6 of them one juror marking each section screenshot 'shows no price'. Forced picks for this README went from 4 of 10 to 7 of 10. Of the 20 round-2 findings, 5 were fixed, 9 went to one backlog row (a route with no terminal and no coding agent), and 6 were dismissed."
status: decided
---

# The README jury

## Setup
- **Subject:** `README.md` and `docs/quickstart.md`, plus the README rendered as GitHub shows it and cut into page images
  (`readme-part-*.png`). The jurors read the rendered page as well as the markdown.
- **Comparators:** CompareGraph's README and code-review-graph's README, each with a render of its first screen.
- **Lenses:**
  - `critical`;
  - `docs` (find-it, run-it, trust-it, next);
  - `taste-readme` (look, type, voice, no-slop), with the standard being the slop signals in
    [the marketing-site playbook](../playbooks/marketing-site.md#slop-signals);
  - `compare`, with a forced pick.
- **Panel:** 10 composite personas.
  - Primary: an agent-skill author, a first-month newcomer, a growth marketer and a UX researcher.
  - Secondary: an engineering manager, a security reviewer and a strategy consultant.
  - Bystander: a newsletter curator, a small shop owner, and a contrarian critic who hunts for overclaims.
- **Spend:** 40 briefs per round, at effort `high`. A reason was recorded because the panel has more than 7 seats.

## Round 1 → round 2
Medians per stratum (primary · secondary · bystander), on a 1–5 scale, for the README itself:

| lens: dimension | round 1 | round 2 |
|---|---|---|
| critical: works | 2·2·3 | 3·4·4 |
| critical: blocks (5 = nothing stops me) | 1·2·3 | 2·3·3 |
| critical: missing | 2·2·3 | 3·3·3 |
| docs: find-it | 3·4·3 | 3·4·4 |
| docs: run-it | 2·3·2 | 3·3·4 |
| docs: trust-it | 3·3·4 | 4·4·4 |
| docs: next | 3·3·3 | 4·4·4 |
| taste: look | 3·4·3 | 4·4·3 |
| taste: type | 3·3·3 | 3·3·3 |
| taste: voice | 3·4·4 | 4·4·4 |
| taste: no-slop | 4·4·4 | 4·4·4 |
| compare: clarity | 2·3·3 | 3·4·3 |

| delta | blockers | landed |
|---|---|---|
| critical | 22 → 9 | yes |
| docs | 5 → 3 | yes |
| taste | 0 → 0 | yes |
| compare | 5 → 8 | **no** |

**The forced pick** (which README would you act on first):

| round | primary | secondary | bystander |
|---|---|---|---|
| 1 | this README 2, CompareGraph 2 | code-review-graph 2, CompareGraph 1 | this README 2, code-review-graph 1 |
| 2 | this README 3, CompareGraph 1 | this README 3 | split 1 / 1 / 1, one for this README |

## What changed between the rounds
Round 1's main finding was that the first command piped an install script into a shell, and that script pointed at
release binaries that did not exist yet. Three things fixed it:
- **The v0.5.0 release** (binaries for five platforms, `SHA256SUMS`, an attestation).
- **A new first screen** with three parts:
  - what it does, in one sentence;
  - what you need (a coding agent; no key for JuryPersona);
  - three numbered steps to try it.
- **A replay of the recorded fixture jury** that needs no agent.

Round 2 added one line beside the install command saying what it does, and linked the uninstall section.

## What did not land, and why
- **Compare.** One juror marked each section screenshot as "shows no price", because the engine offers every
  attachment as an option to compare. Those 6 findings were dismissed: the first screen states that the tool is free
  (Apache-2.0). The real issue is the open question of what counts as an option under `compare`, and this run is
  evidence for answering it: the subject as a whole, plus each comparator.
- **People without a terminal or a coding agent.** The marketer, the consultant and the shop owner cannot run it today.
  That is true, and the README says so. It is one backlog row: a route with no terminal, such as the Claude Desktop
  bundle or a hosted read-only demo of a recorded jury.

## Caveats
Ten composites are not a survey; they show where to look. Scores are relative to the two comparators in the room. The
critic sat as a bystander on purpose: its blockers were read, and its scores were not allowed to move the users'
stratum. They cannot, since strata are never averaged.
