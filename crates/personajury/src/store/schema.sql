-- PersonaJury store, schema v4 (D17 adds run.effort and run.trimmed_json, D18 run.independence, D13 run.rule_class and run.quorum)
-- PersonaJury store, schema v3 (report §4; plan docs/plans/2026-10-23-v0-1-the-engine.md; D8; D9 adds run.draw_seed and run.override_json). Verdicts, picks and decisions are append-only.
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS persona (
  id TEXT NOT NULL, version INTEGER NOT NULL, name TEXT NOT NULL, tags_json TEXT NOT NULL, stake_rules TEXT NOT NULL,
  profile_json TEXT NOT NULL, origin TEXT NOT NULL, hash TEXT NOT NULL, created_at TEXT NOT NULL, PRIMARY KEY (id, version));
CREATE TABLE IF NOT EXISTS material (hash TEXT PRIMARY KEY, sources_json TEXT NOT NULL, brief_path TEXT NOT NULL, created_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS panel (
  name TEXT PRIMARY KEY, query TEXT NOT NULL, persona_ids_json TEXT NOT NULL DEFAULT '[]', hash TEXT, created_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS lens (
  name TEXT NOT NULL, version INTEGER NOT NULL, body_json TEXT NOT NULL, hash TEXT NOT NULL, created_at TEXT NOT NULL,
  PRIMARY KEY (name, version));
CREATE TABLE IF NOT EXISTS subject (
  name TEXT NOT NULL, version INTEGER NOT NULL, kind TEXT NOT NULL, is_comparator INTEGER NOT NULL, hash TEXT NOT NULL,
  created_at TEXT NOT NULL, PRIMARY KEY (name, version));
CREATE TABLE IF NOT EXISTS attachment (
  subject_name TEXT NOT NULL, subject_version INTEGER NOT NULL, label TEXT NOT NULL, media TEXT NOT NULL, hash TEXT NOT NULL,
  source_path TEXT NOT NULL, PRIMARY KEY (subject_name, subject_version, label),
  FOREIGN KEY (subject_name, subject_version) REFERENCES subject(name, version));
CREATE TABLE IF NOT EXISTS run (
  id INTEGER PRIMARY KEY, subject_name TEXT NOT NULL, subject_version INTEGER NOT NULL, subject_hash TEXT NOT NULL,
  panel_name TEXT NOT NULL, panel_hash TEXT NOT NULL, lens_name TEXT NOT NULL, lens_version INTEGER NOT NULL,
  model TEXT NOT NULL, spend INTEGER NOT NULL, stakes_json TEXT NOT NULL, personas_json TEXT NOT NULL,
  comparators_json TEXT NOT NULL, briefs_json TEXT NOT NULL, created_at TEXT NOT NULL,
  draw_seed TEXT, override_json TEXT, effort TEXT, trimmed_json TEXT, independence TEXT NOT NULL DEFAULT 'independent',
  rule_class TEXT, quorum TEXT);
CREATE TABLE IF NOT EXISTS verdict (
  id INTEGER PRIMARY KEY, run_id INTEGER NOT NULL REFERENCES run(id), batch INTEGER NOT NULL, persona_id TEXT NOT NULL,
  persona_version INTEGER NOT NULL, dimension TEXT NOT NULL, score INTEGER NOT NULL, anchor_met INTEGER NOT NULL,
  evidence TEXT NOT NULL, about TEXT, citations_json TEXT NOT NULL, citation_hash TEXT NOT NULL,
  blockers_json TEXT NOT NULL, note TEXT, created_at TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS idx_verdict_run ON verdict(run_id, persona_id, batch);
CREATE TABLE IF NOT EXISTS pick (
  id INTEGER PRIMARY KEY, run_id INTEGER NOT NULL REFERENCES run(id), batch INTEGER NOT NULL, persona_id TEXT NOT NULL,
  choice TEXT NOT NULL, reason TEXT NOT NULL, created_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS finding (
  id TEXT PRIMARY KEY, run_id INTEGER NOT NULL REFERENCES run(id), batch INTEGER NOT NULL, persona_id TEXT NOT NULL,
  subject_name TEXT NOT NULL, dimension TEXT NOT NULL, about TEXT, text TEXT NOT NULL, citation_hash TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS decision (
  id INTEGER PRIMARY KEY, finding_id TEXT NOT NULL REFERENCES finding(id), outcome TEXT NOT NULL CHECK (outcome IN ('fixed','dismissed','row')),
  why TEXT, row_ref TEXT, by TEXT NOT NULL, created_at TEXT NOT NULL, after_run INTEGER NOT NULL);
-- D12 (v0.3): what the agent reported a verdict cost; additive, so a v3 store gains it on open. Append-only.
CREATE TABLE IF NOT EXISTS usage (
  id INTEGER PRIMARY KEY, run_id INTEGER NOT NULL REFERENCES run(id), batch INTEGER NOT NULL, persona_id TEXT NOT NULL,
  input_tokens INTEGER NOT NULL, output_tokens INTEGER NOT NULL, created_at TEXT NOT NULL);
-- D19 (v0.4): who entered a human batch (`verdict --by-human`), beside the verdict rows of persona_id `human:<name>:<seat>`.
CREATE TABLE IF NOT EXISTS human_batch (
  id INTEGER PRIMARY KEY, run_id INTEGER NOT NULL REFERENCES run(id), batch INTEGER NOT NULL, persona_id TEXT NOT NULL,
  by_human TEXT NOT NULL, created_at TEXT NOT NULL);
CREATE TRIGGER IF NOT EXISTS human_batch_no_update BEFORE UPDATE ON human_batch BEGIN SELECT RAISE(ABORT, 'human_batch is append-only'); END;
CREATE TRIGGER IF NOT EXISTS human_batch_no_delete BEFORE DELETE ON human_batch BEGIN SELECT RAISE(ABORT, 'human_batch is append-only'); END;
-- append-only: verdicts, picks and decisions are never rewritten (report §4)
CREATE TRIGGER IF NOT EXISTS verdict_no_update BEFORE UPDATE ON verdict BEGIN SELECT RAISE(ABORT, 'verdict is append-only'); END;
CREATE TRIGGER IF NOT EXISTS verdict_no_delete BEFORE DELETE ON verdict BEGIN SELECT RAISE(ABORT, 'verdict is append-only'); END;
CREATE TRIGGER IF NOT EXISTS pick_no_update BEFORE UPDATE ON pick BEGIN SELECT RAISE(ABORT, 'pick is append-only'); END;
CREATE TRIGGER IF NOT EXISTS pick_no_delete BEFORE DELETE ON pick BEGIN SELECT RAISE(ABORT, 'pick is append-only'); END;
CREATE TRIGGER IF NOT EXISTS usage_no_update BEFORE UPDATE ON usage BEGIN SELECT RAISE(ABORT, 'usage is append-only'); END;
CREATE TRIGGER IF NOT EXISTS usage_no_delete BEFORE DELETE ON usage BEGIN SELECT RAISE(ABORT, 'usage is append-only'); END;
CREATE TRIGGER IF NOT EXISTS decision_no_update BEFORE UPDATE ON decision BEGIN SELECT RAISE(ABORT, 'decision is append-only'); END;
CREATE TRIGGER IF NOT EXISTS decision_no_delete BEFORE DELETE ON decision BEGIN SELECT RAISE(ABORT, 'decision is append-only'); END;
