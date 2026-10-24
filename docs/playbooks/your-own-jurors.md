<!-- sources: docs/commands.md (verdict --by-human, panel terms with a stake); PERSONAS.md (composites only); skills/writing-a-persona/SKILL.md -->

# Your own juror, a devil's advocate, real people

Sometimes the person whose opinion you want is you, a colleague, a harsh critic or someone famous. Each case has a different
answer, and one of them is no.

## Yourself: enter a real verdict, do not simulate it
Write your own verdict as a person and ingest it with `--by-human`:

```bash
jurypersona verdict --run 1 --file my-verdict.json --by-human "Your Name, product owner"
```

A person's batch is labelled as human, and a blocker in it escalates `apply-check` until you record
`decide --by-human`. Do not write a persona that is "you": the persona would be your guess about you, and it flatters.

## Someone you know: describe the traits, not the person
Write a composite from the traits that matter (role, habits, objections, what they skim) with no name and no detail that a
colleague would recognise. Two such personas that could swap lines are one persona; rewrite one.

## An always-negative critic: a tag, a bystander seat, and read the blockers
Give the persona the tag `critic` (`"tags": ["critic"]`) and seat it as a bystander:

```bash
jurypersona panel add with-critic --query "3 builder=primary, 2 company=secondary, 1 critic=bystander"
```

Read the critic's **blockers**, not its scores. A critic is useful for the point it raises that the others missed; as a
score it only pulls one stratum down, which is the only place it can be seen, because strata are never averaged.

## A real named or famous person: no
A verdict "by" a named real person would be a claim about that person (`PERSONAS.md`). Write the archetype instead:

> a contrarian tech CEO who decides in seconds and mocks jargon

## Two cautions
- A flattering self-persona biases upward: it knows what you hoped and says yes.
- A constant critic never scores high: its median says something about the critic, not about your subject.
