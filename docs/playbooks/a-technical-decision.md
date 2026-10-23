# Choose between three libraries (or two API shapes)

**The job:** the team must pick a queue library, or decide between two shapes for a public API. A benchmark cannot settle it,
and everyone has a favourite. A jury will not tell you which one is correct; it tells you who is comfortable with which
trade-off, and where they disagree.

**Correctness stays with tests.** Whether the library gives the right answer is for a test or a benchmark. The jury weighs what
tests cannot: fit, risk, cost to change later, running it at 3 a.m., and the learning curve.

## 1. The subject is the options, as files
One subject, one attachment per option, each a plain text file you assemble by hand or a script copies out:
`option-a.md` (README excerpt, a usage snippet that does our real job, the licence line, the release cadence, the open issue count),
`option-b.md`, `option-c.md`. For two API shapes: the signatures and a call site for each. Jurors cite lines with `#L<n>`
(`option-b.md#L12`), so every claim points at text the whole panel can read.

```
jurygraph lens add --file fixtures/lenses/tradeoffs.toml
jurygraph subject add queue-choice --kind document --attach option-a.md option-b.md option-c.md
```

## 2. The panel
| stratum | who sits | why |
|---|---|---|
| primary | the maintainer, an SRE | they live with it every day |
| secondary | a security reviewer, a new hire | they are affected by it, and see different risks |
| bystander | a finance approver | the cost of a licence or a hosted service |

Briefed with the tradeoffs lens, each juror scores every option on five dimensions (a line per option with `about`) and picks one.
```
jurygraph brief --subject queue-choice --panel tech-choice --lens tradeoffs --model <model>
```

## 3. Read the disagreement
`jurygraph summary` leads with who won and by how many in each stratum, then the numbers for checking. The per-stratum
disagreement is the finding: if the maintainers pick B and the SRE pair picks A, that is the conversation, and it is cheaper to
have it now. A split on `risk` between security and the new hire says which fear is real.

## 4. An expert overrules by a recorded decide
If your most senior engineer knows the jury is wrong about an option, they say so on the record: a blocker the jury raised is closed with
`jurygraph decide <finding> dismissed --why "<the reason>" --by <name>`, and the choice is theirs. The jury advises; the decision and
its reason stay in the store.

## What it cannot do
It cannot measure speed, find a bug, or read the library's source for you. Attach the benchmark's result as a file if you have one; the
jurors will cite it. Run the tests for correctness first.
