# Personas

A persona is a **composite**: an invented profile built from roles, sectors, objections and habits. It is never an
identifiable real person. The jury is a way to read a subject through several concrete points of view; it is not a
record of anyone.

## The rule
A persona in this repository, or in a set offered to it, must not be:
- **named after or modelled on a real person.** No real name of a private individual, no stage name, no "someone like our
  customer Jo at Acme" that a colleague would recognise;
- **identifiable by a combination.** An employer plus a role plus a city that singles one person out is an identity even
  without a name;
- **a scraped profile.** Nothing copied from a social network, a CV site, a forum or a directory;
- **a customer list turned into a person.** No one's customers, users, tickets or interview transcripts go into a persona
  verbatim. Aggregate them into roles and objections first, then write the composite.

A public figure is not a persona either: a verdict "by" a named real person would be a claim about that person.

If a persona turns out to match a real person, that is a bug. Report it (`SUPPORT.md`) and it is changed or removed; it is
not a feature request.

## Writing one
The fixed JSON and the stake rules are in `skills/writing-a-persona/SKILL.md`; `fixtures/personas/` has seven examples.
Make each persona distinct: if two of them could swap lines, rewrite one.

## Generated personas record their material
`jurygraph persona generate --from <material>` writes a brief that asks your agent to write the personas; the engine calls no
model. The brief carries the hash of the material. Add each file with it:
```
jurygraph persona add --file personas/new-one.json --generated-from <hash from the brief>
```
The hash records which material a persona came from, so the same material can be checked against the same panel later. It
does not make the material shareable. Only generate from material you are entitled to share. If you publish a set, say what
kind of material it came from (an ICP document, a market sketch), not the material itself if it holds anyone's data.

## What a reviewer rejects
A pull request that adds or changes personas is declined when:
- a persona matches the "must not be" list above;
- the set was generated from material the contributor cannot share, or it has no recorded `--generated-from` hash although
  it says it was generated;
- a persona carries a field outside the fixed JSON, a stake instead of `stake_rules` words, or a copied paragraph of someone
  else's text;
- two personas of the set are near-copies, so the jury measures one opinion several times;
- the set gives no account of who it is for and where it could mislead.

Shipping a set inside this repository is a maintainer decision with a `docs/decisions.md` entry (`GOVERNANCE.md`).
