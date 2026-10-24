# Publishing results

A jury result is an opinion of invented personas, produced by a model, about a file as it was on one day. Say that when you
publish it.

## A verdict about a comparator is not a claim about the comparator
A line that scores a named third-party product is a **fictional persona's opinion on a dated, hashed snapshot**. It is not a
fact about the product, its price, its quality or its company. The snapshot was attached by whoever ran the jury; the product
may have changed since, and the persona may have misread it. Anyone publishing a result that names a third party carries that
sentence with it, with the snapshot date and the evidence class (`seen`, `told`, `recalled`) of every line about it. A
`recalled` line is the model's memory and says nothing about today's page.

## What a published result carries
A result shared outside the project (a README, a blog post, a pull request, an issue) must carry all of these, or it is prose
and must be labelled as such:
1. the **date** of the run;
2. the **model** as declared with `--model`, marked **unverified**: the engine never calls a model and cannot check what ran;
3. the **subject** and the hash of its attachments;
4. the **panel**, **lens** (with its version) and **brief** hashes;
5. the **independence** of the run (`independence:` in the matrix header, `independent` or `sequential`): a sequential run was
   judged by one context that read every brief, and it is weaker evidence than one context per brief;
6. the **per-stratum table**: primary, secondary and bystander side by side, with every score, median, min and max, from
   `personajury matrix RUN --table`.

**Do not publish an average across strata.** The engine refuses to produce one (`strata_average_refused`); do not compute one
by hand, and do not quote a single "score" for a subject. The median is the lower middle value of a stratum, an integer; there
is no mean anywhere.

## Numbers in this repository
A figure in a README or a doc comes only from a dated file in `docs/reports/` with its front matter (`date`, `valid_until`,
`verdict`, `status`); a claim of the form "model X scores Y" needs a checked-in run behind it (`GOVERNANCE.md`).

## What a citation proves, here
A matrix is meaningful only with the hashes beside it: a citation shows the persona was given an attachment and named a node
that exists, not that it looked or judged well (README). The subject is what the submitter attached.

## Personal data
Walks, screenshots and diffs can hold other people's data. Check what you publish with the result (`SECURITY.md`, Privacy).
