---
title: "Fable's read of the JuryGraph v0.1 plan — law check, the three open questions, the detail knob, jury governance"
date: 2026-10-23
valid_until: 2026-11-23
verdict: "The plan keeps the law; six seams weaken it and each has a one-line fix (uncited told lines, a seen compare line with no comparator, one score per subject so 'scores all' cannot be said, delta blind to the lens version, a bottom score with no blocker that needs no decision, an even-count median that is a mean). The detector targets the maintainer's hand-scored jury table (six criteria, 1–10), not the hostname pick; a split is two strata whose anchor bands do not overlap; decide stays and 'the same point' gains the dimension; the detail knob stays compact|full with a per-rule default."
status: open
read: "report §1–§10; plan 2026-10-23-v0-1-the-engine.md; decisions D1–D5; domain/model.rs, domain/refuse.rs, adapters/cli.rs; lenses/{critical,taste,compare,recall}.toml; the maintainer's private jury table and hostname-pick notes"
---

# Fable's read of the v0.1 plan

A shape review, read-only. Line references are to the plan (`plan:N`) and the report (`§N`).

## A. Law check — §2, §4, §5, §10 against the plan, the types and D5

What holds, checked seam by seam: the seven subject kinds (§2.1 ↔ `model.rs:63-71`); no live fetch (§2.2, §4 ↔ plan:15, R16); the three evidence classes and R1/R3 (§2.2 ↔ `refuse.rs:6-11`); a stake per persona per run, `--stake` required for every panel persona (§2.3 ↔ `cli.rs:62-64`, R8); `Matrix` with no field that spans strata and `AcrossStrata` refused (§2.3, §5 ↔ `model.rs:451-483`, R6); four lenses as data with anchors in words at 1/3/5, `recall` taking `recalled` only (§2.4 ↔ `lenses/*.toml`, R14); no model call, no key, no HTTP crate (§2.5 ↔ plan:15, R16); the run pinned to subject hash, panel hash, lens version, model name (§2.6 ↔ `RunRow`); every §4 table present and verdicts/picks/decisions append-only by trigger (plan:81-86, R17); every §5 refusal mapped (R1, R3–R10); §10's hook as `should-run` with the fixed human classes (`model.rs:515` ↔ the §10 table) and R11–R13, R15. The persona file matches §4's fixed JSON (`model.rs:140-152`).

Where the law is weakened, each with the smallest correction:

| # | rule | where it weakens | smallest correction |
|---|---|---|---|
| A1 | §2.2 "Every verdict line cites what the persona saw" | Only `seen` lines must cite (R1, plan:97). A `told` line has no citation grammar at all (plan:136 names attachments and `cmp:` only), so a persona can mark any claim `told` and cite nothing — the R1 dodge. | Add the locator `brief#<section>` (sections the deterministic brief already emits: `persona`, `stake`, `lens`, `standard`, `options`), resolved against the run's `BriefRef.hash`; a `told` line needs ≥1 `brief#` citation. One grammar branch, one test. |
| A2 | §2.2 "a comparison with no comparator snapshot cannot be `seen`" | R3 fires only when a line *names* a comparator (plan:137 "comparator named and not in run"). A `compare` line with `comparator: null`, `seen`, citing the subject alone passes — a comparison that compares with nothing. | Under a lens with `asks_pick` (i.e. `compare`), a `seen` line must name a comparator the run holds or carry ≥1 `cmp:` citation; else `comparator_not_snapshotted`. |
| A3 | §3 "pick or score … three logo files; the persona picks one and scores all" | A verdict holds one score per dimension per subject (`VerdictLine`, `model.rs:302-317`); `ScoreRow` (`model.rs:440-449`) carries no `comparator`, so two `compare` lines on `clarity` about two comparators collapse into one cell in `matrix` and `delta`. "Scores all" cannot be said for three logos or for three comparators. | Rename `VerdictLine.comparator` → `about: Option<String>` taking any pick option (an attachment label or `cmp:<name>`; same column); add `about` to `ScoreRow`; cell key = (lens, dimension, about). No new table. |
| A4 | §2.6 "pinned to … the lens version" | `delta::show` refuses a different subject *name* or lens *name* (plan:231), not a different lens *version*. Anchors rewritten between runs make the delta meaningless and it is still reported as a delta. | `delta_mismatch` also on `lens_version`; a changed persona version is reported per persona (`persona_version_changed`), not refused. |
| A5 | §10(1) "each finding becomes exactly one of — a fix, a row, or dismissed; none … blocks the lane finish" | D5 makes a finding = one blocker (plan:89). A line at the bottom anchor (`works: 1`) with a note and no blocker produces no finding, so it needs no decision and R13 never sees it. | Refuse a line with `score == scale.min` and no blocker (`bottom_score_without_blocker`, red first). One definition of finding kept; the persona is made to name what blocks. |
| A6 | §10 "Nothing averages" and plan:20 "no mean anywhere" | plan:138: even-count median = mean of the two middle values; on 1–5 that yields 3.5, a value no anchor describes. Inside one stratum, so not a §5 break — a wobble in the plan's own words. | `Cell.median: i64` = the lower middle; `min`/`max` already show the spread. |
| A7 | §10 "a split verdict … escalates one level" | Not built, declared open (plan:257). The one §10 rule with no home in the engine. | Answered in B2: `matrix` emits `escalation` beside the strata. |

Nothing in D5 breaks a rule; rows 1, 5 and 7 are the open questions below.

## B. The three open questions

### B1. The held-out detector: which hand-scored table, and the 1–10 → 1/3/5 mapping

**Recommendation.** The detector's target is **the hand-scored jury table the maintainer keeps privately** (second round: six criteria A–F, five personas, 16 screenshots of ours against 14 of seven comparators, two rounds). The report's wording is a mislabel: the round it cites is four personas, a pick of C among A–D, comparators "from memory, unverified", no dimension. Only the jury table has what the engine is built to reproduce — artifacts as attachments, comparators as snapshots, six dimensions, a ranking, and a round-1 → round-2 delta (F phone layout 5 → 7). Keep the hostname pick as a second, cheap check of `pick`: four personas, lens `compare`, options as `told`, every comparator line labelled `recalled` by R3, ≥3 of 4 pick C.

**The mapping: ranks, not values, per stratum.** The hand table is itself one number per criterion across five personas of three strata — the exact table §2.3 refuses. So the detector cannot be "reproduce the table"; it is: the **primary stratum's** median ranking of the six dimensions is within one rank of the hand ranking (round 2: E > A = D > B > F > C), and every other stratum's ranking is reported beside it. The hand scores (6, 6.5, 7, 7.5, 8, 9) had no anchors; mapping them to anchors is false precision. On 1/3/5 six dimensions tie too often to rank at all (E, A, D all at 5 and B, F, C all at 3 makes "within one rank" decide by noise). So the detector's lens is **1–10, integers, anchors in words at 1, 4, 7, 10** (step 3; the types allow any scale and any anchor set, `model.rs:181-216`).

**Ship it as data, not seeded.** §2.4 says "Four ship"; the detector lens lives at `fixtures/lenses/site.toml`, added with `lens add` by the detector's run script; the four seeded lenses stay 1–5 (D5 row 1 stands). The six criteria become its dimensions with the table's names.

**Cost if wrong.** If the hostname pick was meant: nothing lost, the pick check is kept. If a 1–10 lens is the wrong call: one fixture lens nobody else uses. The reverse error — a 1–5 detector that cannot rank — leaves the held-out case unable to go red, which is the one thing a detector must be able to do.

### B2. What a "split" is, numerically

**Recommendation.** For a (lens, dimension), two strata **split when their `anchor_met` ranges do not overlap**: every persona of one stratum met a strictly lower anchor than every persona of the other. `anchor_met` (`model.rs:305`) is already stored and is the rubric band the persona claims; min and max are already in the cell. No median, no mean, no threshold to tune. Examples on 1/3/5: primary {3, 5} vs bystander {1} → split; primary {3, 5} vs bystander {3} → no split; scores 3 and 4 with both at anchor 3 → no split (a point apart is not an anchor apart). The other half of §10's rule is already crisp: any blocker from a primary-stratum persona in the latest batch.

`matrix` adds a top-level `escalation` beside `strata` (not inside `Matrix`, so `matrix_has_no_cross_strata_field` still holds as "no cross-strata *score*"): `{ splits: [{lens, dimension, strata: [a, b]}], primary_blockers: n, escalate: bool }`. The engine says it; the hook acts on it; nothing is averaged.

**Cost if wrong.** Non-overlap is on the strict side: a stratum of one outlier never splits a stratum of three. Too strict → `low` applies itself on a public defect one bystander saw (the §10 cost); too loose → every `low` run escalates to `high` and the token saving §10 exists for is gone. The fixture run (seven personas, three strata, two lenses) is the detector: if no cell splits on `compare`, read the cells before loosening.

### B3. `decide` as the writer; "the same point"

**`decide` is right.** A decision is "a person's word" (§4); it must be explicit, append-only and signed (`--by`), and none of the other verbs is a person's act. One addition that turns §10(2) into a check: `check` lists `fixed` findings whose subject has no later run with `landed: true` as `fixed_unverified` — a fix claimed in prose with no delta is "visible as such" (§2.6) only if something lists it.

**"The same point" = (persona, dimension, citation hash)** — one key more than D5. With (persona, citation hash) alone, a different complaint about the same node under another dimension (the hero cited for "no price", dismissed; later cited for "contrast") is refused: a wrong refusal, and the first entry in the `wrong_refusal` template. With the finding text in the key, a rewording slips past — the loop §10(3) exists to stop. The dimension is the middle. Note what the citation hash already gives for free: it is built from the attachment hash (plan:136), so a changed subject version is new evidence by construction — the loop-stopper expires exactly when the page changes. Say so in the README.

**Cost if wrong.** A false refusal is loud (the persona's verdict bounces, the template fills); a slipped rewording is silent. If dimension proves too coarse, the key can tighten to (persona, dimension, citation hash, blocker text) without a schema change.

## C. D3 — the detail knob

`--detail compact|full` on `brief` and `matrix` is the right shape for v0.1, with one addition. The requirement names two axes: *how much* (low to high spend) and *what kind of material* (technical, external, feature docs). The second axis is already the composition of the run, not a flag: a `diff` attachment is the technical end, a comparator snapshot is "external", a `document` subject is "feature docs". A lens-shaped or kind-shaped knob would duplicate what `subject add --attach` and `comparator add` already say. The first axis is `detail` (plan:216-219) with the honest measure beside it — bytes per brief, never a token guess.

The addition: `detail` as a field of `TriggerRule` (`model.rs:519-530`, default `compact`), so `jurygraph.toml` can say "console page: three personas, `critical`, compact; public page: full panel, every lens, full". §10 puts the *size* of a run in the rule; the knob belongs where the size is decided, not only on the command line.

Not for v0.1: a byte budget (`--detail <bytes>`) that fills attachments in a fixed order until the cap. The two words can grow a third form later without breaking a caller. Cost if wrong: two levels prove too coarse for a 15-persona release run and the byte form comes forward by a version.

## D. Governance specific to a persona jury

Beyond the generic set of Task 3b, a jury tool needs these written down; one line each on what the document must say.

1. **Personas are composites, never people** (`PERSONAS.md`, linked from CONTRIBUTING): a persona may not be modelled on, named after or identifiable as a real person — no real name, no employer-plus-role that singles one person out, no customer's data in a profile; `persona generate` material must be the contributor's to share; a collision with a real identifiable person is a bug to report, not a feature.
2. **A lens is a standard, so it is versioned like one** (`docs/lenses.md`): a contributed lens carries anchors in words at every anchor score, a `standard` pointer that is public and citable, and a version bump on any anchor change — anchors are never edited in place, because a delta across changed anchors is meaningless (A4), and the engine hashes the body.
3. **A wrong refusal is a report, not a loosening** (`.github/ISSUE_TEMPLATE/wrong_refusal.yml`, planned, and a paragraph in CONTRIBUTING): the template asks for the refusal code, the verdict JSON, the run's brief hash and what the persona should have been allowed to say; a refusal changes only with a red-first test and a numbered decision, never for one model's habit.
4. **What a citation proves** (README, under the evidence classes): the engine checks that a cited node, line or label exists in the run, not that the claim about it is true; `seen` is the persona's claim about what it opened; the subject is what the submitter attached, so a published matrix is meaningful only with its subject, panel, lens and brief hashes beside it.
5. **Model and provider neutrality** (`GOVERNANCE.md`): the engine never calls a model; the repository recommends no provider and holds no key; `model` on a run is the agent's declaration, unverified; no "model X scores Y" claim appears in the README or docs without a checked-in run and its hashes; the PR template asks which model and harness produced a change (superpowers' practice) and the answer is recorded, not judged.
6. **A verdict about a comparator is an opinion on a snapshot** (`docs/publishing-results.md`): a jury line about a named third-party product is a fictional persona's opinion, run by a model, on a dated hashed snapshot; anyone publishing a matrix that names a third party carries that sentence, the snapshot date and the evidence class of every line.
7. **The human list is law** (`GOVERNANCE.md`, non-negotiables): `HUMAN_CLASSES` — money, permission, deploy, secret, licence, public-claim, brand — cannot sit under a persona quorum; a change to the list or to R11 needs a numbered decision; a pull request that weakens it is closed.
8. **What leaves the machine** (`SECURITY.md`, a privacy section): the engine sends nothing anywhere; what reaches a model is what the agent chooses to open from the brief and the attachments; ARIA walks and screenshots can carry a user's customers' data, so `.jurygraph/` is listed as local state and the fixture is our own site.
9. **A published result is a dated report** (`docs/reports/README.md`, planned): a jury result shared outside a project carries date, model name, the four hashes and the per-stratum table; a result without them is prose and is labelled as such.
