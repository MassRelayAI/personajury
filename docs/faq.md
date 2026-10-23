<!-- shape follows comparegraph docs/faq.md -->
# FAQ

**Does JuryGraph call a model?** No. It composes briefs and ingests verdicts. Your agent runs the personas on its own model
and bill. There is no API key setting and no network call at brief or verdict time.

**Why was my verdict refused?** The engine refuses what has no evidence. See [`troubleshooting.md`](troubleshooting.md) for
each code.

**Why is there no overall score?** Because a bystander's reading and a primary user's reading answer different questions. The
matrix shows every score, the median (the lower middle value), min and max within each stratum, and never combines strata (design report section 2.3).

**What does a verdict line look like?** `{"dimension": "works", "about": "cmp:acme-pricing", "score": 2, "anchor_met": 1,
"evidence": "seen", "citations": [...], "blockers": [...], "note": "..."}`. `about` is optional (an attachment label of the
subject or `cmp:<name>`); one line per (dimension, `about`), and every dimension needs at least one. See
[`usage.md`](usage.md).

**What does a citation prove?** That the persona was given the attachment and named a node that exists, not that it looked or
judged well (README).

**Can I publish a result?** With its date, declared model (unverified), the four hashes and the per-stratum table, or it is
prose. See [`publishing-results.md`](publishing-results.md).

**What is `recalled`?** A line the persona wrote from the model's memory. It is shown as such. A comparison with no
comparator snapshot in the run can only be `recalled`.

**Can the jury check a live URL?** No. Subjects are attached files: an ARIA walk, screenshots, a diff, a document. Capture
the page yourself and attach it.

**Can it see colour and layout?** Only through screenshots, and it does not check image regions. A claim about contrast must
cite the screenshot.

**Are the personas real people?** No. They are composites, never identifiable real people (`PERSONAS.md`). The engine cannot tell whether a persona is realistic.

**Does the jury decide for me?** Only where `jurygraph.toml` says so, and never for money, permission, deploy, secret,
licence, public-claim or brand. A split or a primary-stratum blocker escalates one level; in v0.1 the agent applies that rule.

**How much does a run cost?** The model calls your agent makes. The engine reports each brief's byte size and nothing more;
no benchmark is published for v0.1.

**How do I add a lens?** `jurygraph lens add --file my-lens.toml`, with anchors in words at 1, 3 and 5 ([`lenses.md`](lenses.md); editing an anchor means a new version). To get it shipped in
the repository, see `GOVERNANCE.md`.
