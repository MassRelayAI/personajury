<!-- adapted from comparegraph CHANGELOG.md -->
# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0] - unreleased
### Added (v0.6-A, D29-D31)
- Traits as data: 49 traits in 14 families and eight trait sets ship embedded (`traits/`, `trait-sets/`); `trait install | add | list | show`
  and `trait set add | list`; every anchor is our own words, no questionnaire item is copied (D29).
- Schema v5 (one migration for the v0.6 lanes, idempotent): persona `traits` and `serves`, panel cover, run traits and seats, subject
  audience, and the `trait` and `trait_set` tables (D29).
- The persona key `traits` (at most 8) with `unknown_trait`, `trait_value_off_scale`, `trait_option_unknown`, `traits_too_many` and the
  refused list (`refused_trait_id`: typologies, protected attributes, the persona's own beauty, fame or outsider status) (D29).
- `brief --traits <set>|all|off` and a rule's `trait_set`: the block "How this reader tends to decide (told)", rendered as "you tend to";
  `trait_set_incomplete`; `persona generate --traits` (D31).
- Panels: `[trait>=4]` filters on a term, `panel add --cover`, `coverage` (one-way, pairs, balance, collinear, confounded) and the
  cover-aware draw (D30).
- `matrix --slice TRAIT` and `summary --slice TRAIT`: groups per stratum, counted, never averaged (D31).
- `brief --seats auto`, persona `serves`, `subject add --audience`: the personas that serve the audience plus one control.
- The MCP tool `personajury_trait_list`; `traits`, `seats` on brief, `slice` on matrix and summary.

## [0.5.1] - 2026-10-24
### Added
- The fairness rules in the skills (short rules with a one-line reason each) and the naming guidance in
  `docs/playbooks/pick-a-name-or-logo.md`; Dependabot proposes no major bumps.

### Renamed
- The brief rename to JuryPersona (D27) and back to PersonaJury (D28), both before any public use: nothing was public under either
  name, so the name is PersonaJury as in 0.5.0, with no shim and no migration.

## [0.5.0] - unreleased
Nothing was ever released: this one entry holds the whole history, newest work first, and the sub-headings keep the versions the
work was built in (0.5, 0.4, 0.3, 0.2, 0.1).

### Renamed
- Renamed from JuryGraph before the first release: the crate, package and binary `personajury`, `personajury.toml`, the store
  directory `.personajury/`, the MCP tools `personajury_*`, the environment variables `PERSONAJURY_*`, the repository
  `massrelay-ai/personajury`, the release assets `personajury-<target>`. No compatibility shims, no migration of old stores (D25).
  The graph SVG is `docs/assets/personajury-graph.svg`, titled "PersonaJury · <subject>". The version of every manifest is 0.5.0.

### Fixed (the v0.5 review)
- `apply-check --latest` judges every subject that has runs and answers the worst (`human` > `escalate` > `advise` > `apply`), with each
  subject's answer under `subjects`; `--subject S` narrows it. It no longer answers `apply` for the newest subject while another escalates.
- The Action fails closed: an engine error, a missing binary or output that is not JSON answers `unavailable`, which fails the job unless
  `fail-on` leaves it out; default `fail-on: human,escalate,unavailable`; `--rule` is passed when `should-run` matched exactly one rule;
  file names are markdown-escaped in the step summary; `version` defaults to `v0.5.0`.
- `summary`: `report_dir` and MCP `md`/`html` paths must stay inside the root (`invalid_report_dir`); printed text is terminal-safe;
  quotes are cut only between words; the first sentence is plain words; quoted blockers rotate across jurors.
- `apply-check --calibration` refuses a file for another subject, lens, lens version or run (`calibration_mismatch`).
- CSV locators refuse lowercase and `RC1`; a quote inside an unquoted field is literal.

### Added (v0.5, D20 to D24)
- `personajury calibrate JURY_RUN HUMAN_RUN [--out F]` and `apply-check --calibration F`: the jury's rank order against a human run, per
  stratum, never pooled; an uncalibrated jury advises (`uncalibrated`) (D20). `apply-check --latest` judges the subject of the newest run.
- CSV cell locators `<label>#R<row>C<col>` on `.csv` text attachments, with an RFC 4180 reader in `domain/csv.rs` (D21).
- `action.yml`, a composite GitHub Action (`scripts/action-run.sh`, `docs/ci.md`): `should-run` on the PR diff, `check` and `apply-check`
  with a committed store, outputs `quorum`, `spend`, `effort`, `answer`, `check`, `fail-on`, unmatched paths in plain words (D22).
- `docs/agent-sdk.md` and `examples/agent-sdk/run_jury.py`: the Claude Agent SDK recipe, one context per brief (D23).
- `personajury summary RUN... [--html F] [--md F] [--save] [--width N]` and the MCP tool `personajury_summary` (eight tools): sentences, the numbers
  "for checking", then Next; no bars; `report_dir` in `personajury.toml` (D24). Golden files `fixtures/run/summary.txt` and `summary.md`.
- `fixtures/lenses/tradeoffs.toml` and `docs/playbooks/a-technical-decision.md`.
- Docs: "Display is free, the verdict is not", "What goes to a jury", "Where results live"; a three-question rule in the skill.

### 0.4 work
The sections below were built as 0.4, 0.3, 0.2 and 0.1 and ship together in 0.5.0.

### Added (v0.4, D13 to D19)
- `personajury apply-check RUN... [--rule CLASS]` and the MCP tool `personajury_apply_check` (seven tools): `apply | escalate | human |
  advise` with the reasons as data (D13). `matrix` answers `splits`, the cells on which two strata's `anchor_met` ranges do not overlap.
- `personajury lens generate --from PATH [--dimensions N]`: the brief that asks an agent to draft a lens from a standard (D14).
  Three example lenses ship as fixture data (consulting deck, investor pitch, financial model).
- A model per panel term (`3 builder=primary@claude-sonnet-5-5`), stated in every brief of that run; `matrix` answers `models` and
  `models_distinct` (D15).
- `.mcpb` bundles for Claude Desktop (`packaging/mcpb`, `scripts/build-mcpb.sh`), `mcp.json` for Codex and Cursor, `mcpServers` in the
  Gemini extension, `packages` in `server.json`, a `publish-registry` release job, `tests/manifests.rs` and a no-listener guard (D16).
- Effort: `effort = "low|medium|high|max"` in `personajury.toml` and `brief --effort`, `budget_tokens`, `over_budget`, `trimmed`; `doctor`
  shows the effort and the tokens reported over 7 days (D17).
- `brief --independence independent|sequential`, shown by `matrix`, `delta` and `apply-check` (D18).
- Experts: a persona `credential`, a lens `standard` in the matrix header, `verdict --by-human "<name, credential>"` (D19).
- `fixtures/material/usecases.csv`: twelve invented rows for Lantern Notes, the public example for `persona generate`.

### Changed (v0.4)
- **Store schema v4** (D17, D18, D19): a v3 store upgrades in place on open.
- `apply-check` answers `apply` only when every condition is proven (D13 as ruled after the v0.4 review): new reasons
  `incomplete_jury`, `quorum_unknown`, `rule_below_sources`, `overridden_down`, `stale_run`, `delta_unavailable` (advise);
  `check_failing` is renamed `check_failed` and now covers every run of the subject; blockers and splits are read per run. The
  quorum is the highest of the run's recorded quorum (`run.rule_class`, `run.quorum`, schema v4), the `--rule` named and the sources';
  with none it is unknown (`quorum: null`), never an assumed `low`.
- `brief --rule` whose quorum is below the sources' is an override (`quorum`) that needs `--reason`. The medium and high seat caps
  keep one seat per stratum before filling primary first (D17).
- `decide` on a finding a human batch raised needs `--by-human "<name>"` (`human_decision_required`); `--by` is now optional when
  `--by-human` is given (D19).
- Under a rule, `brief` is planned by effort (default `medium`: at most 7 seats). Set `effort = "high"` or `"max"` for more.

### Deferred (to v0.5)
- Calibration (`calibrate`), CSV cell locators, the GitHub Action, the Agent SDK recipe, the marketplace listings.

### Added (v0.3, D11 and D12)
- `personajury graph RUN... --svg [--out F]` and the MCP tool `personajury_graph`: the jury as one deterministic SVG (personas
  in three stratum columns, the dimensions they scored with a median badge per stratum, the findings coloured by decision;
  several runs draw each finding's life). `docs/assets/personajury-graph.svg`, checked against a fresh render by a test.
- `check --history` lists each finding's life across runs (`data.lives`).
- `brief` answers `waves` (`--wave N`, default 30).
- A verdict may report `usage {input_tokens, output_tokens}` (`invalid_usage` when malformed); `matrix` and `doctor` show
  it beside the estimate, labelled as reported by the agent.
- The brief states the allowed `anchor_met` values per lens (the recorded brief hashes changed).
- `--comparator` takes a comma list.

### Changed (v0.3)
- A brief above 12 seats needs `--reason` with or without a rule and records `override_json` with `rule: null`.
- `brief` refuses a malformed `personajury.toml` it reads (`invalid_config`).

### Added (v0.2, D9 and D10)
- A stake per panel term (`3 builder=primary`) and a seeded draw: where a term's pool is bigger than its seats, the seats are
  drawn from the subject's name and the panel's, so round 2 of a subject keeps its bystanders and `delta` can land.
  `brief --draw HEX`, `panel resolve --subject S`; `run.draw_seed` records the seed.
- `brief --rule CLASS`: panel, lenses, spend and detail from `personajury.toml`, one run per lens. A `low` or `high` rule
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

### Added (v0.1)
- The store: personas, panels, lenses, subjects, comparators, runs, verdicts, picks and decisions in SQLite
  (`.personajury/store.db`); verdicts, picks and decisions are append-only.
- Verbs: `persona add|generate|list`, `panel add|resolve`, `lens add|list`, `subject add`, `comparator add`, `brief`,
  `verdict`, `matrix`, `pick`, `delta`, `check`, `decide`, `should-run`, `setup`, `doctor`, `mcp`.
- Four shipped lenses as data under `lenses/`: `critical`, `taste`, `compare`, `recall`.
- Deterministic, hashed briefs, one per persona per lens; `--spend N` and `--detail compact|full`.
- Verdicts refused without evidence: a `seen` line with no citation, a comparator with no snapshot in the run, a score off
  the lens scale, a stale subject, a persona with no stake, a `pick` with no reason; a matrix that averages across strata.
- `personajury.toml` trigger rules and `should-run`; a `human` class under a persona quorum is refused.
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
