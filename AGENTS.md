<!-- adapted from comparegraph AGENTS.md -->
# AGENTS.md - contributing to JuryGraph

A Rust CLI + MCP server that runs a persona jury as an engine: personas, panels, lenses, subjects and comparators in a
SQLite store, deterministic briefs, verdicts refused without evidence, scores per stratum. Users' skills live in `skills/`;
this file is for people and agents changing the engine.

## Layout and rings (dependencies flow inward)
```
crates/jurygraph/src/
  domain/       the standard: types, hashes, panel query, citations, the refusals, matrix, delta, trigger rules - pure, no I/O
  store/        SQLite at <root>/.jurygraph/store.db; verdicts, picks and decisions append-only (triggers)
  application/  one module per verb: persona, panel, lens, subject, brief, verdict, matrix, pick, delta, check, decide,
                should-run, setup, doctor
  adapters/     cli (clap, one-shot), mcp (stdio), the envelope - thin
lenses/   the four shipped lenses as data      skills/  the agent's method      fixtures/  the fixture site and its recorded run
docs/     plans, decisions, reports, quickstart
```
A module never imports outward. `domain` has no I/O; `adapters` are thin.

## Invariants (report §2, §5, §10 — the law of the tool)
- **The engine never calls a model.** No HTTP client in the tree (`deny.toml` bans them; `tests/guards.rs` checks the lock),
  no env var read for a key or token, no network fetch at brief or verdict time. Subjects are attached files.
- **Evidence or it is labelled.** `seen` lines cite an attachment of the run; a comparator with no snapshot in the run can
  only be `recalled`.
- **Strata, never one average.** Scores are reported per stratum (primary, secondary, bystander); nothing averages across.
- Verdicts, picks and decisions are append-only; the latest batch wins; `--history` shows all.
- Determinism: same inputs → byte-identical brief and the same hash.
- Every refusal is a test that was seen red first.

## Commands
```
cargo test -p jurygraph
cargo clippy -p jurygraph --tests -- -D warnings
```
Toolchain pinned in `rust-toolchain.toml`. Tests that use `--apply` run against a temp HOME only.

## Never
- A model call, a network call, an API key or token anywhere in the engine.
- A dependency added, or `Cargo.lock` / `store/schema.sql` edited, without a decision in `docs/decisions.md`.
- `cargo fmt` over the crate: format only files you wrote (`rustfmt <file>`).
- `git add -A`, `--no-verify`, a push or a remote.

## Where things are written down
Plans `docs/plans/`, decisions `docs/decisions.md` (D1…), measurements `docs/reports/`. Conventional commits; stage named
paths.
