---
date: 2026-10-24
valid_until: 2027-10-24
verdict: "PersonaJury, 8 of 8 forced picks (primary 4 of 4, secondary 2 of 2, bystander 2 of 2) over seven other candidates, including the working name JuryGraph, which scored 2 on 'understood' in every stratum."
status: decided
---

# The name, picked by a persona jury

The project was built under the working name JuryGraph and renamed to PersonaJury before its first release (D25). This report
is the method and the numbers behind the choice. The tool was used on itself: the same engine, briefs and refusals as any other
jury, and the same limits (see the end).

## Method
- **Candidates (8).** PersonaJury, PersonaVerdict, PersonaCritic, PersonaCourt, PersonaSim, PersonaPanel, SimJury and the
  working name, JuryGraph. One card per name with its registry facts, so that "can I own it" was judged on data and not on
  memory.
- **Panel (8 personas, three strata).** Primary (4): an agent-skill author, a first-month newcomer, a growth marketer who
  googles "AI focus group", a UX researcher who searches "synthetic users". Secondary (2): an engineering manager, a strategy
  consultant. Bystander (2): a newsletter curator and a tea-shop owner, both non-technical.
- **Lens `naming`, version 1.** Four dimensions scored 1 to 5 with anchors at 1, 3 and 5: *understood* (does the name say what it
  does), *findable* (would the words I would type lead here), *ownable* (is it free to register and free of collisions),
  *memorable*. Then a forced pick of one name, with a reason.
- **8 briefs, 0 refused.** Every verdict cited its card. Scores are reported per stratum, never averaged across strata.

## Registry facts (2026-10-24)
| name | crates.io | npm | GitHub repos by that name | .com | .ai |
|---|---|---|---|---|---|
| personajury | free | free | 0 | free | free |
| jurygraph | free | free | 0 | free | free |
| personaverdict | free | free | 0 | free | free |
| personacritic | free | free | 0 | free | free |
| personacourt | free | free | 1 | free | free |
| personapanel | free | free | 4 | taken | free |
| personasim | free | free | 9 | free | free |
| simjury | free | free | 2 | taken | free |

## Scores: median per stratum (primary / secondary / bystander)
| name | understood | findable | ownable | memorable |
|---|---|---|---|---|
| **PersonaJury** | **5 / 5 / 4** | 3 / 3 / 4 | 5 / 5 / 5 | 4 / 4 / 4 |
| PersonaCourt | 4 / 4 / 4 | 3 / 3 / 3 | 4 / 4 / 4 | 4 / 4 / 3 |
| PersonaCritic | 4 / 4 / 4 | 3 / 3 / 3 | 5 / 5 / 5 | 3 / 4 / 4 |
| PersonaPanel | 4 / 3 / 3 | 3 / 3 / 3 | 2 / 2 / 2 | 3 / 3 / 3 |
| PersonaVerdict | 3 / 3 / 3 | 3 / 3 / 3 | 5 / 5 / 5 | 3 / 3 / 3 |
| PersonaSim | 3 / 3 / 3 | 3 / 3 / 3 | 2 / 2 / 2 | 3 / 3 / 3 |
| JuryGraph | 2 / 2 / 2 | 2 / 2 / 2 | 5 / 5 / 4 | 4 / 4 / 4 |
| SimJury | 2 / 2 / 2 | 2 / 2 / 2 | 2 / 2 / 2 | 3 / 3 / 3 |

## Picks
PersonaJury took 8 of 8: primary 4 of 4, secondary 2 of 2, bystander 2 of 2. The reasons repeat one point in different words:
the name says in two words that personas judge your work, it contains the word people type ("persona"), and it is free
everywhere.

## What the numbers say
- JuryGraph was easy to own and memorable but did not explain itself: *understood* was 2 in every stratum, because "graph" does
  not say what the product does (the product is a jury; the graph is one of its pictures, `personajury graph`).
- *Findable* stayed at 3 for PersonaJury in the technical strata: "persona" alone is common enough to drown in. The name does
  not carry discovery by itself; the description, the repository topics and the first line of the README carry the category
  words (synthetic users, AI personas, AI focus group).
- Decision: PersonaJury. Costs: a rename before any release, so no compatibility shims and no migration of old stores.

## Limits
The jurors are fictional personas run on one model; the pick is a forced choice, so it shows relative preference, not demand. The
registries were checked on one day and the names can be taken tomorrow; the trademark position was not searched. The numbers
are a dated, hashed snapshot of what these personas said, not a market survey. A jury run on the naming lens can be repeated
with `personajury` and a new card set.

## The maintainer's override
On 2026-10-24 the maintainer chose JuryPersona over the jury's pick: "JuryPersona - override go with this." The record above stays as
it was run; this section adds a second run and the decision (D27 in `docs/decisions.md`).

A second jury run put JuryPersona on a card beside PersonaJury, with the same lens and the same panel. Medians per stratum
(primary / secondary / bystander):

| name | understood | findable | ownable | memorable |
|---|---|---|---|---|
| JuryPersona | 3 / 3 / 3 | 3 / 3 / 3 | 3 / 3 / 3 | 3 / 3 / 3 |
| PersonaJury | 4 / 4 / 3 | 3 / 4 / 4 | 5 / 5 / 5 | 4 / 4 / 4 |

The picks were PersonaJury 8 of 8 again. So the maintainer's choice does not rest on the jury: it overrides it, in both runs. A name
is a human class of decision, a brand, and the jury informs it without owning it. JuryPersona was kept by a person who prefers it;
the jury's numbers say it is the weaker name on every axis, most of all on *ownable* (3 against 5). The cost of the override is a
repository rename (GitHub redirects the old URL), release 0.5.1 with new asset names, and a plugin reinstall; v0.5.0 was not adopted
by anyone, so no shim and no migration were written.

## The third run, and the final name
On 2026-10-24 a third naming jury ran with 11 mixed people (primary 4, secondary 3, bystander 4), three models (Sonnet, Opus and Haiku
mixed across the jurors) and neutral cards, scoring each name 1 to 10. PersonaJury took 8 of 11 picks (primary 3 of 4, secondary 3 of 3,
bystander 2 of 4); JuryPersona took 3 (the designer, a retired teacher, a student). Medians, PersonaJury against JuryPersona:
say-and-spell 7 / 7 / 6 against 6 / 6 / 4, and feel 6 / 5 / 6 against 5 / 4 / 4 (primary / secondary / bystander).

The maintainer answered "PersonaJury (Recommended)" and the name is final (D28 in `docs/decisions.md`). The earlier sections stay as
they were run: D27 and its override are history. Nothing was public under either name, so the restore carries no shim and no migration.
