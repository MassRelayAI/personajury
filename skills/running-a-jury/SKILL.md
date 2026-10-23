---
name: running-a-jury
description: Use when a change or a subject (a page, a diff, a design, a price table) should be judged by personas - it is the agent's loop with JuryGraph: should-run, brief, one subagent per persona, verdict, matrix per stratum, foreman, decide, delta.
---
<!-- adapted from comparegraph skills/using-comparegraph/SKILL.md (style) -->
# Running a jury

JuryGraph never calls a model. It holds the store, writes the briefs, ingests verdicts, refuses the ones without evidence,
and reports scores per stratum. You run the personas, with your own subagents and your own model.

## 0. Is a jury needed?
If a diff triggered this, ask the rules first:
```
jurygraph should-run --diff change.diff          # or --paths site/pricing.html,crates/x.rs  [--config jurygraph.toml]
```
It answers `none | low | high | human` and the spend. `none`: stop, tests and the adversarial read decide. `human`: a
person decides; a jury may advise but cannot apply the change. The hook runs at lane finish, not per save.

## 1. Brief
Subject, panel and lens exist already (`subject add`, `panel add`, `lens list`). Every panel persona needs a stake in this
run, declared here and nowhere else:
```
jurygraph brief --subject home --panel fixture --lens critical \
  --stake builder-ana=primary,company-hanna=secondary,investor-claire=bystander,... \
  --model sonnet --spend 3 --detail compact [--comparator acme-pricing]
```
`--spend N` takes the first N personas, primary then secondary then bystander. `--detail full` inlines text attachments
(more tokens per persona); `compact` lists attachment paths and hashes. The answer names the run id and one brief file per
persona and lens. A panel query that resolves to fewer personas than `--spend` is refused.

## 2. One subagent per persona
Give each subagent its brief file path and nothing else. It reads the attachments the brief lists (the ARIA walk, the
PNGs, the comparator snapshots) and writes exactly one verdict JSON in the shape the brief shows:
```json
{ "persona": "builder-ana",
  "lines": [ { "dimension": "price", "score": 2, "anchor_met": 1, "evidence": "seen",
               "citations": ["pricing.walk.json#price-team", "cmp:acme-pricing/pricing.png"],
               "blockers": ["the Team price is hidden behind Contact us"], "note": "..." },
             { "dimension": "price", "about": "cmp:acme-pricing", "score": 5, "anchor_met": 5, "evidence": "seen",
               "citations": ["cmp:acme-pricing/pricing.png"], "blockers": [], "note": "this line scores Acme itself" } ],
  "pick": { "choice": "cmp:acme-pricing", "reason": "..." } }
```
**Keep jurors independent.** Give each subagent a private output folder and tell it to open only its brief and the
files the brief lists, by full path: a juror that globs a shared folder sees the others' scores, and its verdict is void.
In the fixture run two of seven did exactly that; both were re-run.

`about` names the option a line scores, and the score is that option's own. A line without `about` scores the subject
(under `compare`: the subject against the comparators). Never put a comparator in `about` to score the subject against it.
`about` is optional: one of the subject's attachment labels or `cmp:<name>`, and there is one line per (dimension, `about`);
every lens dimension needs at least one line, which is how a persona picks one option and scores all. `pick` only when the
lens asks for one. Citations are `<label>[#locator]` or `cmp:<name>/<label>[#locator]`. Run
personas in parallel; Sonnet per persona is enough.

## 3. Ingest
```
jurygraph verdict --run N --file verdicts/builder-ana.json
```
When it is **refused**, the same persona answers again, given the refusal message. Never edit the evidence, the citation or
the score yourself to make it pass: the engine refuses what the persona did not see. A score at the bottom of the scale needs
a blocker (`bottom_score_without_blocker`); a missing or doubled dimension is refused (`missing_dimension`,
`duplicate_dimension`).

| evidence | meaning | the engine's rule |
|---|---|---|
| `seen` | the persona looked at an attachment of this run | must cite it (a walk node id, an image label, a diff line); under the `compare` lens it must be `about` a comparator or cite a held one |
| `told` | text the brief supplied | |
| `recalled` | the model's memory | shown as recalled; the only class a comparator without a snapshot can have |

## 4. Read per stratum
```
jurygraph matrix N --table [--detail full]      # jurygraph pick N for the forced choices
```
Primary, secondary and bystander are side by side; the engine never averages across them, and neither do you. The
interesting cell is where strata disagree. Within a stratum you get every score, the median (the lower middle value, an integer), min, max and the blockers. There is no
mean anywhere.

## 5. Foreman, then one decision per finding
A foreman pass reads all verdicts and proposes fixes (report §7: Opus for the foreman, Sonnet per persona). Each blocker
(a finding) gets exactly one outcome, recorded by a person or by you with `--by` naming who:
```
jurygraph decide <finding> fixed --by ana
jurygraph decide <finding> dismissed --why "out of scope for v1" --by ana
jurygraph decide <finding> row --row TRK-12 --by ana
jurygraph check            # stale runs (the subject moved) and findings with no decision (from every batch) or fixed and raised again; exits 1
```
A dismissed point (same persona, same dimension, same citation hash; an uncited one is matched by its finding id) cannot
return without new evidence. `check` also lists `fixed_unverified`: findings marked fixed that no later run of the same lens has judged with the persona who raised them. A fixed finding raised again is unresolved (`fix_not_landed`). `delta` lands only when every cell of the earlier run is judged again, no stratum's blockers rose, blockers went down (or stayed at zero) and no dimension fell by more than one anchor.

## 6. Next round
After the fixes, attach the changed subject, `brief` again, ingest, then:
```
jurygraph delta A B
```
A fix claimed in prose without a delta did not land as far as the engine can tell. A run whose subject changed since the
brief is refused until it is briefed again.

## The quorum, and escalation (guidance you apply; the engine does not)
| quorum | who decides | applies itself when |
|---|---|---|
| `none` | tests and the adversarial read | always, no jury |
| `low` | 3 personas, one lens | no primary-stratum blocker and no split |
| `high` | full panel, all lenses, an Opus foreman proposes fixes | the same rule |
| `human` | a person; the jury only advises | never: money, permission, deploy, secret, licence, public-claim, brand |

A split (strata disagree) or any primary blocker escalates one level: low to high to human. Count blockers and agreement;
do not add scores.

**What the engine does not do yet:** it does not compute a split, and it does not apply the quorum rule for you. `matrix`
shows the strata; you read them and decide whether they disagree. It also cannot see colour or layout in a walk: a claim
about contrast or look must cite the screenshot.
