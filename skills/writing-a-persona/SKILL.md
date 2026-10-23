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

## Add it
```
jurygraph persona add --file personas/builder-ana.json
jurygraph persona list
jurygraph panel add fixture --query "3 builder, 2 company, 2 others"
jurygraph panel resolve fixture
```

## Generate from material
```
jurygraph persona generate --from docs/icp.md --count 7
```
This writes a **brief addressed to you**: it asks you, the agent, to write N personas from the material in the fixed JSON.
JuryGraph calls no model. Write the files yourself, then add each one with the material's hash, so the origin is recorded
and the same material regenerates the same panel:
```
jurygraph persona add --file personas/new-one.json --generated-from <hash from the brief>
```
Write from the material, not from habit: take the roles, sectors and objections it names. Spread the tags across the
strata you will need. Personas are composites, never identifiable real people: no real names, no scraped profiles, no customer list turned into a person (`PERSONAS.md`).

## Stake is not in the persona
A persona has `stake_rules` in words as guidance, but the stake itself is declared per run, for every panel persona, with
`brief --stake <persona>=primary|secondary|bystander`. The same persona is primary on one page and a bystander on
another, and scores are reported per stake.

## What the engine cannot see
It does not judge whether a persona is realistic or distinct; it only checks the shape. Read the set yourself: could two of
them swap lines? If so, rewrite one.
