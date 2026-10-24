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

## Traits, as the persona's summary
After the prose, a persona may carry `traits`: at most 8 entries, trait id to value, written as the summary of what the prose already says
(`"trust.marketing_skepticism": 4`, `"choice.decision_style": "rational"`). Run `personajury trait install` once per workspace, then
`personajury trait list` for the ids; a value the trait cannot hold, or an id the store lacks, is refused. A trait is a tendency, not a type:
never write a personality label, and these ids are refused: `mbti`, `enneagram`, `disc`, `learning_style`, `generation`, `age`, `gender`,
`ethnicity`, `race`, `religion`, `disability`, `nationality`, `beauty`, `fame`, `outcast` (the juror's sensitivity to looks, fame or being an
outsider is a trait, a persona's own is not). `access.need` says what the page must do for this reader (a screen reader, captions), not who
the person is. Add `serves` (tags such as `["docs"]`) when the persona should sit in `--seats auto` juries. See `docs/traits.md`.

**Migrate a persona you already have: suggest, check, add.** Write the prose first and the traits as its summary, never the reverse.
`personajury trait suggest --persona ID [--set NAME]` writes a brief laying the persona's prose beside the candidate traits' anchors.
Answer it with one JSON object `{"persona", "traits", "evidence"}`: propose a value only where a sentence of the prose supports it, and
copy that sentence (or a phrase of it, verbatim, at least 3 words and 12 characters, from one field) as the evidence; leave out every trait the prose does not support. Then
`personajury trait suggest --check answer.json` verifies the ids, the values and each quote, and answers `persona_file`, the persona with
the traits set. It writes nothing: save that file and `persona add` it for a new version. Afterwards `personajury persona lint` warns about
near copies and a tag that cannot be told from a band; whether a trait fits the prose is yours to read, the engine cannot. For traits
that belong to a material rather than the catalog, `personajury trait generate --from material.md` and `trait add --generated-from`.

## Add it
```
personajury persona add --file personas/builder-ana.json
personajury persona list
personajury panel add fixture --query "3 builder, 2 company, 2 others"
personajury panel resolve fixture
```

## Generate from material
```
personajury persona generate --from docs/icp.md --count 7
```
This writes a **brief addressed to you**: it asks you, the agent, to write N personas from the material in the fixed JSON.
PersonaJury calls no model. Write the files yourself, then add each one with the material's hash, so the origin is recorded
and the same material regenerates the same panel:
```
personajury persona add --file personas/new-one.json --generated-from <hash from the brief>
```
The brief works in two steps. First list the roles, sectors and objections the material names or implies (who it speaks
to, who pays, who is turned away, what each pushes back on), and hand that `roles` list to the caller; mark which role uses,
which buys or approves, and which only passes by. Then write the `--count` personas, spread over those roles and over the
tags a panel needs. The count is the caller's, never the number of roles: a role may go without a persona, and one persona
may combine two roles.

Write from the material, not from habit: take the roles, sectors and objections it names. Spread the tags across the
strata you will need. Personas are composites, never identifiable real people: no real names, no scraped profiles, no customer list turned into a person (`PERSONAS.md`).

Simulated demographics are allowed, labelled and recorded (D39): a persona may have an age, a gender, a religion or a condition when the subject is read differently because of it. `persona add` answers `sensitive: [...]` with the categories its prose and traits name, the run records them, and `summary` prints "simulated demographics: … — personas are simulations, not real-world data". Check the label says what you meant; a word by chance ("public health" as a field of study) is labelled too. Give it to the persona only where it moves the decision, never as decoration.

## Your own juror, a devil's advocate, real people
- **Yourself:** enter a real human verdict with `verdict --by-human "NAME, CREDENTIAL"`; do not simulate yourself.
- **Someone you know:** describe the traits as a composite, with no name and nothing a colleague would recognise.
- **An always-negative critic:** `"tags": ["critic"]`, seated as a bystander (`1 critic=bystander`); read its blockers, not its scores.
- **A real named or famous person:** no. Write the archetype ("a contrarian tech CEO who decides in seconds and mocks jargon"); `PERSONAS.md` allows composites only.

Two cautions: a flattering self-persona biases upward, and a constant critic never scores high. Worked example: `docs/playbooks/your-own-jurors.md`.

## Fair personas in five lines
- Write from real material (tickets, interviews, notes), not imagination. *Why: invented people agree with you.*
- Give each persona its own fears and its own way of deciding. *Why: if two could swap lines, you have one juror twice.*
- Include someone you are not selling to. *Why: they find what your fans forgive.*
- Describe traits, never a named or famous person. *Why: a guessed real person is fiction with a face.*
- Do not make a persona of yourself; enter your real verdict with `verdict --by-human`. *Why: a flattering self is bias.*

## Stake is not in the persona
A persona has `stake_rules` in words as guidance, but the stake itself belongs to the audience: the panel's term
(`2 approver=secondary`) gives it, and `brief --stake <persona>=primary|secondary|bystander` overrides it for one run. The
same persona is primary on one page and a bystander on another, and scores are reported per stake.

## What the engine cannot see
It does not judge whether a persona is realistic or distinct; it only checks the shape. Read the set yourself: could two of
them swap lines? If so, rewrite one.
