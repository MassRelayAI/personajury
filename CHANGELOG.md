<!-- adapted from comparegraph CHANGELOG.md -->
# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
