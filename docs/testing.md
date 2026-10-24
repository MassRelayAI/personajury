<!-- shape follows superpowers docs/testing.md (structure only) -->
# Testing

```
cargo test -p jurypersona
cargo clippy -p jurypersona --tests -- -D warnings
```
Tests run through the cargo runner in `.cargo/config.toml` (`tools/pidns-run.sh`). Tests that use `setup --apply` run against
a temporary HOME only.

## Every refusal is a test seen red first
The refusals R1 to R17 in the v0.1 plan each name their test and their error code. The order of work is fixed:
1. write the test for the refusal;
2. run it and watch it fail for the right reason (the message you expect, not a compile error);
3. write the refusal;
4. run it green.

A guard that has never been seen to fail is not known to work. A check that passes while printing nothing has not run.

## How to see a guard fail
- **The no-network guard.** `tests/guards.rs::no_network_crate_in_lock` reads `Cargo.lock`. In a scratch copy of the
  repository (never the real lock), add a `[[package]]` entry named after an HTTP client crate, run the test, and watch it go
  red; delete the scratch copy. `deny.toml` bans the same crates in CI (`cargo deny --locked check`).
- **A refusal.** In a scratch copy, break one: for example make the `seen`-without-citation check in `domain/refuse.rs`
  accept an empty citation list. `domain::refuse::tests::seen_line_without_citation_is_refused` and
  `tests/refusals_judging.rs::verdict_seen_without_citation` must go red. Restore it and they go green.
- **The key guard.** `tests/guards.rs::no_secret_env_read_in_src` goes red if source reads an environment variable that
  names a key or token.
- **Append-only.** `tests/store.rs::verdict_is_append_only` goes red if the SQLite trigger on `verdict` is dropped.

## Where the tests live
All under `crates/jurypersona/tests/`:
| file | covers |
|---|---|
| `store.rs` | the schema, versioning of personas, lenses and subjects, append-only verdicts, picks and decisions |
| `authoring.rs`, `refusals_authoring.rs` | personas, panels, lenses, subjects, briefs; the refusals at brief time (no stake, `--spend` above the panel) |
| `judging.rs`, `refusals_judging.rs` | verdicts, matrix, pick, delta, check, decide, `should-run`; every refusal a verdict or a table can trip, dismissed points, `check_failed` |
| `guards.rs` | no network crate in `Cargo.lock`, no key read from the environment in `src/` |
| `mcp.rs`, `setup.rs`, `doctor.rs` | the MCP server, `setup --apply` against a temporary HOME, `doctor` |
| `fixture_replay.rs` | the fixture run replayed byte for byte (next section); written after this page, in plan Task 7 |

## The fixture replay
The Lantern Notes site in `fixtures/` carries planted defects (the answer key in `fixtures/README.md`). A recorded run on it
is checked in as a matrix. `tests/fixture_replay.rs` replays the briefs and verdicts from the repository and compares the
resulting matrix with the checked-in one byte for byte; any difference fails. This is the contract: a change to a brief, a
hash or a table changes the bytes and must be a deliberate edit of the checked-in matrix in the same pull request. `tests/fixture_replay.rs`
is the contract the fixture run writes next (plan Task 7): until it exists, this section describes what it will do.

## The held-out check
A jury run on the fixture site is compared with the answer key: do the findings include the planted defects? The result
goes in `docs/reports/` with its front matter before it is claimed anywhere.
