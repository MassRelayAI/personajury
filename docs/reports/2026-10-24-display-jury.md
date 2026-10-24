---
title: "How to show a jury's answer: sentences first, then numbers, no bars"
date: 2026-10-24
valid_until: 2027-04-24
verdict: "Bars lost: 0 of 8 picks, and the lowest medians on every question. Numbers and sentences tied 4 to 4, split by reader: technical checkers picked numbers, non-technical readers picked sentences. Decision: sentences first, a numbers table for checking, a next line; no bars (D24)."
status: decided
decisions: D24 (docs/decisions.md)
---

# How to show a jury's answer

## The question

`jurypersona summary` has to put a jury's answer in front of a person, in a chat or a terminal. Four layouts were candidates. The cards showed
the same result (the name jury, [the name, picked by a persona jury](2026-10-24-the-name-picked-by-a-persona-jury.md)) in four ways:

| card | what it shows |
|---|---|
| A, bars | blocks per stratum, one block per point out of 5 |
| B, numbers | one winning line, then a grid of medians `users · buyers · passers-by` |
| C, sentences | one winning sentence, then four bullets in plain words |
| D, ranked | a ranked table with picks, clarity and the other columns |

## The jury

Eight composite personas judged them with a purpose-built lens `display` (three dimensions, scored 1 to 5 with anchors in words, plus a forced pick):

- **answer**: I get the answer at a glance.
- **trust**: I can trust and check it.
- **act**: I know what to do next.

Primary: an agent-skill author, a growth marketer, a weekend newcomer, a UX researcher. Secondary: an engineering manager, a strategy
consultant. Bystander: a newsletter curator, a tea-shop owner. Each saw the four cards and scored them. The medians below are per stratum,
primary · secondary · bystander, never averaged.

## What came out

| card | answer | trust | act |
|---|---|---|---|
| A, bars | 3·3·2 | 2·3·2 | 2·2·1 |
| B, numbers | 4·5·4 | 3·3·3 | 5·5·4 |
| C, sentences | 5·4·5 | 3·2·4 | 5·5·5 |
| D, ranked | 4·4·4 | 3·3·3 | 5·5·4 |

Forced picks: bars 0 of 8, ranked 0 of 8, **numbers 4 of 8 and sentences 4 of 8**.

## The split by reader

| reader | picked | why (paraphrased from their reasons) |
|---|---|---|
| primary, technical (agent-skill author, UX researcher) | numbers | the grid of per-group numbers is the closest to something they could diff and check; it keeps each group separate |
| secondary (engineering manager, strategy consultant) | numbers, 2 of 2 | the verdict up top and every group's scores to check, to defend it to someone else |
| primary, non-technical (growth marketer, newcomer) | sentences | the only one they could read aloud or in five seconds without decoding |
| bystander (newsletter curator, tea-shop owner) | sentences, 2 of 2 | reads like a friend telling the result in plain words |

Sentences scored best on `answer` for the primary and bystander readers (numbers did for the secondary) and on `act`; numbers scored best on `trust` among the technical readers (4 from the agent-skill author and the UX
researcher against 3 for sentences), while the bystanders trusted sentences more (median 4 against 3). No layout won every reader.

## Decision

Sentences first, a numbers table beneath it headed "for checking", then a `Next:` line. No bars anywhere, not even for picks. Picks are written
as numbers ("users 4 of 4"), changes as "15 → 6". This serves both halves of the split: the person who wants the answer reads two to five
sentences; the person who wants to check reads the table, with the strata side by side and no pooled number. Encoded in D24 and in the golden
files `fixtures/run/summary.txt` and `fixtures/run/summary.md`.

Display is free, the verdict is not: the layout is a recommendation and any agent may restyle the JSON, with two rules (never pool strata,
never write a number the data lacks). `apply-check` is binding only where it is wired.

## Limits

Eight composite personas, one model, one subject (a naming result), one round. The 4 to 4 tie is a reader split, not a measurement of a
population. It would change if a real reader group, run through `calibrate`, ranked the layouts differently.
