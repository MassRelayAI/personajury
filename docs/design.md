# Design: the law of the tool

This page summarises the design report, "JuryGraph: our second OSS tool, the persona jury as an engine", dated 2026-10-23
(kept privately by the maintainer; decision D1). It adds nothing to it. Section numbers below are the report's.

| rule | what it says | report |
|---|---|---|
| review, not prediction | the subject is an artifact (a page's ARIA walk and screenshots, a diff, a design, a logo, a document, a menu, a price table); a run says what is wrong and compared with what, never what will happen | section 2.1 |
| evidence or it is labelled | each verdict line is `seen` (an attachment of the run, cited), `told` (text the brief supplied) or `recalled` (the model's memory); a comparison with no comparator snapshot cannot be `seen` | section 2.2 |
| strata, never one average | a persona's stake is `primary`, `secondary` or `bystander`, declared per run; scores are reported per stratum and the engine refuses a table that averages across them | section 2.3 |
| lenses, chosen and priced | a lens is a dimension set with anchors in words; four ship (`critical`, `taste`, `compare`, `recall`); a developer spends N personas on one lens | section 2.4 |
| the engine never calls a model | the agent runs the personas with its own subagents and bill; a CLI runner over an OpenAI-compatible endpoint may come later and is not v0.1 | section 2.5 |
| rounds diff | a run is pinned to the subject, panel, lens version and model; `delta` shows what moved; `check` marks a run stale when the subject changed | section 2.6 |
| the store | personas, panels, lenses, subjects, comparators, runs, verdicts, decisions; verdicts and decisions are append-only | section 4 |
| what the engine refuses | the list in section 5, each a test seen red first, with codes in the v0.1 plan | section 5 |
| not every change needs a jury | `should-run` classifies a diff against `jurygraph.toml` into `none`, `low`, `high`, `human`; findings each get one decision; a dismissed point cannot return without new evidence | section 10 |
| the human classes | money, permission, deploy, secret, licence, public-claim, brand; no persona quorum may decide them, and a `jurygraph.toml` that tries is refused | section 10 |

Build-time choices the report leaves open are in [`decisions.md`](decisions.md) (D3 to D5). What the engine cannot see (image
regions, whether a persona really read what it cites, colour and layout outside screenshots) is stated in the README.
