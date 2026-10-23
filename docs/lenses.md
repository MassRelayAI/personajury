# Lenses

A lens is a **versioned standard**: the dimensions a jury judges a subject on, with the meaning of the scores written in
words. It is data (TOML), loaded with `jurygraph lens add --file lens.toml` and listed with `jurygraph lens list`. The shipped
lenses are in `lenses/`.

## Shape
```toml
name = "critical"
version = 1
scale = [1, 5]
asks_pick = false
evidence = ["seen", "told", "recalled"]
standard = ""

[[dimension]]
id = "works"
title = "It does the job I came for"
anchors = { "1" = "I cannot do what I came for", "3" = "I get it done, with a detour", "5" = "I get it done on the first try" }
```

## Anchors are words at fixed scores
Each dimension says in words what one score looks like: the shipped lenses anchor 1, 3 and 5 on a 1 to 5 scale. A persona's
verdict line gives a score and the anchor it met (`anchor_met`), which must be an anchor at or below the score; otherwise the
line is refused (`bad_anchor`). A score nobody can tie to words is a feeling; the anchors are what makes two runs comparable.

## Editing an anchor means a new version
**Never edit an anchor in place.** A change to any anchor, title, scale or dimension is a new `version`. The store keeps every
version it was given, so an old run still points at the standard it was judged by; the engine hashes the lens body, and a run
is pinned to the lens version. `delta` refuses to compare two runs on different lens versions, because a change in a score
across changed anchors says nothing about the subject. To use a changed lens, bump `version` and `lens add` it again.

## The fields
- **`compares`**: the lens judges the subject against named comparators, each a snapshot attached to the run
  (`comparator add`, `brief --comparator NAME`). The shipped `compare` lens does this. A line about a comparator the run
  holds no snapshot of can only be `recalled` (`comparator_not_snapshotted` otherwise). Under such a lens a `seen` line must be
  `about` a comparator (`"about": "cmp:<name>"`) or cite a held comparator; a `seen` line that does neither is refused
  (`seen_compare_without_comparator`).
- **`asks_pick`**: the lens requires a forced choice. The verdict's `pick` names one option (an attachment label of the
  subject, or `cmp:<name>` for a comparator in the run) with a reason; `pick_missing`, `pick_unknown_option` and
  `pick_without_reason` refuse an incomplete one. `jurygraph pick RUN` reads them back per stratum. A persona "picks one and
  scores all" by writing one line per (dimension, `about`) for the options.
- **`evidence`**: the evidence classes a line under this lens may carry. `recall` allows only `recalled`; a line with another
  class is refused (`evidence_not_allowed`).
- **`standard`**: a pointer to the public standard the lens measures against (a design system, WCAG, a brand guide). Empty
  means the persona's own taste, and the brief says so. The `taste` lens is meant to be given one per project.

## The shipped lenses
| lens | asks | notes |
|---|---|---|
| `critical` | works, blocks, missing | does it do the job, what stops me, what is absent |
| `taste` | look, type, motion, voice | against the project's `standard`, or the persona's own taste if empty |
| `compare` | clarity, price, trust | `asks_pick`; against comparators held as snapshots |
| `recall` | first-read, would-return | `recalled` only; a cheap first impression from memory |

All four are 1 to 5 and are present in every new store.

## A fixture lens: `site`
`fixtures/lenses/site.toml` is a 1 to 10 `site` lens, kept as fixture data to show a different scale. It is **not seeded**:
load it yourself with `jurygraph lens add --file fixtures/lenses/site.toml`. It exists for the held-out check on the fixture
site and for tests, not as a fifth shipped lens.

## Proposing a lens
Open a feature request with the file, the question it asks that the four above do not, and one run that shows it separates
good from bad (`GOVERNANCE.md`). A contributed lens has anchors in words at every anchor score, a public and citable
`standard` if it measures against one, and a new version on every anchor change. A lens that only renames a shipped dimension
is declined.
