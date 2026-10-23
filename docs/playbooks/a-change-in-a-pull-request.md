<!-- sources: fixtures/jurygraph.toml (the rules); docs/commands.md (should-run); README.md "Not every change needs a jury" (quorum table, human classes); docs/reports/2026-10-24-how-far-can-a-jury-go.md §4 (when a change applies itself); docs/roadmap.md (CI runner not built). v0.4 items (effort, apply-check) are described from the v0.4 plan and may not be in your build yet. -->

# Check a change in a pull request

Most changes need no jury. A typo fix in the tests does not. A new pricing page does. JuryGraph lets you write that down
once, as rules, and then asks the rules on every change.

## 1. Write the rules once

`jurygraph.toml` lists paths and what they need. The example in [`fixtures/jurygraph.toml`](../../fixtures/jurygraph.toml)
says:
- **the site and pricing pages:** the full panel, every lens (`high`);
- **copy only:** one persona, one cheap lens (`low`);
- **internals and tests:** no jury; tests decide (`none`);
- **money, permission, deploy, secret, licence, public claim, brand:** a person decides (`human`).

The engine refuses a rules file that lets a jury decide a human class.

## 2. Ask: does this change need a jury?

```bash
git diff main > change.diff
jurygraph should-run --diff change.diff
```

The answer is one word, `none`, `low`, `high` or `human`, with the rule that matched and how many personas it wants.
- `none`: merge on tests.
- `low` or `high`: run a jury on what changed (screenshots of the changed page, the changed doc).
- `human`: ask a person. A jury may advise, but cannot decide.

## 3. Say how much you want to spend (v0.4)

> **v0.4.** `effort` may not be in your build yet.

One word, the same words as your agent's effort setting: `low`, `medium`, `high` or `max`. Set it in `jurygraph.toml`
(`effort = "medium"` is the default), or per run with `jurygraph brief --effort low`. You can also set a hard cap,
`budget_tokens`.

| effort | personas | lenses | detail |
|---|---|---|---|
| low | 3, one per group, users first | the rule's first | compact |
| medium | the rule's, at most 7 | the rule's | compact |
| high | the rule's, at most 12 | the rule's | the rule's |
| max | the rule's, more with a reason | the rule's | the rule's |

**The rule says what a change needs. Effort says what you will spend. The run gets the smaller.** Every cut is listed. If
your effort is below what the rule needs, the jury still runs, but it may only advise.

## 4. Ask: may the agent apply the fixes? (v0.4)

> **v0.4.** `apply-check` may not be in your build yet. Until then, your agent applies the same rule by hand
> ([`running-a-jury`](../../skills/running-a-jury)).

After the jury and the fixes, one command gives one answer:

```bash
jurygraph apply-check <run>...
```

| answer | when | what happens |
|---|---|---|
| `apply` | no blocker from the people who would use it, the groups agree, `check` passes, `delta` landed | the agent applies the fixes |
| `escalate` | a blocker from a primary persona, two groups split, or a fix that did not land | a bigger jury, one level up; the answer names it |
| `human` | the change touches a human class | a person decides |
| `advise` | the jury was cheaper than the rule asked, or the jurors did not answer independently | the jury's view is advice only |

Two groups **split** when their score ranges do not overlap at all. A split is not averaged away; it goes up.

The command never applies anything itself. It gives a verdict, with its reasons, and the agent obeys it.

## 5. In CI, later

Today the jury runs inside your agent, on your machine, with your model and your bill. The engine calls no model and
holds no key.

A runner for CI is on the [roadmap](../roadmap.md), not built. Until then, you can still run `should-run` in CI. It needs
no model. Fail the build when it says `high` or `human` and the pull request has no jury run or person's sign-off
attached.

## What a jury on a diff will not do

- **Replace tests.** A jury reads what a person sees. Tests check what the code does.
- **Judge what it cannot see.** Give it the changed page as screenshots and a walk, not only the diff.
- **Decide human classes.** Money, permissions, deploys, secrets, licences, public claims and brand stay with a person.
