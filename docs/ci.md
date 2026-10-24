# PersonaJury in CI

The Action classifies a pull request against `personajury.toml` and, if you commit your store, runs `check` and `apply-check`.
It runs **no model, needs no secret and no network** beyond the binary download (checksum verified against the release's
`SHA256SUMS`). The jury itself is run by your agent, on your model; CI is where its answer becomes binding.

```yaml
# .github/workflows/personajury.yml
name: personajury
on: pull_request
jobs:
  jury:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }          # the diff needs the base
      - id: jg
        uses: massrelay-ai/personajury@v0.6.0
        with:
          config: personajury.toml
          store: .                          # optional: a root holding a committed .personajury
          fail-on: human,escalate,unavailable   # the default; a failing check always fails the job
      - run: echo "quorum ${{ steps.jg.outputs.quorum }}, answer ${{ steps.jg.outputs.answer }}"
```

| input | default | meaning |
|---|---|---|
| `config` | `personajury.toml` | the rules |
| `base` | the pull request base | what the diff is taken against |
| `store` | none | a project root with a committed `.personajury`; without it only `should-run` runs |
| `version` | this Action's release (`v0.6.0`) | the release to install; `latest` follows the newest release |
| `fail-on` | `human,escalate,unavailable` | answers that fail the job: `human`, `escalate`, `advise`, `unavailable` |

Outputs: `quorum` (`none | low | high | human`, or `unavailable`), `spend` (JSON), `effort`, `answer` (`apply | escalate |
human | advise | unavailable`, or `none` without a store), `check` (`ok | failed | unavailable | skipped`). File names in the
step summary are markdown-escaped: a pull request from a fork names its own files.

**The Action fails closed (D13).** `unavailable` means nothing was proven: the binary is missing, `jq` is missing,
`should-run`, `check` or `apply-check` returned an error (a broken `personajury.toml`, a store that cannot be opened, a rule
`apply-check` cannot find), or the output was not JSON. `unavailable` fails the job by default. Only a `fail-on` you set
without `unavailable` lets it through, and the step summary still says the answer is unavailable and why. `check: failed`
(findings with no decision, or a fix that did not land) always fails the job; `check: unavailable` is an error, not a
failing check, and makes the answer `unavailable`. The step summary shows the table and says in words which files
matched no rule: "N files matched no rule and got no jury: ...; add a rule if one of them is something people see."

`human` in `fail-on` also fails a change whose `should-run` quorum is `human` even with no store: a person decides money,
permission, deploy, secret, licence, public-claim and brand. With a store, `apply-check --latest` judges the latest round of
every subject that has runs, each on its own, and answers the worst (`human` > `escalate` > `advise` > `apply`); the answer
lists each subject's own answer and reasons under `subjects`. When `should-run` matched exactly one rule, its class is passed
as `--rule`, so the diff's quorum reaches apply-check; the store's root must then hold the same `personajury.toml` (normally the
store is the repository root, `store: .`). Commit the store only if CI reads it (it is append-only and not meant for git
otherwise).

The script is `scripts/action-run.sh` (needs `jq`, present on GitHub's runners); run it by hand with `JG_BIN`, `JG_BASE` and
`JG_STORE` set to see what CI will say.
