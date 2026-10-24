---
name: writing-a-persona
description: Use when a jury needs personas - to write one by hand in the fixed JSON, to generate a panel from material (an ICP document, a customer list) with persona generate, and to tag them for panel queries.
---
<!-- adapted from comparegraph skills/writing-a-claim/SKILL.md (style) -->
# Writing a persona

A persona is a concrete person with a reason to be on this page, not a demographic. Make each one distinct: a jury of
seven near-copies measures one opinion seven times.

## The fixed JSON
Exactly these keys; any other key is refused.
```json
{ "id": "builder-ana", "name": "Ana Costa", "tags": ["builder"],
  "role": "who they are, in one line",
  "context": "what they arrive with: how they found it, the time and money they have",
  "daily_tasks": "what their day is made of",
  "fears": "what would make them leave or regret",
  "never_reads": "what they skip, so the jury does not credit them for reading it",
  "decides_by": "how they decide, in the order they look",
  "stake_rules": "in words: when they are primary, secondary or bystander for a subject" }
```
`id` is kebab-case and stable (`builder-ana`). `tags` are matched literally, lower-cased, by panel queries: `panel add
fixture --query "3 builder, 2 company, 2 others"`; `others` means none of the query's other tags, and the first term wins a
persona. Use one tag form (`builder`, not `builders`).

A panel term may carry the stake its seats get: `panel add docs-public --query "3 builder=primary, 1 newcomer=primary, 2
approver=secondary, 1 others=bystander"`. Keep the pool bigger than the seats where a stratum should vary between subjects
(six `others` for one bystander seat): the engine draws the seats from the subject's name, and the same subject keeps them
between rounds. A panel is 12 seats or fewer unless you pass `--allow-large`.

## Add it
```
jurypersona persona add --file personas/builder-ana.json
jurypersona persona list
jurypersona panel add fixture --query "3 builder, 2 company, 2 others"
jurypersona panel resolve fixture
```

## Generate from material
```
jurypersona persona generate --from docs/icp.md --count 7
```
This writes a **brief addressed to you**: it asks you, the agent, to write N personas from the material in the fixed JSON.
JuryPersona calls no model. Write the files yourself, then add each one with the material's hash, so the origin is recorded
and the same material regenerates the same panel:
```
jurypersona persona add --file personas/new-one.json --generated-from <hash from the brief>
```
The brief works in two steps. First list the roles, sectors and objections the material names or implies (who it speaks
to, who pays, who is turned away, what each pushes back on), and hand that `roles` list to the caller; mark which role uses,
which buys or approves, and which only passes by. Then write the `--count` personas, spread over those roles and over the
tags a panel needs. The count is the caller's, never the number of roles: a role may go without a persona, and one persona
may combine two roles.

Write from the material, not from habit: take the roles, sectors and objections it names. Spread the tags across the
strata you will need. Personas are composites, never identifiable real people: no real names, no scraped profiles, no customer list turned into a person (`PERSONAS.md`).

## Your own juror, a devil's advocate, real people
- **Yourself:** enter a real human verdict with `verdict --by-human "NAME, CREDENTIAL"`; do not simulate yourself.
- **Someone you know:** describe the traits as a composite, with no name and nothing a colleague would recognise.
- **An always-negative critic:** `"tags": ["critic"]`, seated as a bystander (`1 critic=bystander`); read its blockers, not its scores.
- **A real named or famous person:** no. Write the archetype ("a contrarian tech CEO who decides in seconds and mocks jargon"); `PERSONAS.md` allows composites only.

Two cautions: a flattering self-persona biases upward, and a constant critic never scores high. Worked example: `docs/playbooks/your-own-jurors.md`.

## Stake is not in the persona
A persona has `stake_rules` in words as guidance, but the stake itself belongs to the audience: the panel's term
(`2 approver=secondary`) gives it, and `brief --stake <persona>=primary|secondary|bystander` overrides it for one run. The
same persona is primary on one page and a bystander on another, and scores are reported per stake.

## What the engine cannot see
It does not judge whether a persona is realistic or distinct; it only checks the shape. Read the set yourself: could two of
them swap lines? If so, rewrite one.
