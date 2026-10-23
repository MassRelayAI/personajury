<!-- shape follows comparegraph docs/troubleshooting.md -->
# Troubleshooting

Read `error.code` in the envelope (`--pretty`). Codes are the contract in the plan's "The refusals" table (plus the codes added by D6 in `decisions.md`).

| code | meaning | what to do |
|---|---|---|
| `seen_without_citation` | a `seen` line cites nothing | the persona answers again and cites an attachment of the run, or labels the line `told` or `recalled` if that is what it was |
| `citation_unresolved` | the citation names a label, ARIA node or diff line the run does not hold | check the label against the brief; a diff line is `L<n>` within the diff |
| `comparator_not_snapshotted` | a line names a comparator the run holds no snapshot of | add `--comparator NAME` at `brief`, or mark the line `recalled` |
| `score_out_of_range` | the score is outside the lens scale | shipped lenses are 1 to 5 |
| `unknown_dimension` | the lens has no such dimension | use the dimension ids in the brief |
| `evidence_not_allowed` | the lens does not allow this evidence class (`recall` takes `recalled` only) | change the class or the lens |
| `strata_average_refused` | `matrix --across-strata` | by design; read the strata side by side |
| `subject_moved` | the subject changed since the brief | `brief` again; `check` lists stale runs |
| `persona_without_stake` | no stake was declared for a persona, or the verdict is from a persona not in the run | give `--stake` for every panel persona |
| `pick_without_reason`, `pick_unknown_option`, `pick_missing` | the forced choice is incomplete | the pick needs an option from the brief and a reason |
| `panel_too_small` | the panel resolves to fewer personas than `--spend` | lower `--spend` or widen the query |
| `human_class_under_persona_quorum` | `jurygraph.toml` puts a human class under `low` or `high` | the human classes are fixed in the engine |
| `dismissed_point_repeated` | the same persona raised a dismissed point without new evidence | add new evidence or leave it dismissed |
| `duplicate_dimension` | a second line for the same (dimension, `about`) | one line per dimension and option |
| `missing_dimension` | a lens dimension has no line | add at least one line for every dimension of the lens |
| `unknown_about` | `about` is neither an attachment label of the subject nor `cmp:<name>` of the run | use a label or comparator named in the brief |
| `seen_compare_without_comparator` | under the `compare` lens a `seen` line is not `about` a comparator and cites none | make it `about` or cite a held comparator, or label it `recalled` |
| `bottom_score_without_blocker` | a score at the bottom of the scale names no blocker | name the blocker, or score higher |
| `bad_anchor` | `anchor_met` is not an anchor of the lens at or below the score | pick one of the lens's anchors; the brief lists them (`anchor_met: one of 1, 3, 5, at or below your score`) |
| `invalid_usage` | a verdict's `usage` is not `{input_tokens, output_tokens}` of non-negative integers | fix the numbers or leave `usage` out |
| `human_decision_required` | `decide` on a finding a human batch (`verdict --by-human`) raised, without `--by-human` | a person decides it: `decide <finding> ... --by-human "<name>"`; the agent and the foreman never do |
| `invalid_query`, `invalid_config` | a panel query or a `jurygraph.toml` is malformed | fix the text; the message says where |
| `check_failed` | `check` found findings with no decision (exit 1); `data.unresolved` lists them, a fixed finding raised again is listed with `fix_not_landed`; `fixed_unverified` lists fixes no later run of that lens and persona has judged | `jurygraph decide` each; run the next round |

Never edit a verdict's evidence, citation or score by hand to make it pass.

**Is my store healthy?** `jurygraph doctor`. **Where is it?** `<root>/.jurygraph/store.db`; `--root` changes `<root>`.
**A tool is missing in my agent.** `jurygraph setup` prints a plan; `jurygraph setup --apply` registers the server.

If the engine's answer looks wrong, open a "Wrong refusal" issue with the code, the verb and the minimal verdict.
