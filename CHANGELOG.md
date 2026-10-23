<!-- adapted from comparegraph CHANGELOG.md -->
# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (v0.3, D11 and D12)
- `jurygraph graph RUN... --svg [--out F]` and the MCP tool `jurygraph_graph`: the jury as one deterministic SVG (personas
  in three stratum columns, the dimensions they scored with a median badge per stratum, the findings coloured by decision;
  several runs draw each finding's life). `docs/assets/jury-graph.svg`, checked against a fresh render by a test.
- `check --history` lists each finding's life across runs (`data.lives`).
- `brief` answers `waves` (`--wave N`, default 30).
- A verdict may report `usage {input_tokens, output_tokens}` (`invalid_usage` when malformed); `matrix` and `doctor` show
  it beside the estimate, labelled as reported by the agent.
- The brief states the allowed `anchor_met` values per lens (the recorded brief hashes changed).
- `--comparator` takes a comma list.

### Changed (v0.3)
- A brief above 12 seats needs `--reason` with or without a rule and records `override_json` with `rule: null`.
- `brief` refuses a malformed `jurygraph.toml` it reads (`invalid_config`).

### Added (v0.2, D9 and D10)
- A stake per panel term (`3 builder=primary`) and a seeded draw: where a term's pool is bigger than its seats, the seats are
  drawn from the subject's name and the panel's, so round 2 of a subject keeps its bystanders and `delta` can land.
  `brief --draw HEX`, `panel resolve --subject S`; `run.draw_seed` records the seed.
- `brief --rule CLASS`: panel, lenses, spend and detail from `jurygraph.toml`, one run per lens. A `low` or `high` rule
  must name a `panel` (`rule_without_panel`) and lenses.
- Agent overrides on top of a rule (`--panel`, `--query`, `--spend`, `--lens`, `--detail`), each needing `--reason`
  (`override_without_reason`), recorded in `run.override_json` and shown by `matrix` (JSON `override`, a table header).
- Seat ceilings: 12, or 1000 with `--allow-large` on `panel add` and `brief`; above 1000 never (`panel_too_large`).
  Every brief answers `estimate {briefs, tokens}`, and `large: true` above 12 seats.
- Further refusals: `rule_panel_unknown`, `rule_lens_unknown`, `panel_term_empty`; `#L<n>` into a UTF-8 text attachment
  must name a line the file has (`citation_unresolved`).
- `should-run --subject S` and warnings for a panel or lens the store lacks; `persona generate` asks for the roles first.

### Fixed (review round 1)
- `--stake` and `--draw` under a rule need a reason; each term draws from its own stream; `brief --rule` refuses a subject
  under a human rule; `spend = 0` caps at 12; overflow in seat totals and the estimate is `panel_too_large`; only `L<n>`
  locators of a text file are checked; the matrix override header names the rule and the runs; a v2 store is refused
  before anything is written.

### Changed
- Store schema v3 (`run.draw_seed`, `run.override_json`); a v2 store is refused by its `schema_version`.
- `brief --stake` is optional when the panel's terms carry stakes; `--panel` is optional with `--rule` or `--query`.

## [0.1.0] - unreleased

### Added
- The store: personas, panels, lenses, subjects, comparators, runs, verdicts, picks and decisions in SQLite
  (`.jurygraph/store.db`); verdicts, picks and decisions are append-only.
- Verbs: `persona add|generate|list`, `panel add|resolve`, `lens add|list`, `subject add`, `comparator add`, `brief`,
  `verdict`, `matrix`, `pick`, `delta`, `check`, `decide`, `should-run`, `setup`, `doctor`, `mcp`.
- Four shipped lenses as data under `lenses/`: `critical`, `taste`, `compare`, `recall`.
- Deterministic, hashed briefs, one per persona per lens; `--spend N` and `--detail compact|full`.
- Verdicts refused without evidence: a `seen` line with no citation, a comparator with no snapshot in the run, a score off
  the lens scale, a stale subject, a persona with no stake, a `pick` with no reason; a matrix that averages across strata.
- `jurygraph.toml` trigger rules and `should-run`; a `human` class under a persona quorum is refused.
- The MCP server, the skills `running-a-jury` and `writing-a-persona`, the Claude Code plugin files, the install script.
- `fixtures/`: the Lantern Notes site with planted defects, ARIA walks, a comparator, seven personas and trigger rules.
- Verdict lines may be `about` one option (an attachment label or `cmp:<name>`), only under a lens that picks or compares;
  one line per (dimension, about); further refusals `duplicate_dimension`, `missing_dimension`, `unknown_about`,
  `seen_compare_without_comparator`, `bottom_score_without_blocker`, `dismissed_point_repeated` (D6).
- The matrix median is the lower middle value of a stratum; nothing is averaged. `check` fails while a finding has no
  decision and lists `fixed_unverified`.
- `fixtures/run/`: a recorded seven-persona jury (14 verdicts, 26 findings, the foreman's decisions) that
  `tests/fixture_replay.rs` replays byte for byte; `fixtures/lenses/site.toml` (1–10) for the held-out detector.
- After the final review (D8, store schema v2): `check` reads findings from every batch and fails a fixed finding raised
  again (`fix_not_landed`); `delta` lands only when every cell is judged again and no stratum's blockers rose; a newer
  subject version makes older runs stale; `matrix` refuses mixed subjects, carries evidence counts per cell and marks
  `recalled`/`told` scores in the table; names are validated before any path; human-class names are normalised;
  git-quoted diff paths are read and an unparseable header is refused (`invalid_diff`); `about` joins the finding id;
  `pick_not_asked`; `lens add` validates the lens.
- Governance and community files (GOVERNANCE, MAINTAINERS, SUPPORT, NOTICE, CITATION.cff, issue forms, PERSONAS.md),
  harness manifests for Codex, Cursor and Gemini, and `docs/` (quickstart, usage, commands, lenses, testing, reports).
