# Docs: where each kind of truth lives
<!-- shape follows comparegraph docs/reports/README.md and the superpowers docs layout (structure only) -->

| kind | where | rule |
|---|---|---|
| the law of the tool | [`design.md`](design.md) | a summary of the design report (evidence classes, strata, refusals, human classes) with section pointers; adds nothing to it |
| what to build, step by step | [`plans/`](plans/README.md) | one dated plan per release; a plan is not edited to match the code afterwards, a decision is written instead |
| why a choice was made | [`decisions.md`](decisions.md) | numbered D1...; a change to the law needs an entry and a red-first test (`../GOVERNANCE.md`) |
| a measurement or a verdict | [`reports/`](reports/README.md) | a dated file, written before anyone claims the number |
| who decides, and what only a decision may change | [`../GOVERNANCE.md`](../GOVERNANCE.md) | the law of the tool, model and provider neutrality, the human classes |
| personas | [`../PERSONAS.md`](../PERSONAS.md) | composites, never identifiable real people; generated ones record their material hash |
| traits | [`traits.md`](traits.md) | the catalog of tendencies a persona may hold, embedded and installed; the brief block, cover, quotas and slices; what is refused |
| lenses | [`lenses.md`](lenses.md) | a versioned standard; anchors in words, never edited in place; the four shipped and the fixture `site` lens |
| publishing a result | [`publishing-results.md`](publishing-results.md) | date, declared model, four hashes and the per-stratum table, or it is prose |
| privacy and vulnerabilities | [`../SECURITY.md`](../SECURITY.md) | the engine sends nothing; attachments stay in `.personajury/blobs` |
| how to use it | [`usage.md`](usage.md), [`commands.md`](commands.md), [`mcp-tools.md`](mcp-tools.md), [`faq.md`](faq.md), [`troubleshooting.md`](troubleshooting.md), [`../README.md`](../README.md) | describe the v0.1 contract in the plan until the engine is built |
| what comes next | [`roadmap.md`](roadmap.md) | only what the design names; each item says "not built" |
| how it is tested | [`testing.md`](testing.md) | every refusal is a test seen red first |
| how it is released | [`dev/release.md`](dev/release.md) | maintainers only |
| licences of adapted files | [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) | one row per adapted file |

## The report rule
A measurement, a bake-off, a proof or a verdict ends in a dated file in `docs/reports/` before anyone claims it in a README, a
pull request or an issue. The front matter has four keys: `date`, `valid_until`, `verdict`, `status`. A report past its
`valid_until` is re-run or marked stale, never silently kept. Numbers in the README come only from these files; at v0.1 none
exist, which is why the README says no benchmarks are published.

## Evidence for the community files
The set of repository files (issue forms, `SECURITY.md`, `CODEOWNERS`, `.editorconfig`, `NOTICE`, `CITATION.cff`,
`SUPPORT.md`, `MAINTAINERS.md`, `GOVERNANCE.md`, a releasing document, a roadmap) follows a survey of 175 cloned OSS
repositories, 2026-10-23. Counts of repositories that ship each: issue templates 114, SECURITY 97, CODEOWNERS 59,
.editorconfig 33, NOTICE 16, CITATION.cff 16, SUPPORT 8, MAINTAINERS 7, GOVERNANCE 6, RELEASING 6, ROADMAP 6, ADOPTERS 4.
We ship all of these except ADOPTERS: there are no adopters to list.
