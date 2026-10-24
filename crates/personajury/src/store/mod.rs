//! adapted from comparegraph crates/comparegraph/src/store/mod.rs (error type, now_iso, open/open_existing)
//! The SQLite store at `<root>/.personajury/store.db`. Verdicts, picks and decisions are append-only (triggers in schema.sql).
use crate::domain::traits::{TraitDef, TraitSetDef};
use crate::domain::*;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const SCHEMA: &str = include_str!("schema.sql");
pub const SCHEMA_VERSION: i64 = 5;
/// The columns schema v4 added to `run`; a v3 store (or an early v4 one) gains them on open (idempotent
/// `ALTER TABLE`, D17, D18, D13: the rule class a run was briefed under and its effective quorum).
const V4_RUN_COLUMNS: [(&str, &str); 5] = [
    ("effort", "TEXT"),
    ("trimmed_json", "TEXT"),
    ("independence", "TEXT NOT NULL DEFAULT 'independent'"),
    ("rule_class", "TEXT"),
    ("quorum", "TEXT"),
];
/// The columns schema v5 added to existing tables (idempotent `ALTER TABLE`, checked with `pragma_table_info`):
/// (table, column, declaration). D29 persona traits and `serves`; D30 panel cover; D31 run traits; `--seats auto` run seats
/// and subject audience.
/// A stored trait's latest version: the definition, its origin, its hash and the reason a forced install recorded.
pub type TraitRow = (TraitDef, String, String, Option<String>);
const V5_COLUMNS: [(&str, &str, &str); 10] = [
    ("trait", "reason", "TEXT"),
    ("persona", "traits_json", "TEXT"),
    ("persona", "serves_json", "TEXT"),
    ("panel", "cover_json", "TEXT"),
    ("run", "traits_json", "TEXT"),
    ("run", "seats_json", "TEXT"),
    ("subject", "audience_json", "TEXT"),
    // v0.6-C (D32, D33, D34)
    ("run", "options_mode", "TEXT"),
    ("verdict", "phase", "TEXT"),
    ("pick", "about", "TEXT"),
];
pub const DIR: &str = ".personajury";
pub const DB: &str = "store.db";

/// The four shipped lenses, seeded into a store that has no lens of that name (plan Task 2).
pub const SHIPPED_LENSES: [(&str, &str); 4] = [
    ("critical", include_str!("../../../../lenses/critical.toml")),
    ("taste", include_str!("../../../../lenses/taste.toml")),
    ("compare", include_str!("../../../../lenses/compare.toml")),
    ("recall", include_str!("../../../../lenses/recall.toml")),
];

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("cannot use {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot open {path}: {source}")]
    Open {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("schema: {0}")]
    Schema(String),
    #[error("{0}")]
    NotFound(String),
}
pub type Result<T> = std::result::Result<T, StoreError>;

pub fn now_iso() -> String {
    iso_from_secs(now_secs())
}
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
/// The timestamp `days` days before now, in `now_iso`'s form (they compare as text).
pub fn iso_days_ago(days: u64) -> String {
    iso_from_secs(now_secs().saturating_sub(days.saturating_mul(86_400)))
}
fn iso_from_secs(secs: u64) -> String {
    // seconds since epoch rendered as an RFC 3339 UTC string without a chrono dependency
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // civil-from-days (Howard Hinnant), valid for the years we care about
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Add the v5 columns an older store lacks. Idempotent: a column already there is left alone.
fn upgrade_v4(conn: &Connection) -> rusqlite::Result<()> {
    for (table, col, decl) in V5_COLUMNS {
        let have: Vec<String> = conn
            .prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))?
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if !have.iter().any(|h| h == col) {
            conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {col} {decl}"), [])?;
        }
    }
    Ok(())
}

/// Add the v4 columns a v3 `run` table lacks. Idempotent: a column already there is left alone.
fn upgrade_v3(conn: &Connection) -> rusqlite::Result<()> {
    let have: Vec<String> = conn
        .prepare("SELECT name FROM pragma_table_info('run')")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for (col, decl) in V4_RUN_COLUMNS {
        if !have.iter().any(|h| h == col) {
            conn.execute(&format!("ALTER TABLE run ADD COLUMN {col} {decl}"), [])?;
        }
    }
    Ok(())
}

fn bad(what: &str, e: impl std::fmt::Display) -> StoreError {
    StoreError::Schema(format!("stored {what} does not parse: {e}"))
}
fn json_vec<T: serde::de::DeserializeOwned>(what: &str, s: &str) -> Result<T> {
    serde_json::from_str(s).map_err(|e| bad(what, e))
}
fn to_json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).expect("plain data serialises")
}
/// The hash of a lens: sha256 of its JSON with `version` set to 0, so a version bump never changes it.
fn lens_body(l: &Lens) -> (String, String) {
    let mut c = l.clone();
    c.version = 0;
    let body = to_json(&c);
    (hex::encode(Sha256::digest(body.as_bytes())), body)
}

fn brief_from_value(v: &serde_json::Value) -> Result<BriefRef> {
    let s = |k: &str| {
        v[k].as_str()
            .map(str::to_string)
            .ok_or_else(|| bad("brief", k))
    };
    Ok(BriefRef {
        persona: s("persona")?,
        persona_version: v["persona_version"]
            .as_i64()
            .ok_or_else(|| bad("brief", "persona_version"))?,
        stake: Stake::parse(&s("stake")?).ok_or_else(|| bad("brief", "stake"))?,
        path: s("path")?,
        hash: s("hash")?,
        bytes: v["bytes"].as_u64().ok_or_else(|| bad("brief", "bytes"))? as usize,
        // a run made before v0.4 recorded none: `row_to_run` fills in the run's own model
        model: v["model"].as_str().unwrap_or("").to_string(),
    })
}

const RUN_COLS: &str = "id, subject_name, subject_version, subject_hash, panel_name, panel_hash, lens_name, lens_version, model, spend, stakes_json, personas_json, comparators_json, briefs_json, created_at, draw_seed, override_json, effort, trimmed_json, independence, rule_class, quorum";
fn row_to_run(r: &rusqlite::Row) -> rusqlite::Result<RunRow> {
    let text = |i: usize| r.get::<_, String>(i);
    let conv = |what: &'static str, e: StoreError| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(format!("{what}: {e}"))),
        )
    };
    let stakes_raw: BTreeMap<String, String> =
        json_vec("run stakes", &text(10)?).map_err(|e| conv("stakes", e))?;
    let mut stakes = BTreeMap::new();
    for (k, v) in stakes_raw {
        stakes.insert(
            k,
            Stake::parse(&v).ok_or_else(|| conv("stakes", bad("stake", &v)))?,
        );
    }
    let briefs_raw: Vec<serde_json::Value> =
        json_vec("run briefs", &text(13)?).map_err(|e| conv("briefs", e))?;
    let run_model: String = r.get(8)?;
    let mut briefs = briefs_raw
        .iter()
        .map(brief_from_value)
        .collect::<Result<Vec<_>>>()
        .map_err(|e| conv("briefs", e))?;
    for b in &mut briefs {
        if b.model.is_empty() {
            b.model = run_model.clone();
        }
    }
    Ok(RunRow {
        id: r.get(0)?,
        subject_name: r.get(1)?,
        subject_version: r.get(2)?,
        subject_hash: r.get(3)?,
        panel_name: r.get(4)?,
        panel_hash: r.get(5)?,
        lens_name: r.get(6)?,
        lens_version: r.get(7)?,
        model: r.get(8)?,
        spend: r.get(9)?,
        stakes,
        personas: json_vec("run personas", &text(11)?).map_err(|e| conv("personas", e))?,
        comparators: json_vec("run comparators", &text(12)?).map_err(|e| conv("comparators", e))?,
        briefs,
        created_at: r.get(14)?,
        draw_seed: r.get(15)?,
        override_json: match r.get::<_, Option<String>>(16)? {
            None => None,
            Some(t) => Some(json_vec("run override", &t).map_err(|e| conv("override", e))?),
        },
        effort: r.get(17)?,
        trimmed: match r.get::<_, Option<String>>(18)? {
            None => vec![],
            Some(t) => json_vec("run trimmed", &t).map_err(|e| conv("trimmed", e))?,
        },
        independence: r.get(19)?,
        rule: r.get(20)?,
        quorum: r.get(21)?,
    })
}

const PERSONA_COLS: &str = "id, version, name, tags_json, stake_rules, profile_json, origin, hash, traits_json, serves_json";
fn row_to_persona(r: &rusqlite::Row) -> rusqlite::Result<Persona> {
    let tags: String = r.get(3)?;
    Ok(Persona {
        id: r.get(0)?,
        version: r.get(1)?,
        name: r.get(2)?,
        tags: serde_json::from_str(&tags).unwrap_or_default(),
        stake_rules: r.get(4)?,
        profile_json: r.get(5)?,
        origin: Origin::from_db(&r.get::<_, String>(6)?),
        hash: r.get(7)?,
        traits: r
            .get::<_, Option<String>>(8)?
            .and_then(|t| serde_json::from_str(&t).ok()),
        serves: r
            .get::<_, Option<String>>(9)?
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default(),
    })
}

const VERDICT_COLS: &str = "v.run_id, v.batch, v.persona_id, v.persona_version, v.dimension, v.score, v.anchor_met, v.evidence, v.about, v.citations_json, v.citation_hash, v.blockers_json, v.note, v.created_at";
/// `v` rows of the latest initial batch of each persona in the run (D33: a deliberated batch never replaces it).
const LATEST_V: &str = "COALESCE(v.phase,'initial') = 'initial' AND v.batch = (SELECT max(v2.batch) FROM verdict v2 WHERE v2.run_id = v.run_id AND v2.persona_id = v.persona_id AND COALESCE(v2.phase,'initial') = 'initial')";
/// `v` rows of the latest deliberated batch of each persona in the run.
const LATEST_D: &str = "v.phase = 'deliberated' AND v.batch = (SELECT max(v2.batch) FROM verdict v2 WHERE v2.run_id = v.run_id AND v2.persona_id = v.persona_id AND v2.phase = 'deliberated')";

fn row_to_finding(r: &rusqlite::Row) -> rusqlite::Result<FindingRow> {
    Ok(FindingRow {
        id: r.get(0)?,
        run_id: r.get(1)?,
        batch: r.get(2)?,
        persona: r.get(3)?,
        subject_name: r.get(4)?,
        dimension: r.get(5)?,
        about: r.get(6)?,
        text: r.get(7)?,
        citation_hash: r.get(8)?,
    })
}
fn row_to_decision(r: &rusqlite::Row) -> rusqlite::Result<DecisionRow> {
    Ok(DecisionRow {
        id: r.get(0)?,
        finding_id: r.get(1)?,
        outcome: r.get(2)?,
        why: r.get(3)?,
        row_ref: r.get(4)?,
        by: r.get(5)?,
        created_at: r.get(6)?,
        after_run: r.get(7)?,
    })
}
const DECISION_COLS: &str = "id, finding_id, outcome, why, row_ref, by, created_at, after_run";
const FINDING_COLS: &str =
    "f.id, f.run_id, f.batch, f.persona_id, f.subject_name, f.dimension, f.about, f.text, f.citation_hash";

#[derive(Debug)]
pub struct Store {
    conn: Connection,
    path: PathBuf,
}

impl Store {
    /// Open a store that must already exist; never creates `.personajury`. Read verbs use this.
    pub fn open_existing(root: &Path) -> Result<Store> {
        let dir = root.join(DIR);
        if !dir.join(DB).is_file() {
            return Err(StoreError::NotFound(format!(
                "no workspace at {} — run a personajury verb that writes first",
                dir.display()
            )));
        }
        Store::open(root)
    }
    /// Create if needed, apply the schema, seed the shipped lenses.
    pub fn open(root: &Path) -> Result<Store> {
        let dir = root.join(DIR);
        std::fs::create_dir_all(&dir).map_err(|e| StoreError::Io {
            path: dir.clone(),
            source: e,
        })?;
        let path = dir.join(DB);
        let open_err = |source| StoreError::Open {
            path: path.clone(),
            source,
        };
        let conn = Connection::open(&path).map_err(open_err)?;
        // refuse a store of another version before anything is written to it (no pragma, no table)
        let has_meta = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'meta'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if has_meta {
            let v: Option<String> = conn
                .query_row("SELECT value FROM meta WHERE key = 'schema_version'", [], |r| r.get(0))
                .optional()?;
            // a v3 store is upgraded below; anything else is another binary's
            if let Some(v) = v.filter(|v| !matches!(v.parse::<i64>().ok(), Some(3) | Some(4) | Some(5))) {
                return Err(StoreError::Schema(format!(
                    "{} has schema_version {v}, this binary needs {SCHEMA_VERSION}",
                    path.display()
                )));
            }
        }
        conn.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;",
        )
        .map_err(open_err)?;
        conn.execute_batch(SCHEMA).map_err(open_err)?;
        upgrade_v3(&conn).map_err(open_err)?;
        upgrade_v4(&conn).map_err(open_err)?;
        let existing: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'schema_version'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        match existing.as_deref().map(|v| v.parse::<i64>()) {
            None => {
                conn.execute(
                    "INSERT INTO meta(key, value) VALUES ('schema_version', ?1)",
                    params![SCHEMA_VERSION.to_string()],
                )?;
            }
            Some(Ok(v)) if v == SCHEMA_VERSION => {}
            Some(Ok(3)) | Some(Ok(4)) => {
                conn.execute(
                    "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
                    params![SCHEMA_VERSION.to_string()],
                )?;
            }
            Some(other) => {
                return Err(StoreError::Schema(format!(
                    "{} has schema_version {}, this binary needs {SCHEMA_VERSION}",
                    path.display(),
                    match other {
                        Ok(v) => v.to_string(),
                        Err(_) => "(unreadable)".into(),
                    },
                )))
            }
        }
        let store = Store { conn, path };
        for (name, toml) in SHIPPED_LENSES {
            let have: Option<i64> = store
                .conn
                .query_row(
                    "SELECT 1 FROM lens WHERE name = ?1 LIMIT 1",
                    params![name],
                    |r| r.get(0),
                )
                .optional()?;
            if have.is_none() {
                let lens = Lens::from_toml(toml)
                    .map_err(|e| StoreError::Schema(format!("shipped lens {name}: {e}")))?;
                store.insert_lens(&lens)?;
            }
        }
        Ok(store)
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
    pub fn schema_version(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row(
                "SELECT value FROM meta WHERE key='schema_version'",
                [],
                |r| r.get::<_, String>(0),
            )?
            .parse()
            .unwrap_or(0))
    }

    /// The versioning rule shared by personas, lenses and subjects: the version to write for `hash`, or None when
    /// the latest version already has that hash (then `Err(version)` is the existing one).
    fn next_version(
        &self,
        table: &str,
        key_col: &str,
        key: &str,
        hash: &str,
    ) -> Result<std::result::Result<i64, i64>> {
        let latest: Option<(i64, String)> = self
            .conn
            .query_row(
                &format!("SELECT version, hash FROM {table} WHERE {key_col} = ?1 ORDER BY version DESC LIMIT 1"),
                params![key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        Ok(match latest {
            Some((v, h)) if h == hash => Err(v),
            Some((v, _)) => Ok(v + 1),
            None => Ok(1),
        })
    }

    // ---- personas ----
    /// Same id and hash as the latest → that version, no new row; else version + 1. Returns the version.
    pub fn insert_persona(&self, p: &Persona) -> Result<i64> {
        let version = match self.next_version("persona", "id", &p.id, &p.hash)? {
            Err(existing) => return Ok(existing),
            Ok(v) => v,
        };
        self.conn.execute(
            "INSERT INTO persona(id, version, name, tags_json, stake_rules, profile_json, origin, hash, created_at, traits_json, serves_json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![p.id, version, p.name, to_json(&p.tags), p.stake_rules, p.profile_json, p.origin.to_db(), p.hash, now_iso(),
                p.traits.as_ref().map(to_json), (!p.serves.is_empty()).then(|| to_json(&p.serves))],
        )?;
        Ok(version)
    }
    pub fn persona(&self, id: &str) -> Result<Option<Persona>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {PERSONA_COLS} FROM persona WHERE id = ?1 ORDER BY version DESC LIMIT 1"),
                params![id],
                row_to_persona,
            )
            .optional()?)
    }
    pub fn persona_at(&self, id: &str, version: i64) -> Result<Option<Persona>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {PERSONA_COLS} FROM persona WHERE id = ?1 AND version = ?2"),
                params![id, version],
                row_to_persona,
            )
            .optional()?)
    }
    /// Latest version of each persona, by id.
    pub fn personas(&self) -> Result<Vec<Persona>> {
        let mut st = self.conn.prepare(&format!(
            "SELECT {PERSONA_COLS} FROM persona p WHERE version = (SELECT max(version) FROM persona q WHERE q.id = p.id) ORDER BY id"
        ))?;
        let rows = st
            .query_map([], row_to_persona)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
    pub fn insert_material(&self, hash: &str, sources: &[String], brief_path: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO material(hash, sources_json, brief_path, created_at) VALUES (?1,?2,?3,?4)",
            params![hash, to_json(&sources), brief_path, now_iso()],
        )?;
        Ok(())
    }
    pub fn material_exists(&self, hash: &str) -> Result<bool> {
        Ok(self
            .conn
            .query_row(
                "SELECT 1 FROM material WHERE hash = ?1",
                params![hash],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
            .is_some())
    }

    // ---- panels ----
    /// Sets the query; an existing panel keeps its resolution until `set_panel_resolution` runs again.
    pub fn upsert_panel(&self, name: &str, query: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO panel(name, query, created_at) VALUES (?1,?2,?3) ON CONFLICT(name) DO UPDATE SET query = excluded.query",
            params![name, query, now_iso()],
        )?;
        Ok(())
    }
    /// The traits a panel's draw covers (D30); an empty list clears it.
    pub fn set_panel_cover(&self, name: &str, cover: &[String]) -> Result<()> {
        self.conn.execute(
            "UPDATE panel SET cover_json = ?2 WHERE name = ?1",
            params![name, (!cover.is_empty()).then(|| to_json(&cover))],
        )?;
        Ok(())
    }
    pub fn panel_cover(&self, name: &str) -> Result<Vec<String>> {
        let t: Option<Option<String>> = self
            .conn
            .query_row("SELECT cover_json FROM panel WHERE name = ?1", params![name], |r| r.get(0))
            .optional()?;
        Ok(t.flatten().map(|t| json_vec("panel cover", &t)).transpose()?.unwrap_or_default())
    }
    pub fn set_panel_resolution(&self, name: &str, ids: &[String], hash: &str) -> Result<()> {
        let n = self.conn.execute(
            "UPDATE panel SET persona_ids_json = ?2, hash = ?3 WHERE name = ?1",
            params![name, to_json(&ids), hash],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(format!("no panel named {name}")));
        }
        Ok(())
    }
    fn panel_from(r: &rusqlite::Row) -> rusqlite::Result<(PanelRow, String)> {
        Ok((
            PanelRow {
                name: r.get(0)?,
                query: r.get(1)?,
                persona_ids: vec![],
                hash: r.get(3)?,
            },
            r.get(2)?,
        ))
    }
    fn finish_panel((mut p, ids): (PanelRow, String)) -> Result<PanelRow> {
        p.persona_ids = json_vec("panel persona ids", &ids)?;
        Ok(p)
    }
    pub fn panel(&self, name: &str) -> Result<Option<PanelRow>> {
        self.conn
            .query_row(
                "SELECT name, query, persona_ids_json, hash FROM panel WHERE name = ?1",
                params![name],
                Self::panel_from,
            )
            .optional()?
            .map(Self::finish_panel)
            .transpose()
    }
    pub fn panels(&self) -> Result<Vec<PanelRow>> {
        let mut st = self
            .conn
            .prepare("SELECT name, query, persona_ids_json, hash FROM panel ORDER BY name")?;
        let rows = st
            .query_map([], Self::panel_from)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rows.into_iter().map(Self::finish_panel).collect()
    }

    // ---- traits and trait sets (D29) ----
    /// Same rule as lenses, by the hash of the body with `version` 0. `Err(version)`: the latest already has this hash.
    pub fn insert_trait(&self, t: &TraitDef, origin: &str) -> Result<std::result::Result<i64, i64>> {
        self.insert_trait_why(t, origin, None)
    }
    /// `insert_trait` with the reason a forced `trait install` recorded (D29).
    pub fn insert_trait_why(&self, t: &TraitDef, origin: &str, reason: Option<&str>) -> Result<std::result::Result<i64, i64>> {
        let (body, hash) = t.body_and_hash();
        let version = match self.next_version("trait", "id", &t.id, &hash)? {
            Err(existing) => return Ok(Err(existing)),
            Ok(v) => v,
        };
        self.conn.execute(
            "INSERT INTO trait(id, version, family, body_json, hash, origin, created_at, reason) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![t.id, version, t.family, body, hash, origin, now_iso(), reason],
        )?;
        Ok(Ok(version))
    }
    /// The origin and hash of the latest version of trait `id`.
    pub fn trait_latest(&self, id: &str) -> Result<Option<(String, String)>> {
        Ok(self
            .conn
            .query_row(
                "SELECT origin, hash FROM trait WHERE id = ?1 ORDER BY version DESC LIMIT 1",
                params![id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?)
    }
    fn trait_from(version: i64, body: &str) -> Result<TraitDef> {
        let mut t: TraitDef = json_vec("trait", body)?;
        t.version = version;
        Ok(t)
    }
    pub fn trait_def(&self, id: &str) -> Result<Option<TraitDef>> {
        self.conn
            .query_row(
                "SELECT version, body_json FROM trait WHERE id = ?1 ORDER BY version DESC LIMIT 1",
                params![id],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?
            .map(|(v, b)| Self::trait_from(v, &b))
            .transpose()
    }
    /// Latest version of each trait, by id, with its origin, hash and the reason a forced install recorded.
    pub fn traits(&self) -> Result<Vec<TraitRow>> {
        let mut st = self.conn.prepare(
            "SELECT version, body_json, origin, hash, reason FROM trait t WHERE version = (SELECT max(version) FROM trait u WHERE u.id = t.id) ORDER BY id",
        )?;
        let rows = st
            .query_map([], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, Option<String>>(4)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rows.into_iter().map(|(v, b, o, h, why)| Ok((Self::trait_from(v, &b)?, o, h, why))).collect()
    }
    /// The latest trait definitions by id.
    pub fn trait_map(&self) -> Result<BTreeMap<String, TraitDef>> {
        Ok(self.traits()?.into_iter().map(|(t, _, _, _)| (t.id.clone(), t)).collect())
    }
    pub fn insert_trait_set(&self, s: &TraitSetDef) -> Result<std::result::Result<i64, i64>> {
        let (body, hash) = s.body_and_hash();
        let version = match self.next_version("trait_set", "name", &s.name, &hash)? {
            Err(existing) => return Ok(Err(existing)),
            Ok(v) => v,
        };
        self.conn.execute(
            "INSERT INTO trait_set(name, version, body_json, hash, created_at) VALUES (?1,?2,?3,?4,?5)",
            params![s.name, version, body, hash, now_iso()],
        )?;
        Ok(Ok(version))
    }
    pub fn trait_set(&self, name: &str) -> Result<Option<TraitSetDef>> {
        self.conn
            .query_row(
                "SELECT version, body_json FROM trait_set WHERE name = ?1 ORDER BY version DESC LIMIT 1",
                params![name],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?
            .map(|(v, b)| {
                let mut s: TraitSetDef = json_vec("trait set", &b)?;
                s.version = v;
                Ok(s)
            })
            .transpose()
    }
    pub fn trait_sets(&self) -> Result<Vec<TraitSetDef>> {
        let mut st = self.conn.prepare(
            "SELECT name FROM trait_set t WHERE version = (SELECT max(version) FROM trait_set u WHERE u.name = t.name) ORDER BY name",
        )?;
        let names = st.query_map([], |r| r.get::<_, String>(0))?.collect::<std::result::Result<Vec<_>, _>>()?;
        let mut out = Vec::new();
        for n in names {
            if let Some(s) = self.trait_set(&n)? {
                out.push(s);
            }
        }
        Ok(out)
    }
    /// Record what a run rendered: `{"mode","set","versions","draw"}` (D31) and the seats `--seats auto` chose.
    pub fn set_run_traits(&self, id: i64, traits: Option<&serde_json::Value>, seats: Option<&serde_json::Value>) -> Result<()> {
        self.conn.execute(
            "UPDATE run SET traits_json = ?2, seats_json = ?3 WHERE id = ?1",
            params![id, traits.map(to_json), seats.map(to_json)],
        )?;
        Ok(())
    }
    pub fn run_traits(&self, id: i64) -> Result<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let (t, s): (Option<String>, Option<String>) = self.conn.query_row(
            "SELECT traits_json, seats_json FROM run WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        Ok((
            t.map(|t| json_vec("run traits", &t)).transpose()?,
            s.map(|t| json_vec("run seats", &t)).transpose()?,
        ))
    }
    /// The audience tags a subject was added with (`subject add --audience`).
    /// The options mode a run was briefed under (D32): `subject` or `attachments`; none for a run made before v0.6.
    pub fn run_options_mode(&self, id: i64) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT options_mode FROM run WHERE id = ?1", params![id], |r| r.get::<_, Option<String>>(0))
            .optional()?
            .flatten())
    }
    pub fn set_run_options_mode(&self, id: i64, mode: &str) -> Result<()> {
        self.conn.execute("UPDATE run SET options_mode = ?2 WHERE id = ?1", params![id, mode])?;
        Ok(())
    }
    pub fn subject_audience(&self, name: &str) -> Result<Vec<String>> {
        let t: Option<Option<String>> = self
            .conn
            .query_row(
                "SELECT audience_json FROM subject WHERE name = ?1 ORDER BY version DESC LIMIT 1",
                params![name],
                |r| r.get(0),
            )
            .optional()?;
        Ok(t.flatten().map(|t| json_vec("subject audience", &t)).transpose()?.unwrap_or_default())
    }
    pub fn set_subject_audience(&self, name: &str, audience: &[String]) -> Result<()> {
        self.conn.execute(
            "UPDATE subject SET audience_json = ?2 WHERE name = ?1 AND version = (SELECT max(version) FROM subject m WHERE m.name = ?1)",
            params![name, to_json(&audience)],
        )?;
        Ok(())
    }

    // ---- lenses ----
    /// Same rule as personas, by the hash of the lens's canonical JSON with `version` set to 0. Returns the version.
    pub fn insert_lens(&self, l: &Lens) -> Result<i64> {
        let (hash, body) = lens_body(l);
        let version = match self.next_version("lens", "name", &l.name, &hash)? {
            Err(existing) => return Ok(existing),
            Ok(v) => v,
        };
        self.conn.execute(
            "INSERT INTO lens(name, version, body_json, hash, created_at) VALUES (?1,?2,?3,?4,?5)",
            params![l.name, version, body, hash, now_iso()],
        )?;
        Ok(version)
    }
    fn lens_from(version: i64, body: &str) -> Result<Lens> {
        let mut l: Lens = json_vec("lens", body)?;
        l.version = version;
        Ok(l)
    }
    pub fn lens(&self, name: &str) -> Result<Option<Lens>> {
        self.conn
            .query_row(
                "SELECT version, body_json FROM lens WHERE name = ?1 ORDER BY version DESC LIMIT 1",
                params![name],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?
            .map(|(v, b)| Self::lens_from(v, &b))
            .transpose()
    }
    pub fn lens_at(&self, name: &str, version: i64) -> Result<Option<Lens>> {
        self.conn
            .query_row(
                "SELECT version, body_json FROM lens WHERE name = ?1 AND version = ?2",
                params![name, version],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?
            .map(|(v, b)| Self::lens_from(v, &b))
            .transpose()
    }
    /// Latest version of each lens, by name.
    pub fn lenses(&self) -> Result<Vec<Lens>> {
        let mut st = self.conn.prepare(
            "SELECT version, body_json FROM lens l WHERE version = (SELECT max(version) FROM lens m WHERE m.name = l.name) ORDER BY name",
        )?;
        let rows = st
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rows.into_iter()
            .map(|(v, b)| Self::lens_from(v, &b))
            .collect()
    }

    // ---- subjects and comparators ----
    /// Same rule as personas, by `Subject.hash` (a comparator and a subject never share a name). Returns the version.
    pub fn insert_subject(&self, s: &Subject) -> Result<i64> {
        let version = match self.next_version("subject", "name", &s.name, &s.hash)? {
            Err(existing) => return Ok(existing),
            Ok(v) => v,
        };
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO subject(name, version, kind, is_comparator, hash, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![s.name, version, s.kind.name(), s.is_comparator as i64, s.hash, now_iso()],
        )?;
        for a in &s.attachments {
            tx.execute(
                "INSERT INTO attachment(subject_name, subject_version, label, media, hash, source_path) VALUES (?1,?2,?3,?4,?5,?6)",
                params![s.name, version, a.label, a.media, a.hash, a.source_path],
            )?;
        }
        tx.commit()?;
        Ok(version)
    }
    fn subject_where(&self, clause: &str, p: &[&dyn rusqlite::ToSql]) -> Result<Vec<Subject>> {
        let mut st = self.conn.prepare(&format!(
            "SELECT name, version, kind, is_comparator, hash FROM subject s WHERE {clause}"
        ))?;
        let rows = st
            .query_map(p, |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut at = self.conn.prepare(
            "SELECT label, media, hash, source_path FROM attachment WHERE subject_name = ?1 AND subject_version = ?2 ORDER BY label",
        )?;
        let mut out = Vec::new();
        for (name, version, kind, cmp, hash) in rows {
            let attachments = at
                .query_map(params![name, version], |r| {
                    Ok(Attachment {
                        label: r.get(0)?,
                        media: r.get(1)?,
                        hash: r.get(2)?,
                        source_path: r.get(3)?,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let kind = SubjectKind::parse(&kind).ok_or_else(|| bad("subject kind", &kind))?;
            out.push(Subject {
                name,
                version,
                kind,
                is_comparator: cmp != 0,
                hash,
                attachments,
            });
        }
        Ok(out)
    }
    pub fn subject(&self, name: &str) -> Result<Option<Subject>> {
        Ok(self
            .subject_where(
                "name = ?1 AND version = (SELECT max(version) FROM subject m WHERE m.name = s.name)",
                &[&name],
            )?
            .pop())
    }
    pub fn subject_at(&self, name: &str, version: i64) -> Result<Option<Subject>> {
        Ok(self
            .subject_where("name = ?1 AND version = ?2", &[&name, &version])?
            .pop())
    }
    pub fn subjects(&self, comparators: bool) -> Result<Vec<Subject>> {
        self.subject_where(
            "is_comparator = ?1 AND version = (SELECT max(version) FROM subject m WHERE m.name = s.name) ORDER BY name",
            &[&(comparators as i64)],
        )
    }

    // ---- runs ----
    pub fn insert_run(&self, r: &RunRow) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO run(subject_name, subject_version, subject_hash, panel_name, panel_hash, lens_name, lens_version, model, spend, stakes_json, personas_json, comparators_json, briefs_json, created_at, draw_seed, override_json, effort, trimmed_json, independence, rule_class, quorum) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)",
            params![
                r.subject_name,
                r.subject_version,
                r.subject_hash,
                r.panel_name,
                r.panel_hash,
                r.lens_name,
                r.lens_version,
                r.model,
                r.spend,
                to_json(&r.stakes),
                to_json(&r.personas),
                to_json(&r.comparators),
                to_json(&r.briefs),
                r.created_at,
                r.draw_seed,
                r.override_json.as_ref().map(to_json),
                r.effort,
                (!r.trimmed.is_empty()).then(|| to_json(&r.trimmed)),
                r.independence,
                r.rule,
                r.quorum
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }
    /// Set the brief references after the files are written (the run id names their folder).
    pub fn set_run_briefs(&self, id: i64, briefs: &[BriefRef]) -> Result<()> {
        let n = self.conn.execute(
            "UPDATE run SET briefs_json = ?2 WHERE id = ?1",
            params![id, to_json(&briefs)],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(format!("no run {id}")));
        }
        Ok(())
    }
    pub fn run(&self, id: i64) -> Result<Option<RunRow>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {RUN_COLS} FROM run WHERE id = ?1"),
                params![id],
                row_to_run,
            )
            .optional()?)
    }
    pub fn runs(&self) -> Result<Vec<RunRow>> {
        let mut st = self
            .conn
            .prepare(&format!("SELECT {RUN_COLS} FROM run ORDER BY id"))?;
        let rows = st
            .query_map([], row_to_run)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ---- verdicts ----
    /// 1 + the highest batch of (run, persona), or 1.
    pub fn next_batch(&self, run_id: i64, persona: &str) -> Result<i64> {
        let max: Option<i64> = self.conn.query_row(
            "SELECT max(batch) FROM verdict WHERE run_id = ?1 AND persona_id = ?2",
            params![run_id, persona],
            |r| r.get(0),
        )?;
        Ok(max.unwrap_or(0) + 1)
    }
    /// Lines, pick and findings in one transaction. `persona_version` comes from the run's brief reference.
    pub fn insert_verdict(
        &self,
        run_id: i64,
        batch: i64,
        persona_version: i64,
        subject_name: &str,
        v: &ValidVerdict,
    ) -> Result<()> {
        self.insert_verdict_by(run_id, batch, persona_version, subject_name, v, None)
    }
    /// `insert_verdict`, and when a person entered the batch, who (D19), in the same transaction.
    pub fn insert_verdict_by(
        &self,
        run_id: i64,
        batch: i64,
        persona_version: i64,
        subject_name: &str,
        v: &ValidVerdict,
        by_human: Option<&str>,
    ) -> Result<()> {
        self.insert_verdict_phase(run_id, batch, persona_version, subject_name, v, by_human, None)
    }
    /// `insert_verdict_by`, in a phase: `deliberated` for the advise-only round (D33), none for an initial batch.
    #[allow(clippy::too_many_arguments)]
    pub fn insert_verdict_phase(
        &self,
        run_id: i64,
        batch: i64,
        persona_version: i64,
        subject_name: &str,
        v: &ValidVerdict,
        by_human: Option<&str>,
        phase: Option<&str>,
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        let now = now_iso();
        for l in &v.lines {
            tx.execute(
                "INSERT INTO verdict(run_id, batch, persona_id, persona_version, dimension, score, anchor_met, evidence, about, citations_json, citation_hash, blockers_json, note, created_at, phase) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
                params![
                    run_id,
                    batch,
                    v.persona,
                    persona_version,
                    l.line.dimension,
                    l.line.score,
                    l.line.anchor_met,
                    l.line.evidence.name(),
                    l.line.about,
                    to_json(&l.line.citations),
                    l.citation_hash,
                    to_json(&l.line.blockers),
                    l.line.note,
                    now,
                    phase
                ],
            )?;
        }
        if let Some(by) = by_human {
            tx.execute(
                "INSERT INTO human_batch(run_id, batch, persona_id, by_human, created_at) VALUES (?1,?2,?3,?4,?5)",
                params![run_id, batch, v.persona, by, now],
            )?;
        }
        if let Some(p) = &v.pick {
            tx.execute(
                "INSERT INTO pick(run_id, batch, persona_id, choice, reason, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
                params![run_id, batch, v.persona, p.choice, p.reason, now],
            )?;
        }
        if let Some(u) = &v.usage {
            tx.execute(
                "INSERT INTO usage(run_id, batch, persona_id, input_tokens, output_tokens, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
                params![run_id, batch, v.persona, u.input_tokens as i64, u.output_tokens as i64, now],
            )?;
        }
        // A finding id seen before keeps its first row (and run): the same point raised again is the same finding.
        // Whether that repeat is refused at verdict time (R12) depends only on a `dismissed` decision, which the
        // application layer checks through `dismissed_citations`; the store just keeps the first row for `check`.
        for f in &v.findings {
            tx.execute(
                "INSERT OR IGNORE INTO finding(id, run_id, batch, persona_id, subject_name, dimension, about, text, citation_hash) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![f.id, run_id, batch, f.persona, subject_name, f.dimension, f.about, f.text, f.citation_hash],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    /// history=false: the latest batch of each persona only. Ordered by persona, batch, insertion.
    pub fn verdict_rows(&self, run_id: i64, history: bool) -> Result<Vec<VerdictRow>> {
        self.verdict_rows_in(run_id, if history { None } else { Some(false) })
    }
    /// The latest deliberated batch of each persona (D33).
    pub fn deliberated_rows(&self, run_id: i64) -> Result<Vec<VerdictRow>> {
        self.verdict_rows_in(run_id, Some(true))
    }
    /// `None`: every batch; `Some(false)`: the latest initial batch of each persona; `Some(true)`: the latest deliberated.
    fn verdict_rows_in(&self, run_id: i64, only: Option<bool>) -> Result<Vec<VerdictRow>> {
        let sql = format!(
            "SELECT {VERDICT_COLS} FROM verdict v WHERE v.run_id = ?1 {} ORDER BY v.persona_id, v.batch, v.id",
            match only {
                None => String::new(),
                Some(false) => format!("AND {LATEST_V}"),
                Some(true) => format!("AND {LATEST_D}"),
            }
        );
        let mut st = self.conn.prepare(&sql)?;
        let raw = st
            .query_map(params![run_id], |r| {
                Ok((
                    VerdictRow {
                        run_id: r.get(0)?,
                        batch: r.get(1)?,
                        persona: r.get(2)?,
                        persona_version: r.get(3)?,
                        dimension: r.get(4)?,
                        score: r.get(5)?,
                        anchor_met: r.get(6)?,
                        evidence: Evidence::Seen,
                        about: r.get(8)?,
                        citations: vec![],
                        citation_hash: r.get(10)?,
                        blockers: vec![],
                        note: r.get(12)?,
                        created_at: r.get(13)?,
                    },
                    r.get::<_, String>(7)?,
                    r.get::<_, String>(9)?,
                    r.get::<_, String>(11)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        raw.into_iter()
            .map(|(mut row, ev, cit, blk)| {
                row.evidence = Evidence::parse(&ev).ok_or_else(|| bad("evidence", &ev))?;
                row.citations = json_vec("citations", &cit)?;
                row.blockers = json_vec("blockers", &blk)?;
                Ok(row)
            })
            .collect()
    }
    /// The phase of every batch of a run: `(persona, batch)` to `initial` or `deliberated`.
    pub fn batch_phases(&self, run_id: i64) -> Result<BTreeMap<(String, i64), String>> {
        let mut st = self.conn.prepare(
            "SELECT DISTINCT persona_id, batch, COALESCE(phase,'initial') FROM verdict WHERE run_id = ?1",
        )?;
        let rows = st
            .query_map(params![run_id], |r| Ok(((r.get::<_, String>(0)?, r.get::<_, i64>(1)?), r.get::<_, String>(2)?)))?
            .collect::<std::result::Result<BTreeMap<_, _>, _>>()?;
        Ok(rows)
    }
    /// The deliberation round of a run (D33): the files written, as stored; none before `deliberate`.
    pub fn deliberation(&self, run_id: i64) -> Result<Option<serde_json::Value>> {
        let t: Option<String> = self
            .conn
            .query_row("SELECT briefs_json FROM deliberation WHERE run_id = ?1", params![run_id], |r| r.get(0))
            .optional()?;
        t.map(|t| json_vec("deliberation", &t)).transpose()
    }
    pub fn insert_deliberation(&self, run_id: i64, briefs: &serde_json::Value) -> Result<()> {
        self.conn.execute(
            "INSERT INTO deliberation(run_id, briefs_json, created_at) VALUES (?1,?2,?3)",
            params![run_id, to_json(briefs), now_iso()],
        )?;
        Ok(())
    }
    /// The tokens the agent reported over every verdict of these runs (all batches: a re-run was spent too), and how
    /// many verdicts reported. None when nothing was reported. `run_ids` empty = every run.
    pub fn reported_usage(&self, run_ids: &[i64]) -> Result<Option<(u64, u64, u64)>> {
        let (mut input, mut output, mut n) = (0u64, 0u64, 0u64);
        let mut st = self
            .conn
            .prepare("SELECT run_id, input_tokens, output_tokens FROM usage")?;
        let rows = st.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?)))?;
        for row in rows {
            let (run, i, o) = row?;
            if run_ids.is_empty() || run_ids.contains(&run) {
                input = input.saturating_add(i.max(0) as u64);
                output = output.saturating_add(o.max(0) as u64);
                n += 1;
            }
        }
        Ok((n > 0).then_some((input, output, n)))
    }
    /// Who entered each human batch of a run (D19): persona id (`human:<name>:<seat>`) to the name and credential given.
    /// A later batch of the same id names who entered it.
    pub fn human_batches(&self, run_id: i64) -> Result<BTreeMap<String, String>> {
        let mut st = self
            .conn
            .prepare("SELECT persona_id, by_human FROM human_batch WHERE run_id = ?1 ORDER BY id")?;
        let rows = st
            .query_map(params![run_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows.into_iter().collect())
    }
    /// The tokens reported by verdicts stored at or after `since` (an `iso_days_ago` text): (input, output, verdicts).
    pub fn reported_usage_since(&self, since: &str) -> Result<Option<(u64, u64, u64)>> {
        let (mut input, mut output, mut n) = (0u64, 0u64, 0u64);
        let mut st = self
            .conn
            .prepare("SELECT input_tokens, output_tokens FROM usage WHERE created_at >= ?1")?;
        let rows = st.query_map(params![since], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            let (i, o) = row?;
            input = input.saturating_add(i.max(0) as u64);
            output = output.saturating_add(o.max(0) as u64);
            n += 1;
        }
        Ok((n > 0).then_some((input, output, n)))
    }
    pub fn picks(&self, run_id: i64, history: bool) -> Result<Vec<PickRow>> {
        self.picks_in(run_id, if history { None } else { Some(false) })
    }
    /// The picks of each persona's latest deliberated batch (D33).
    pub fn deliberated_picks(&self, run_id: i64) -> Result<Vec<PickRow>> {
        self.picks_in(run_id, Some(true))
    }
    fn picks_in(&self, run_id: i64, only: Option<bool>) -> Result<Vec<PickRow>> {
        // "latest" is the persona's latest verdict batch of the phase: a later batch without a pick shows no pick
        let filter = match only {
            None => "",
            Some(false) => "AND v.batch = (SELECT max(p2.batch) FROM verdict p2 WHERE p2.run_id = v.run_id AND p2.persona_id = v.persona_id AND COALESCE(p2.phase,'initial') = 'initial')",
            Some(true) => "AND v.batch = (SELECT max(p2.batch) FROM verdict p2 WHERE p2.run_id = v.run_id AND p2.persona_id = v.persona_id AND p2.phase = 'deliberated') AND EXISTS (SELECT 1 FROM verdict p3 WHERE p3.run_id = v.run_id AND p3.persona_id = v.persona_id AND p3.batch = v.batch AND p3.phase = 'deliberated')",
        };
        let mut st = self.conn.prepare(&format!(
            "SELECT v.run_id, v.batch, v.persona_id, v.choice, v.reason, v.created_at, v.about FROM pick v WHERE v.run_id = ?1 {filter} ORDER BY v.persona_id, v.batch, v.id"
        ))?;
        let rows = st
            .query_map(params![run_id], |r| {
                Ok(PickRow {
                    run_id: r.get(0)?,
                    batch: r.get(1)?,
                    persona: r.get(2)?,
                    choice: r.get(3)?,
                    reason: r.get(4)?,
                    created_at: r.get(5)?,
                    about: r.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
    /// The findings first raised in the run, from every batch (C3: a later batch without the blocker never clears it).
    /// A finding id seen before keeps its first row, so a point repeated in a later run is read through `finding(id)`.
    pub fn findings(&self, run_id: i64) -> Result<Vec<FindingRow>> {
        let mut st = self.conn.prepare(&format!(
            "SELECT {FINDING_COLS} FROM finding f WHERE f.run_id = ?1 ORDER BY f.persona_id, f.id"
        ))?;
        let rows = st
            .query_map(params![run_id], row_to_finding)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
    pub fn finding(&self, id: &str) -> Result<Option<FindingRow>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {FINDING_COLS} FROM finding f WHERE f.id = ?1"),
                params![id],
                row_to_finding,
            )
            .optional()?)
    }
    /// The highest run id, 0 when there is none.
    pub fn max_run_id(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT coalesce(max(id), 0) FROM run", [], |r| r.get(0))?)
    }
    /// Remove a run that never got its briefs (brief failed after the insert). Refused once it holds a verdict.
    pub fn delete_run(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM run WHERE id = ?1 AND NOT EXISTS (SELECT 1 FROM verdict WHERE run_id = ?1)",
            params![id],
        )?;
        Ok(())
    }

    // ---- decisions ----
    pub fn insert_decision(&self, d: &DecisionRow) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO decision(finding_id, outcome, why, row_ref, by, created_at, after_run) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![d.finding_id, d.outcome, d.why, d.row_ref, d.by, d.created_at, d.after_run],
        )?;
        Ok(self.conn.last_insert_rowid())
    }
    pub fn decisions(&self, finding_id: &str) -> Result<Vec<DecisionRow>> {
        let mut st = self.conn.prepare(&format!(
            "SELECT {DECISION_COLS} FROM decision WHERE finding_id = ?1 ORDER BY id"
        ))?;
        let rows = st
            .query_map(params![finding_id], row_to_decision)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
    pub fn latest_decision(&self, finding_id: &str) -> Result<Option<DecisionRow>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {DECISION_COLS} FROM decision WHERE finding_id = ?1 ORDER BY id DESC LIMIT 1"),
                params![finding_id],
                row_to_decision,
            )
            .optional()?)
    }
    /// What a persona may not raise again on a subject (R12, D6 B3, I9): for each of its findings whose latest decision
    /// is `dismissed`, `<dimension>\t<about>\t<citation hash>` when the finding has citations (about empty for the
    /// subject as a whole), else the finding id (an uncited point is "the same" only by its dimension, option and
    /// text; the hash of no citations is shared by every uncited point).
    pub fn dismissed_citations(
        &self,
        subject_name: &str,
        persona: &str,
    ) -> Result<BTreeSet<String>> {
        let empty = crate::domain::hash::sha256_hex(b"");
        let mut st = self.conn.prepare(
            "SELECT DISTINCT CASE WHEN f.citation_hash = ?3 THEN f.id ELSE f.dimension || char(9) || coalesce(f.about, '') || char(9) || f.citation_hash END FROM finding f \
             WHERE f.subject_name = ?1 AND f.persona_id = ?2 \
             AND (SELECT d.outcome FROM decision d WHERE d.finding_id = f.id ORDER BY d.id DESC LIMIT 1) = 'dismissed'",
        )?;
        let rows = st
            .query_map(params![subject_name, persona, empty], |r| {
                r.get::<_, String>(0)
            })?
            .collect::<std::result::Result<BTreeSet<_>, _>>()?;
        Ok(rows)
    }
}
