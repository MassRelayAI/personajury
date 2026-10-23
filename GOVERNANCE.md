<!-- structure follows the governance files of goose, A2A, keycloak, openllmetry and openlineage (Apache-2.0); text is our own -->
# Governance

JuryGraph is a small project with one maintainer. This file says who decides, how a decision is recorded, what only a
recorded decision may change, and how that could widen later. It describes what is true now; it promises no committee.

## Roles
- **Contributor**: anyone who opens an issue, sends a pull request, proposes a lens or a persona set, or answers a question.
- **Maintainer**: has merge rights and the last word. Today that is one person, Nicholas Andrea Sujitno
  (support@massrelay.ai), who also owns the `massrelay-ai` organisation that hosts the repository. See `MAINTAINERS.md`.

There are no other maintainers yet, so there is no vote, no steering committee and no quorum of people. A decision is the
maintainer's, made in public and written down.

## How decisions are made and recorded
- Small changes (a fix, a doc, a test) are decided in the pull request.
- A choice the design leaves open, a new dependency, a change to `Cargo.lock` or `store/schema.sql`, or anything a later
  reader would ask "why" about, becomes a numbered entry in `docs/decisions.md` (D1, D2, ...). An entry names who decided,
  the alternative, and for a provisional choice what would change it.
- A measurement or verdict ends in a dated file in `docs/reports/` before anyone claims it (see `docs/reports/README.md`).
- Disagreement goes in the issue or pull request. If the maintainer overrules you, the reason is written there; you may
  fork, which the licence allows.

## What only a decision may change: the law of the tool
These are the commitments users rely on (the design in `docs/design.md`, report sections 2, 5 and 10):
- the three evidence classes (`seen`, `told`, `recalled`) and what each requires;
- the strata (`primary`, `secondary`, `bystander`) and the rule that nothing averages across them;
- the refusals (the table in `docs/plans/2026-10-23-v0-1-the-engine.md`, each with its error code);
- the human classes (money, permission, deploy, secret, licence, public-claim, brand), which no persona quorum may decide.
  `HUMAN_CLASSES` in the engine is law: a `jurygraph.toml` that puts one under a persona quorum is refused
  (`human_class_under_persona_quorum`), the list lives in the engine because a toml must not be able to weaken it, a change
  to the list or to that refusal needs a numbered decision, and a pull request that weakens it is closed;
- the rule that the engine never calls a model and never touches the network.

A change to any of these needs **both** a numbered entry in `docs/decisions.md` **and** a test that was seen failing before
the change. A pull request that edits them without both is closed with a pointer to this paragraph.

## Model and provider neutrality
- The engine never calls a model, so the repository holds no API key and has no setting for one.
- No provider is recommended. Docs and skills may say which model a measurement used; they do not say which to buy.
- `--model` on `brief` is the agent's declaration of what it will run the personas on. The engine records it and cannot
  verify it; read it as unverified everywhere it appears.
- No claim of the form "model X scores Y" appears in the README or the docs without a checked-in run and its hashes behind
  it, in a dated report (`docs/reports/README.md`, `docs/publishing-results.md`).
- A pull request that adds a provider-specific code path, key or default is out of bounds (`AGENTS.md`).

## Proposing a lens or a persona set
- **A lens** is data: a TOML file with dimensions and anchors in words at 1, 3 and 5 (see `lenses/`). Open an issue
  (feature request) with the file, the question the lens asks that the four shipped ones do not, and one run that shows it
  separates good from bad. A lens that only renames a shipped dimension is declined. Loading your own is always possible
  without us: `jurygraph lens add --file my-lens.toml`.
- **A persona set** is data: JSON personas in the fixed shape (see `skills/writing-a-persona`), plus the material they were
  written from. Say who they are for, what stake rules they carry and where the set could mislead. Personas are composites and must not copy a
  real person's identity (`PERSONAS.md`); a lens follows `docs/lenses.md`. Shipping a set in this repository is a maintainer decision and gets a `docs/decisions.md` entry.

## Becoming a maintainer
There is no ladder yet, because there is nobody to climb it with. The honest path: send several pull requests that are
merged without rework, review other people's changes, and show you hold the invariants in `AGENTS.md`. When that is true the
maintainer may invite you; the invitation and the reason are recorded as a decision, `MAINTAINERS.md` is updated in the same
pull request, and the new maintainer is added to the `@massrelay-ai/maintainers` team. A maintainer who is inactive for a
year, or who asks, steps down by the same route. If the sole maintainer leaves the project and nobody has been invited, the
repository stays open under Apache-2.0 and can be forked by anyone.

## Conduct
Everyone taking part follows `CODE_OF_CONDUCT.md`. Reports go to support@massrelay.ai. Conduct decisions are the
maintainer's; when the report concerns the maintainer, say so in the message and ask for it to be handled outside the
repository.

## Changing this file
By pull request, with a `docs/decisions.md` entry saying why. It will be revisited when a second maintainer joins.
