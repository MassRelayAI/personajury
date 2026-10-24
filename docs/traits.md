# Traits
<!-- the catalog table is generated from traits/*/*.toml; tests/traits_catalog.rs keeps the embedded list and the files equal -->

A **trait** is a tendency or a standing fact that makes one reader decide differently from another, never a type or a diagnosis.
It is an input to a persona. It is never a score, never averaged, never turned into a probability, and the model that plays the
persona reads it as "you tend to ...". Decisions: D29 (the catalog), D30 (cover and quotas), D31 (the brief block and slices), D35 and D36 (generating, observing and calibrating traits).

## The catalog

49 traits in 14 families ship in the binary as data (`traits/<family>/<trait>.toml`) and eight trait sets
(`trait-sets/<name>.toml`). Nothing is seeded at store creation: `personajury trait install` adds every trait and set the store
lacks (the same hash is skipped; an earlier catalog version is updated to the catalog's text), and answers
`{installed, skipped, versions_added, kept_local}`. It never overwrites a trait you edited: `trait add` of a shipped id with other
text records origin `edited`, and `trait install` then keeps it and lists it in `kept_local`. `trait install --force --reason TEXT`
overwrites them with the catalog's text and records the reason on the new version (`trait list` shows it); it is idempotent after.

Licence: every anchor is our own words describing a construct. Published questionnaire items are copyrighted or restricted to
academic use, so none is reproduced, paraphrased item by item or scored; each file says `items_copied = false`, and the source is
cited so a reader can check the construct. The catalog does not administer an instrument.

Shapes: a **band** trait has anchors in words at 1, 3 and 5 (2 and 4 lean toward their neighbour); a **category** has two or more
options, each with a sentence (`access.need` may hold a list of them); a **words** trait (`situation.job`) is a phrase of at most 200
characters. `persuasion.route` is not a trait: the brief derives it from `persuasion.involvement` and `cognition.need_for_cognition`.

| id | meaning | shape | moves | evidence | source (construct) |
|---|---|---|---|---|---|
| **access** | | | | | |
| `access.need` | An assistive need the subject must serve | category, multi | read_docs, pick_name_or_design, adopt | strong | WCAG 2.2 success criteria; WHO ICF (functioning model); WebAIM screen-reader surveys |
| **adoption** | | | | | |
| `adoption.adopter_category` | Where on a diffusion curve this reader usually sits | category | adopt | weak | Rogers 1962/2003 (diffusion of innovations) |
| `adoption.ease_demanded` | How little effort the reader will tolerate before leaving | band 1-5 | adopt, read_docs | strong | Davis 1989 (technology acceptance, perceived ease of use); Venkatesh et al. 2003 (effort expectancy) |
| `adoption.innovativeness` | Tries new tools in this domain before others do | band 1-5 | adopt | moderate | Goldsmith & Hofacker 1991 (domain-specific innovativeness) |
| `adoption.tech_self_efficacy` | Believes they can make an unfamiliar tool work | band 1-5 | adopt, read_docs | moderate | Compeau & Higgins 1995 (computer self-efficacy); Bandura 1977 (self-efficacy) |
| **aesthetics** | | | | | |
| `aesthetics.halo_susceptibility` | Reads a good-looking page as true, usable and competent | band 1-5 | trust_a_claim, adopt, read_docs | strong | Thorndike 1920 (halo); Dion, Berscheid & Walster 1972; Tractinsky, Katz & Ikar 2000; Lindgaard et al. 2006 |
| `aesthetics.sensitivity` | How much the look of a thing weighs in the judgement of it | band 1-5 | pick_name_or_design, buy, adopt | moderate | Bloch, Brunel & Arnold 2003 (centrality of visual product aesthetics); openness facet aesthetics in Costa & McCrae 1992 |
| **buying** | | | | | |
| `buying.role` | The seat held in a purchase | category | buy, approve_vendor, trust_a_claim | strong | Webster & Wind 1972 (buying centre); Bonoma 1982; Miller & Heiman 1985 |
| **choice** | | | | | |
| `choice.decision_style` | The habitual way a choice gets made | category | buy, trust_a_claim, adopt, read_docs, pick_name_or_design, approve_vendor | moderate | Scott & Bruce 1995 (general decision-making style) |
| `choice.loss_aversion` | A loss weighs more than an equal gain | band 1-5 | buy, approve_vendor, adopt | strong | Kahneman & Tversky 1979 (prospect theory); Gachter, Johnson & Herrmann 2007 |
| `choice.maximizing` | Searches every alternative and regrets the one not taken | band 1-5 | buy, pick_name_or_design, approve_vendor | moderate | Schwartz et al. 2002 (maximization scale); Turner, Rim, Betz & Nygren 2012 (maximization inventory) |
| `choice.regulatory_focus` | Pursues gains or guards against losses | category | buy, trust_a_claim, read_docs | moderate | Higgins 1997; Higgins et al. 2001 (regulatory focus questionnaire) |
| **cognition** | | | | | |
| `cognition.ambiguity_tolerance` | Comfortable acting with incomplete information | band 1-5 | adopt, approve_vendor | moderate | Budner 1962; McLain 2009 (MSTAT-II) |
| `cognition.need_for_closure` | Wants an answer now and dislikes ambiguity | band 1-5 | buy, adopt | moderate | Webster & Kruglanski 1994; Roets & Van Hiel 2011 (need for closure) |
| `cognition.need_for_cognition` | Enjoys effortful thinking and reads the long version | band 1-5 | read_docs, trust_a_claim | strong | Cacioppo & Petty 1982 (need for cognition) |
| `cognition.reflection` | Overrides the first intuitive answer when it is wrong | band 1-5 | trust_a_claim, approve_vendor | strong | Frederick 2005 (cognitive reflection test); Thomson & Oppenheimer 2016 (CRT-2) |
| **consumer** | | | | | |
| `consumer.brand_consciousness` | Known names and logos reassure | band 1-5 | buy, trust_a_claim | weak | Sproles & Kendall 1986 (consumer styles inventory) |
| `consumer.choice_overload` | Many options confuse rather than help | band 1-5 | buy, read_docs | moderate | Sproles & Kendall 1986 (confused by overchoice); Iyengar & Lepper 2000 |
| `consumer.price_consciousness` | Price is read first and weighs most | band 1-5 | buy, approve_vendor | moderate | Lichtenstein, Ridgway & Netemeyer 1993; Sproles & Kendall 1986 (price-value style) |
| `consumer.quality_perfectionism` | Will not settle for good enough | band 1-5 | buy, pick_name_or_design | weak | Sproles & Kendall 1986 (consumer styles inventory) |
| **expertise** | | | | | |
| `expertise.domain` | How well the reader knows this subject's field | band 1-5 | read_docs, trust_a_claim, approve_vendor | strong | Dreyfus & Dreyfus 1980 (novice to expert); Chi, Glaser & Farr 1988 (nature of expertise) |
| `expertise.numeracy` | Comfort reading numbers, rates and tables | band 1-5 | read_docs, approve_vendor | strong | Fagerlin et al. 2007 (subjective numeracy); Cokely et al. 2012 (Berlin numeracy) |
| **personality** | | | | | |
| `personality.agreeableness` | Avoids conflict and gives the benefit of the doubt | band 1-5 | trust_a_claim | moderate | Costa & McCrae 1992 (NEO five-factor); Goldberg 1990/1999 (Big Five, IPIP) |
| `personality.conscientiousness` | Plans, checks and finishes; dislikes mess and shortcuts | band 1-5 | read_docs, approve_vendor | moderate | Costa & McCrae 1992 (NEO five-factor); Goldberg 1990/1999 (Big Five, IPIP) |
| `personality.extraversion` | Energised by people and pace, decides out loud | band 1-5 | - | weak | Costa & McCrae 1992 (NEO five-factor); Goldberg 1990/1999 (Big Five, IPIP) |
| `personality.honesty_humility` | Sincere and unentitled, unmoved by status and flattery | band 1-5 | trust_a_claim, pick_name_or_design | moderate | Ashton & Lee 2007 (HEXACO honesty-humility factor) |
| `personality.neuroticism` | Worries and feels the downside first | band 1-5 | buy, approve_vendor | moderate | Costa & McCrae 1992 (NEO five-factor); Goldberg 1990/1999 (Big Five, IPIP) |
| `personality.openness` | Seeks new ideas and aesthetics, tolerates the unfamiliar | band 1-5 | adopt, pick_name_or_design | moderate | Costa & McCrae 1992 (NEO five-factor); Goldberg 1990/1999 (Big Five, IPIP) |
| **persuasion** | | | | | |
| `persuasion.involvement` | How much this subject matters to the reader personally | band 1-5 | trust_a_claim, read_docs | strong | Zaichkowsky 1985 (personal involvement inventory); Petty & Cacioppo 1986 (elaboration likelihood) |
| **risk** | | | | | |
| `risk.ethical` | Bends rules when it pays | band 1-5 | approve_vendor | moderate | Weber, Blais & Betz 2002; Blais & Weber 2006 (DOSPERT, ethical domain) |
| `risk.financial` | Willing to spend or lose money on an uncertain return | band 1-5 | buy, approve_vendor | strong | Weber, Blais & Betz 2002; Blais & Weber 2006 (DOSPERT, financial domain) |
| `risk.health_safety` | Accepts physical or safety exposure | band 1-5 | - | moderate | Weber, Blais & Betz 2002; Blais & Weber 2006 (DOSPERT, health and safety domain) |
| `risk.social` | Willing to look wrong in front of others | band 1-5 | adopt, pick_name_or_design | moderate | Weber, Blais & Betz 2002; Blais & Weber 2006 (DOSPERT, social domain) |
| **situation** | | | | | |
| `situation.budget_authority` | What this reader may spend without asking | category | buy, approve_vendor | strong | Webster & Wind 1972; practitioner B2B qualification (budget, authority, need, timing) |
| `situation.job` | The job the reader has come to get done | words | buy, trust_a_claim, adopt, read_docs, pick_name_or_design, approve_vendor | weak | Christensen et al. 2016; Ulwick 2005 (jobs to be done) |
| `situation.switching_cost` | What leaving the current tool would cost | band 1-5 | adopt, buy | strong | Burnham, Frels & Mahajan 2003 (switching costs); Samuelson & Zeckhauser 1988 (status quo bias) |
| `situation.time_pressure` | How long before the tab is closed | band 1-5 | buy, trust_a_claim, adopt, read_docs, pick_name_or_design, approve_vendor | strong | Svenson & Maule 1993 (time pressure in judgement); Payne, Bettman & Johnson 1993 (adaptive decision maker) |
| **social** | | | | | |
| `social.influence_susceptibility` | Weight given to endorsements, logos and counts of others | band 1-5 | trust_a_claim, buy, adopt | strong | Bearden, Netemeyer & Teel 1989; Cialdini 1984/2001 (social proof, authority); Goldstein, Cialdini & Griskevicius 2008 |
| `social.need_for_uniqueness` | Prefers what few others have | band 1-5 | buy, pick_name_or_design, adopt | moderate | Snyder & Fromkin 1977; Tian, Bearden & Hunter 2001 (consumers' need for uniqueness) |
| `social.need_to_belong` | Wants to use what their people use | band 1-5 | adopt, pick_name_or_design | moderate | Baumeister & Leary 1995 (need to belong); Leary et al. 2013 |
| `social.rejection_sensitivity` | Expects to be turned away or laughed at | band 1-5 | read_docs, adopt, trust_a_claim | moderate | Downey & Feldman 1996 (rejection sensitivity questionnaire) |
| `social.self_esteem` | Steadiness of self-worth under a page that talks down or over-promises | band 1-5 | trust_a_claim, read_docs | moderate | Rosenberg 1965 (self-esteem scale); Rhodes & Wood 1992 (self-esteem and persuasibility: an inverted U) |
| `social.self_monitoring` | Adjusts choices to how they will look to others | band 1-5 | buy, pick_name_or_design, trust_a_claim | moderate | Snyder 1974 (self-monitoring); Lennox & Wolfe 1984; Snyder & DeBono 1985 (image versus quality appeals) |
| `social.status_consciousness` | Buys and recommends what signals standing | band 1-5 | buy, pick_name_or_design, approve_vendor | moderate | Veblen 1899 (conspicuous consumption); Eastman, Goldsmith & Flynn 1999; Lichtenstein, Ridgway & Netemeyer 1993 |
| `social.stigma_consciousness` | Expects to be judged as an outsider in this field | band 1-5 | read_docs, adopt, trust_a_claim | moderate | Pinel 1999 (stigma consciousness) |
| **trust** | | | | | |
| `trust.automation` | Relies on a system's output without checking it | band 1-5 | adopt, trust_a_claim | moderate | Lee & See 2004 (trust in automation); Jian, Bisantz & Drury 2000; Korber 2019 |
| `trust.institutions` | Expects companies and authorities to act in good faith | band 1-5 | trust_a_claim, approve_vendor | moderate | Mayer et al. 1995 (ability, benevolence, integrity); institutional-confidence items in national surveys |
| `trust.marketing_skepticism` | Disbelieves advertising and claims by default | band 1-5 | trust_a_claim, buy, read_docs | strong | Obermiller & Spangenberg 1998, skepticism toward advertising (SKEP) |
| `trust.propensity` | General readiness to rely on strangers before there is evidence | band 1-5 | trust_a_claim, buy | strong | Mayer, Davis & Schoorman 1995 (propensity to trust); Rotter 1967 (interpersonal trust) |

`moves` are the decisions a trait is known or expected to move. `evidence` is honest: most personality traits predict behaviour
modestly (correlations around .1 to .3), which is why the engine counts what a trait does in a given jury rather than assume it.

The eight trait sets: `developer-tool`, `consumer-app`, `b2b-purchase`, `marketing-site`, `docs`, `investor-deck`,
`public-sector-legal`, `name-and-design`. A set lists at most 8 traits and names a `cover` of at most 4 of them
(`personajury trait set list`). Adding a trait file or a set of your own: `trait add --file F`, `trait set add --file F` (a trait
whose family is not one of the fourteen is `project.<slug>`).

## On a persona

A persona file may carry `traits`: a map of trait id to value, **at most 8**, written after the prose as its summary:

```json
"traits": {"trust.marketing_skepticism": 4, "choice.decision_style": "rational", "access.need": ["screen_reader"]}
```

A persona trait is accepted only if the store's catalog holds it: shipped and installed, or added with `trait add` (generated
from material or not). Refusals at `persona add`: `unknown_trait` (the store has no such trait), `refused_trait_id` (the refused
list and the `culture` family, checked first), `trait_text_refused` (a words value: at most 80 characters, no digit, not starting
"You are" or "I am"), `trait_value_off_scale`, `trait_option_unknown` (an option the trait lacks, or a list on a trait that holds
one) and `traits_too_many` above 8. A sensitive attribute is never refused: the answer lists it under `sensitive` (see "Simulated
demographics" below). A persona with no `traits` keeps the hash it had. `persona list` shows `traits: n`.

A persona may also carry `serves`, tags for the audiences it speaks for, used by `brief --seats auto`.

## In the brief

`brief --traits <set>|all|off` (MCP `traits`; a rule's `trait_set` is the default, else `off`). With `off` no byte of any brief
changes. Otherwise each brief gets, between the persona and the stake, a block:

```
## How this reader tends to decide (told)

- You tend to: <the anchor at 1, 3 or 5, or the option's sentence>
- You tend to lean this way: <the anchor at 1 for a 2, at 5 for a 4>
- Your work: <situation.job, the words value>
- You tend to: <the derived route, last, when both of its inputs are rendered>

Told, not seen: a line leaning on one is `told`; a line about the subject still cites it.
```

Trait ids in sorted order. The value, the id and the title are never printed; no digit and no "You are" appears in the block.
`set` mode renders the set's cover traits the persona has, and its other set traits unless they sit at the middle of a band (3,
the default reading); the answer lists what is missing as `trait_set_incomplete`
(`apply-check` adds the informational reason of the same name). A `--traits` that differs from the rule's is an override and
needs `--reason` (`override_without_reason`); an unknown set is `trait_set_unknown`. The run records
`traits_json = {mode, set, versions, draw}` (and `incomplete`, the seats lacking set traits).

`persona generate --from M --count N --traits <set>` adds the set's traits and one row per persona to the brief: for each cover
trait, the band `low`, `mid` or `high` by `(k + index of the trait) mod 3`, a Latin-square start for pairwise coverage.

## Panels: quotas, cover, balance

A panel term may carry a filter: `3 builder=primary[trust.marketing_skepticism>=4, access.need=screen_reader]`. The operators are
`=`, `!=`, `>=` and `<=`; `=` on a multi-option category means "contains"; `>=` and `<=` on a category are `invalid_filter`. A
persona without the trait never matches. The filter narrows the pool before the seeded draw.

`panel add NAME --query Q --cover a,b,c` (at most 4 traits, else `cover_too_large`) makes the draw cover-aware: for each seat the
persona that adds the most new one-way band cells for the term's stratum, then the most new panel-wide pairs, ties to the earliest
in the seeded order. Under a cover a persona added to one term can move another term's seats (coverage trades the per-term
isolation of the plain draw); the pool is the same between rounds, so `delta` is unaffected.

`panel resolve` and the `brief` answer then carry `coverage`: `one_way` (seats per band, per stratum, with `unset`), `pairs`
(panel-wide: `covered` of `total`, the `uncovered` list, the correlated pairs left out), `balance` (bands no seat of a stratum
holds), `collinear` (two cover traits that sort the seats identically) and `confounded` (a band every seated persona of a tag
shares while a seat without the tag differs). Bands: low 1-2, mid 3, high 4-5; a category's bands are its options. All counts
of seats; nothing is a score. Read `balance` before you brief.

## Results: slices

`matrix RUN... --slice TRAIT [--table]` (MCP `slice`) adds `slices`, a sibling of `strata`, never a field of one: for each stratum, the
groups by band (low, mid, high, or the options in order) with their personas, `n`, `thin` (fewer than two), the cells of that group
(scores, median as the lower middle, min, max, blockers), who has the trait `unset`, `separates` (cells on which two groups'
`anchor_met` ranges do not overlap, of the stratum's cells) and warnings (`collinear with ...`). A band nobody holds is omitted.
No group is pooled with another stratum's. `summary RUN... --slice TRAIT` adds one fixed sentence per stratum:
"Among primary jurors, those high on "..." (3: a, b, c) scored clarity 4, 5, 5; those low (1) scored it 2. In this jury, not a
claim about people."

## Generating traits from material

`personajury trait generate --from PATH [--count N]` (D35; PATH is a file or a folder, N is 3 by default, 1 to 8) writes
`.personajury/generate/trait-<material hash>.md`, a brief for your agent, and calls no model. Step one: list, in the material's own
words, the ways its readers are known to differ ("procurement versus an engineering lead", "clinicians under time pressure"). Step two:
draft N trait files in the shape of the catalog's, with `family = "project.<slug>"` (the slug is the material's file or folder name),
`evidence = "project"`, `source = "material:<hash>"`, anchors or options as what the reader does after "You tend to". Then
`trait add --file F --generated-from HASH` for each. **Every text of the trait is read there, as for any trait**, and a sensitive
attribute is labelled in the answer (`sensitive`); `trait generate` labels the material's name the same way and refuses only a
`culture` material (`refused_trait_id`). A project trait is accepted by the project, not by the catalog: it shows `(P)` in
`trait list` (`badge`) and in `matrix --slice` headers. No typology, no country dimension.

## Traits from an existing juror: suggest (prose, with quotes) and observed (verdicts, with counts)

Two ways to give an existing persona its traits, neither with a model and neither writing back (D35). Prose first, traits as its
summary, each with its line of evidence.

**`trait suggest --persona ID [--set NAME]`** writes `.personajury/generate/trait-suggest-<id>-<hash8>.md`: the persona's prose beside
the anchors of the candidate traits (the set's, or every trait in the store), and a task: propose a value only where a sentence of the
prose supports it, and copy that sentence or a phrase of it as the evidence. The agent answers one JSON object
`{"persona": ID, "traits": {...}, "evidence": {...}}`. **`trait suggest --check FILE [--persona ID]`** (MCP
`personajury_trait_suggest_check`) verifies it: ids and values as `persona add` would, a quote for every trait
(`suggestion_without_quote`), each quote at least 3 words and 12 characters (`quote_too_short`) and verbatim inside one prose field
with whitespace folded (`quote_not_found`, case-sensitive; a quote that runs from one field into the next is not found). It answers
`data.persona_file` (the stored persona with the suggested traits set; the others kept) and `data.sensitive`, and writes nothing: save
the file and run `persona add`, which makes a new version.

**`trait observed --persona ID [--persona ID ...] [--runs A,B]`** (MCP `personajury_trait_observed`) counts what the persona did in its
accepted initial verdicts: where its score fell against its stratum's median in each (lens, dimension) (`below`, `at`, `above`), its
blockers and its bottom-of-scale scores against the stratum's median count (`more`, `same`, `fewer`), the cell it punished most, its
evidence classes, and its picks (subject, comparator or attachment; with or against the stratum's majority). A comparison needs a
stratum of three jurors. Then, for each trait the persona declares, `consistent`, `contradicted`, `mixed` or `no_signal`, with the counts
of each rule behind it. A 3 predicts nothing and is always `no_signal`.

The mapping table is `traits/observe-map.toml` (18 rows, embedded; a project adds rows in `.personajury/observe-map.toml` and
overrides none):

| rule | signal | direction | bears on | pole |
|---|---|---|---|---|
| `price-severity` | compare / price | below | `consumer.price_consciousness` | high |
| `trust-severity-skeptic` | compare / trust | below | `trust.marketing_skepticism` | high |
| `trust-severity-trusting` | compare / trust | above | `trust.propensity` | high |
| `critical-severity` | critical / any | below | `adoption.ease_demanded` | high |
| `taste-severity` | taste / any | below | `aesthetics.sensitivity` | high |
| `clarity-severity-novice` | compare / clarity | below | `expertise.domain` | low |
| `clarity-severity-rejection` | compare / clarity | below | `social.rejection_sensitivity` | high |
| `risk-severity` | tradeoffs / risk | below | `risk.financial` | low |
| `loss-severity` | tradeoffs / risk | below | `choice.loss_aversion` | high |
| `numeracy-severity` | financial-model / any | below | `expertise.numeracy` | high |
| `blockers-skeptic` | blockers | more | `trust.marketing_skepticism` | high |
| `blockers-maximizer` | blockers | more | `choice.maximizing` | high |
| `evidence-recalled` | evidence | recalled over seen | `cognition.need_for_cognition` | low |
| `evidence-seen` | evidence | seen over recalled | `cognition.need_for_cognition` | high |
| `evidence-seen-rational` | evidence | seen over recalled | `choice.decision_style` | `rational` |
| `pick-unique` | pick | against the majority | `social.need_for_uniqueness` | high |
| `pick-follows` | pick | with the majority | `social.influence_susceptibility` | high |
| `bottom-harsh` | bottom | more | `personality.agreeableness` | low |

**A bearing, not a measurement; a contradicted trait is a question for the author, not a verdict on the persona.** A rule is
`consistent` when its direction's count exceeds the opposite one's, `contradicted` when the opposite exceeds it, `no_signal` on a tie.

## Calibrating a trait against people: predicts, played, missed, inert, thin

A person who enters a verdict can declare their own traits: `verdict --run N --file F --by-human "Dr Rao, ward nurse" --traits T.json`,
where `T.json` is a `traits` map in the persona's shape (the same refusals). They are what the person says of themselves; the engine
administers no questionnaire. `matrix` shows them under `human[].traits`. The MCP has no `by_human`, so no `traits` there either.

`calibrate JURY HUMAN --slice TRAIT [--out F]` groups both sides by the trait's bands or options inside each stratum (a side without
the trait is `unset`, left out), compares each group's cells by the D20 rank rule, and counts the cells on which two groups' `anchor_met`
ranges do not overlap, in the jury (`separates_in_jury`) and in the people (`separates_in_people`). The stratum's word, in `slices`:

| word | when |
|---|---|
| `predicts` | both sides separate on at least one common cell: the trait moves the jury as it moves people |
| `played` | the jury separates and the people do not (or on other cells): the model acts the label |
| `missed` | the people separate and the jury does not |
| `inert` | neither separates |
| `thin` | a group on either side has fewer than two, or fewer than two groups are held: it says nothing more |

`calibrated` and `apply-check --calibration` are untouched. `trait report --calibration F [F ...]` counts the words per trait and
stratum across files (and how many files sliced it); a file that is not a calibration is `not_a_calibration`. No total across strata.

## Lint

`persona lint [ID ...]` warns, never refuses, and exits 0: `near_copy {a, b}` (two personas with the same non-empty trait map and a
shared tag, or the same tags) and `confounded {trait, tag, band}` (two or more personas with the tag hold the band, one or more
without it hold another: in what this store would show, the tag and the band cannot be told apart). Whether a trait fits the prose
is not checked: the engine cannot read prose for meaning, and the author does that. A verdict whose note or blocker repeats a label the
persona was handed (the trait's title, or the id's last segment with spaces: "marketing skepticism", "a marketing skeptic") is
accepted with `warnings: [{code: "label_echo", line, trait}]`; `matrix` counts them per stratum. Only the traits the run's brief
rendered for that persona count (its trait mode and set); a run briefed with traits off gives no echo. `graph RUN --svg --slice TRAIT` fills
the persona nodes by group with a legend line of the counts per stratum.

## What is refused, and why

`refused_trait_id` names its reason. Typologies without predictive validity: `mbti`, `myers_briggs`, `enneagram`, `disc`,
`learning_style`, `generation`, `astrology`, `zodiac`, `blood_type`. Attributes of the persona rather than sensitivities of the
juror: `appearance`, `attractiveness`, `beauty`, `fame`, `celebrity`, `outcast`, `outsider`. Beauty, fame and outsider status are
the juror's sensitivities (`aesthetics.sensitivity`, `social.influence_susceptibility`, `social.stigma_consciousness`), never the
persona's attributes. The check looks at the last segment of the id, so `x.beauty` is refused too. The `culture` family (as
`project.culture` too) is refused: "a country dimension, not a mechanism". Nothing else is refused for what it names (D39).

## Simulated demographics: allowed, labelled, recorded

Personas are simulations (D39, the maintainer: "We are keeping everything as sims, not deciding or angling"). A trait or persona
that names an age, a gender, a religion or another sensitive attribute is accepted and labelled `sensitive: [...]`, with these
twelve categories, in this order: age, gender, ethnicity, religion, nationality, health, disability, sexuality, politics, marital,
pregnancy, genetics. `trait add`, `trait generate`, `trait add --generated-from`, `persona add`, `trait suggest --check` and
`verdict --by-human --traits` answer the labels. `brief` records the categories its seated personas name on the run
(`uses_sensitive`); `matrix` answers them; `summary` (and `--md`, `--html`) adds one line, "simulated demographics: age, gender —
personas are simulations, not real-world data". No cap and no human class follow from it. Composites only, still: never a real,
named person (`PERSONAS.md`).

**What is read.** Every text of a trait (id, family, title, caveats, source, each anchor, each option's id, title and sentence),
every free-text value, and a persona's prose (role, context, daily tasks, fears, never reads, decides by, credential, stake rules,
tags). **How.** The text is normalised first, without a crate: fullwidth letters, mathematical letters and ligatures folded (the part
of NFKC the table needs); the Latin look-alikes of Cyrillic and Greek letters mapped (a Cyrillic "е" in "Gеnder"); lower case;
zero-width characters, the soft hyphen, direction marks and combining marks removed; marked Latin letters folded ("género" is
"genero"). Then it is read twice: word by word, and with the separators `_ - .` and space removed between words, so `gen_der`,
`non-binary` and "senior citizen" are read whole. A joined stem starts where a word starts; a short prose word ("a", "be", "the")
never starts one, so "a general rule" is not `age` and "be hindered" is not `behinder`.

**The table** (`SENSITIVE_STEMS` in `domain/traits.rs`, one test per entry). *within*: anywhere in a word; *start*: at its
start; *word*: the whole word only, for a short or ambiguous stem.

| category | stems |
|---|---|
| age | `age` (start; not `agent`, `agency`, `agenda`), `elderly`, `senior citizen`, `pensioner`, `edad` (word), `alter` (word; not `alternative`) |
| gender | `gender`, `sex` (start), `trans` (word; not `transform`, `transaction`), `transgender`, `nonbinary`, `género`/`genero` (word; not `generous`), `geschlecht`, `female`, `male` (word), `woman`, `women` (word) |
| sexuality | `sexual`, `orientation` (start) |
| ethnicity | `race` (start), `racial`, `ethnic`, `etni`, `étni`, `ethnie`, `raza`, `rasse` (word), `caste`, `tribe`, `skin`, `color`, `colour` (start), `tribal` |
| religion | `religion`, `religio`, `religi`, `faith`, `muslim`, `christian`, `jewish`, `hindu`, `buddhist`, `islam`, `catholic`, `atheis`, `sikh` (word) |
| nationality | `nationality`, `national`, `country`, `nacional` (start; not `international`), `nationalit`, `citizenship`, `immigra` |
| health | `health` |
| disability | `disab`, `behinder`, `handicap` |
| politics | `politic`, `union` (start) |
| marital | `marital` |
| pregnancy | `pregnan`, `schwanger` |
| genetics | `genetic` |

Unmarked stems match *within* a word. The table errs wide (`colorful` is labelled ethnicity): a label is a line, not a refusal. The
shipped catalog is reviewed by hand: a text it ships unchanged carries its reviewed label, and only `risk.health_safety` has one
(health); "in good faith" and "colour vision" are words by chance. An edited text is read again. A words value (`situation.job`)
is at most 80 characters, holds no digit and may not start "You are" or "I am" (`trait_text_refused`); the brief prints it as
"Your work: ...".

## Culture

A market's view comes from personas written from real material about that market (`persona generate --from`); no country-dimension
trait ships. The design jury split on it and the ship decision stands on the safety numbers (D29).
