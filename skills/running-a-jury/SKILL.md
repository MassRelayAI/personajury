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
Subject, panel and lens exist already (`subject add`, `panel add`, `lens list`). The panel's own terms give each persona
its stake (`3 builder=primary, 2 approver=secondary, 1 others=bystander`), and where a term's pool is bigger than its seats
the engine draws them, seeded from the subject's name: the same subject keeps its bystanders between rounds.

When a rule matched, let it brief for you:
```
jurygraph brief --subject home --rule public-surface --model sonnet
```
The rule's panel, lenses, spend and detail are the defaults, one run per lens. The answer shows `estimate {briefs, tokens}`
before anyone spends them. You may override anything: `--panel`, `--query "2 builder=primary, 1 others=bystander"` (an
ad-hoc panel, not saved), `--spend`, `--lens`, `--detail`. A value that differs from the rule, or a brief above 12 seats,
needs `--reason "<why>"`, and is recorded on the run (`--by` names you); `matrix` shows it. Above 12 seats needs
`--allow-large`, and 1000 is the most. No override can run a jury on a human class.

Without a rule, name everything:
```
jurygraph brief --subject home --panel fixture --lens critical --model sonnet --spend 3 --detail compact \
  [--stake builder-ana=bystander] [--comparator acme-pricing]
```
`--stake` wins over the panel's term and is needed only for a persona whose term has none. `--spend N` takes the first N
personas, primary then secondary then bystander. `--detail full` inlines text attachments (more tokens per persona);
`compact` lists attachment paths and hashes. The answer names the run id and one brief file per persona and lens. A panel
query that resolves to fewer personas than `--spend` is refused, and so is a term that resolves to nobody
(`panel_term_empty`). A citation into a text file (`README.md#L153`) must name a line the file has.

### How much to spend, and who the jurors are
`effort` is one word, set in `jurygraph.toml` or per run with `--effort low|medium|high|max` (medium under a rule): `low` runs 3
seats (one per stratum) and the rule's first lens, `medium` up to 7 seats, `high` up to 12, `max` the whole panel. A
`budget_tokens` in the toml fits the run under a cap (`trimmed` in the answer lists every cut) or refuses it as `over_budget`.
The rule says what the change needs, effort says what you will spend; the run gets the smaller, and `apply-check` will say
`advise` for a cheap jury on a risky change.
A panel term can pin a model (`3 builder=primary@claude-sonnet-5-5`); `--model` is the default. Two models make a less
correlated jury; `apply-check` notes `single_model` when there was only one.
**If you cannot dispatch one subagent per brief, brief with `--independence sequential`.** A sequential run advises; it never
applies. Claiming `independent` from one window is a false record.

### Large juries: run in waves
Every brief answers `waves`, the persona ids in groups of 30 (`--wave N` changes it; up to 12 seats it is one wave). **Run
one wave at a time:** give its personas to subagents, ingest their verdicts, and only then start the next wave. A jury of a
hundred or a thousand then never floods your rate limit. Above 12 seats the brief needs `--allow-large` and `--reason`,
rule or no rule.

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
Each brief states, per lens, the exact values `anchor_met` may take (`one of 1, 3, 5, at or below your score`): copy one of
them, never a number in between. A juror may add `"usage": {"input_tokens": N, "output_tokens": M}` to its verdict (the
tokens it spent, whole numbers); `matrix` and `doctor` then show the total next to the estimate, as reported by you.

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

To look at the whole jury at once, `jurygraph graph N --svg --out jury.svg` (or `graph A B --svg` over two rounds, which
draws each finding's life as a strip of dots). Personas by stake, the dimensions they scored, the findings by decision;
the medians are per stratum, as in the matrix.

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

## The quorum
| quorum | who decides | applies itself when |
|---|---|---|
| `none` | tests and the adversarial read | always, no jury |
| `low` | 3 personas, one lens | `apply-check` answers `apply` |
| `high` | full panel, all lenses, an Opus foreman proposes fixes | the same |
| `human` | a person; the jury only advises | never: money, permission, deploy, secret, licence, public-claim, brand |

## 7. Apply-check, last
```
jurygraph apply-check <run>... [--rule <class>]
```
Run it last and obey it. It answers `apply | escalate | human | advise` and the reasons as data; it applies nothing itself.
| answer | what you do |
|---|---|
| `apply` | every condition is proven: every briefed seat answered, a known quorum, the run under its rule and not overridden down, independent, current, the last round compared and landed, no primary blocker, no split, `check` clean for the subject |
| `escalate` | run the next quorum up (`next_quorum`: low to high, high to human), with the reasons in hand: `primary_blocker`, `split`, `check_failed`, `delta_not_landed`, `human_blocker` |
| `advise` | something is unknown, missing or lowered: `incomplete_jury` (ingest every seat), `quorum_unknown` (brief under a rule), `rule_below_sources`, `overridden_down`, `effort_below_quorum`, `independence_sequential`, `stale_run` (brief again), `delta_unavailable`: use it as advice, re-run for more |
| `human` | the change falls under a human class (money, permission, deploy, secret, licence, public-claim, brand): the jury advises, a person decides |

A split is two strata whose `anchor_met` ranges on a cell do not overlap (touching ranges overlap); `matrix` lists them as
`splits`. Count blockers and agreement; never add scores across strata.

## Experts
A persona may carry a `credential` (shown in its brief and badged `(C)` in the matrix); a lens's `standard` names the source of its
anchors. A real expert's word is `verdict --run N --file v.json --by-human "Dr Rao, ward nurse"`: the same refusals, a seat of the
run to speak from, and the rows stand beside the jury's, never in a cell. Only a person enters one; you never do. A blocker in it
makes `apply-check` escalate until a person closes the finding with `decide --by-human "<name>"`; you cannot.

## The dial: what you decide, what needs a reason, what a person decides
The default is the fast one. You move freely inside the rule and the effort; to leave the rule you give a reason that is
recorded; four things stay with a person.

| choice | you decide (no reason) | needs a recorded reason (`--reason`, `--by`) | a person only |
|---|---|---|---|
| persona mix | the panel the rule names; the seeded draw picks who sits; `--subject` and `--comparator` are yours | `--panel`, `--query` or `--stake` that differ from the rule | |
| persona count | the rule's seats, trimmed by effort | `--spend` above the rule; 13 to 1000 seats with `--allow-large` | above 1000 seats: nobody |
| lens choice | any of the rule's lenses; the first under effort `low` | `--lens` that the rule does not name | |
| dimensions and anchors | `lens generate` drafts them, you accept with `lens add` | editing a shipped lens's anchors (a new version) | an anchor set claimed as a named standard carries the expert's name in `standard` |
| detail, model, seed, independence | `--model` (also per panel term), `--wave`, `--independence` | `--detail` and `--draw` under a rule | |
| effort and tokens | effort at or below the toml's; `budget_tokens` trims | `--effort` above the toml's | |
| quorum | you never set it; the rule's paths do; `--rule` names the rule (recorded on the run with its quorum) | a bigger jury than the rule asks; a rule below the sources' quorum (then `apply-check` advises, `rule_below_sources`) | lowering a quorum is a toml edit, a person's commit; a human class under a persona quorum is refused whoever edits |
| findings | the foreman decides `fixed`, `dismissed` or `row` with `--by foreman` | a `dismissed` needs `--why` | a finding raised by a human batch is decided by a person (`decide --by-human`; a plain --by is refused, `human_decision_required`) |
| apply | `apply-check` answers `apply` only when every condition is proven: every briefed seat answered, a known quorum, the run under its rule and not overridden down, effort within the quorum, independent, current, the delta computed and landed, no primary blocker or split in any run, `check` clean for the subject, no open human blocker | | `human` classes: the jury advises, a person applies |
| human verdicts | | | only a person enters one (`--by-human`) |

**The four hard stops:** the human classes; seats above 1000; a verdict edited to pass; a human-raised finding, or a human verdict
entered by an agent. Everything else is yours, with a reason where the rule said otherwise, and `apply-check` turns a weak, cheap,
incomplete, stale or overridden-down run into `advise` instead of refusing it: anything unknown, missing or lowered advises, never
applies. Raising seats, lenses, effort or detail above the rule is fine. A jury that cannot apply can still advise, which is how a
small subscription still gets a jury.

It also cannot see colour or layout in a walk: a claim about contrast or look must cite the screenshot.
