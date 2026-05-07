use std::path::Path;

use chrono::{DateTime, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, OptionalExtension};

#[allow(unused_imports)]
use chrono as _chrono_keepalive;

use crate::error::PulseResult;
use crate::model::{
    AttributionConfidence, DailyPoint, ModelShare, Settings, TaskMetadata, TaskSnapshot,
    UsageTotals,
};
use crate::turn::ParsedTurn;

pub type DbPool = Pool<SqliteConnectionManager>;

#[derive(Clone)]
pub struct Db {
    pool: DbPool,
}

impl Db {
    pub fn open(path: &Path) -> PulseResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let manager = SqliteConnectionManager::file(path).with_init(|conn| {
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA temp_store = MEMORY;
                 PRAGMA mmap_size  = 268435456;",
            )
        });
        let pool = Pool::builder().max_size(8).build(manager)?;
        {
            let conn = pool.get()?;
            migrate(&conn)?;
        }
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    // message_id is the dedup PK; streaming records collapse to the final usage.
    pub fn upsert_turn(
        &self,
        turn: &ParsedTurn,
        task_id: Option<&str>,
        confidence: AttributionConfidence,
        confidence_score: f64,
        cost_usd: f64,
    ) -> PulseResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            r#"INSERT INTO turns
                (message_id, session_id, request_id, ts, provider, model, branch, cwd,
                 input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens,
                 cost_usd, task_id, confidence, confidence_score)
                VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
                ON CONFLICT(message_id) DO UPDATE SET
                  input_tokens          = excluded.input_tokens,
                  output_tokens         = excluded.output_tokens,
                  cache_creation_tokens = excluded.cache_creation_tokens,
                  cache_read_tokens     = excluded.cache_read_tokens,
                  cost_usd              = excluded.cost_usd,
                  task_id               = excluded.task_id,
                  confidence            = excluded.confidence,
                  confidence_score      = excluded.confidence_score
            "#,
            params![
                turn.message_id,
                turn.session_id,
                turn.request_id,
                turn.ts.to_rfc3339(),
                turn.provider,
                turn.model,
                turn.branch,
                turn.cwd,
                turn.input_tokens as i64,
                turn.output_tokens as i64,
                turn.cache_creation_tokens as i64,
                turn.cache_read_tokens as i64,
                cost_usd,
                task_id,
                confidence_str(confidence),
                confidence_score,
            ],
        )?;
        Ok(())
    }

    pub fn upsert_session(
        &self,
        session_id: &str,
        cwd: Option<&str>,
        branch: Option<&str>,
        model: Option<&str>,
        project: Option<&str>,
        provider: &str,
        file_path: &str,
        ts: DateTime<Utc>,
    ) -> PulseResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            r#"INSERT INTO sessions
                (session_id, cwd, project, model, branch, provider, first_seen, last_seen, file_path, file_offset)
                VALUES (?,?,?,?,?,?,?,?,?,0)
                ON CONFLICT(session_id) DO UPDATE SET
                  cwd        = COALESCE(excluded.cwd, sessions.cwd),
                  project    = COALESCE(excluded.project, sessions.project),
                  model      = COALESCE(excluded.model, sessions.model),
                  branch     = COALESCE(excluded.branch, sessions.branch),
                  provider   = excluded.provider,
                  last_seen  = excluded.last_seen
            "#,
            params![
                session_id,
                cwd,
                project,
                model,
                branch,
                provider,
                ts.to_rfc3339(),
                ts.to_rfc3339(),
                file_path,
            ],
        )?;
        Ok(())
    }

    pub fn set_session_offset(&self, file_path: &str, offset: u64) -> PulseResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE sessions SET file_offset = ?1 WHERE file_path = ?2",
            params![offset as i64, file_path],
        )?;
        Ok(())
    }

    pub fn session_offset(&self, file_path: &str) -> PulseResult<u64> {
        let conn = self.pool.get()?;
        let off: Option<i64> = conn
            .query_row(
                "SELECT file_offset FROM sessions WHERE file_path = ?1",
                params![file_path],
                |r| r.get(0),
            )
            .optional()?;
        Ok(off.unwrap_or(0) as u64)
    }

    pub fn get_setting(&self, key: &str) -> PulseResult<Option<String>> {
        let conn = self.pool.get()?;
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .optional()?;
        Ok(v)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> PulseResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO settings(key, value) VALUES(?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn load_settings(&self) -> PulseResult<Settings> {
        if let Some(s) = self.get_setting("pulse.settings")? {
            if let Ok(v) = serde_json::from_str::<Settings>(&s) {
                return Ok(v);
            }
        }
        Ok(Settings::default())
    }

    pub fn save_settings(&self, s: &Settings) -> PulseResult<()> {
        self.set_setting("pulse.settings", &serde_json::to_string(s)?)
    }

    pub fn count_sessions(&self) -> PulseResult<u64> {
        let conn = self.pool.get()?;
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?;
        Ok(n as u64)
    }

    pub fn reset(&self) -> PulseResult<()> {
        let conn = self.pool.get()?;
        conn.execute_batch("DELETE FROM turns; DELETE FROM sessions;")?;
        Ok(())
    }

    pub fn usage_totals(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> PulseResult<UsageTotals> {
        let conn = self.pool.get()?;
        let row: (i64, i64, i64, i64, f64, i64) = conn.query_row(
            r#"SELECT
                  COALESCE(SUM(input_tokens), 0),
                  COALESCE(SUM(output_tokens), 0),
                  COALESCE(SUM(cache_creation_tokens), 0),
                  COALESCE(SUM(cache_read_tokens), 0),
                  COALESCE(SUM(cost_usd), 0),
                  COUNT(*)
                FROM turns
                WHERE ts BETWEEN ?1 AND ?2"#,
            params![from.to_rfc3339(), to.to_rfc3339()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )?;
        let (inp, out, ccr, crd, cost, calls) = row;
        let mut totals = UsageTotals {
            input_tokens: inp as u64,
            output_tokens: out as u64,
            cache_creation_tokens: ccr as u64,
            cache_read_tokens: crd as u64,
            total_tokens: (inp + out + ccr + crd) as u64,
            cost_usd: cost,
            calls: calls as u64,
            cache_hit_rate: 0.0,
        };
        totals.recompute_cache_hit_rate();
        Ok(totals)
    }

    pub fn usage_by_task(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> PulseResult<Vec<TaskSnapshot>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"SELECT
                  COALESCE(task_id, ''),
                  COALESCE(branch, ''),
                  COALESCE(cwd, ''),
                  MAX(model),
                  MAX(confidence),
                  AVG(confidence_score),
                  SUM(input_tokens),
                  SUM(output_tokens),
                  SUM(cache_creation_tokens),
                  SUM(cache_read_tokens),
                  SUM(cost_usd),
                  COUNT(*),
                  MIN(ts),
                  MAX(ts)
                FROM turns
                WHERE ts BETWEEN ?1 AND ?2
                GROUP BY COALESCE(task_id, '__unattributed__')
                ORDER BY SUM(input_tokens + output_tokens) DESC"#,
        )?;
        let rows = stmt.query_map(params![from.to_rfc3339(), to.to_rfc3339()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<f64>>(5)?.unwrap_or(0.0),
                r.get::<_, i64>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, i64>(8)?,
                r.get::<_, i64>(9)?,
                r.get::<_, f64>(10)?,
                r.get::<_, i64>(11)?,
                r.get::<_, String>(12)?,
                r.get::<_, String>(13)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (task_raw, branch, cwd, model, conf_raw, conf_score, inp, out_t, ccr, crd, cost, calls, first, last) =
                row?;
            let mut usage = UsageTotals {
                input_tokens: inp as u64,
                output_tokens: out_t as u64,
                cache_creation_tokens: ccr as u64,
                cache_read_tokens: crd as u64,
                total_tokens: (inp + out_t + ccr + crd) as u64,
                cost_usd: cost,
                calls: calls as u64,
                cache_hit_rate: 0.0,
            };
            usage.recompute_cache_hit_rate();
            let task_id = if task_raw.is_empty() { None } else { Some(task_raw) };
            let metadata = match &task_id {
                Some(id) => self.task_metadata(id).ok().flatten(),
                None => None,
            };
            let task_name = metadata
                .as_ref()
                .and_then(|m| m.title.clone())
                .or_else(|| task_id.clone());
            out.push(TaskSnapshot {
                task_id: task_id.clone(),
                task_name,
                branch: opt_string(branch),
                cwd: opt_string(cwd),
                model,
                confidence: confidence_from_str(conf_raw.as_deref()),
                confidence_score: conf_score,
                usage,
                first_seen: parse_ts(&first),
                last_seen: parse_ts(&last),
                metadata,
            });
        }
        Ok(out)
    }

    pub fn daily_series(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> PulseResult<Vec<DailyPoint>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"SELECT
                  substr(ts, 1, 10) AS day,
                  SUM(input_tokens + output_tokens + cache_creation_tokens + cache_read_tokens),
                  SUM(cost_usd),
                  COUNT(*)
                FROM turns
                WHERE ts BETWEEN ?1 AND ?2
                GROUP BY day
                ORDER BY day ASC"#,
        )?;
        let rows = stmt.query_map(params![from.to_rfc3339(), to.to_rfc3339()], |r| {
            Ok(DailyPoint {
                date: r.get(0)?,
                tokens: r.get::<_, i64>(1)? as u64,
                cost: r.get(2)?,
                calls: r.get::<_, i64>(3)? as u64,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn model_share(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> PulseResult<Vec<ModelShare>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"SELECT COALESCE(model, 'unknown'),
                      SUM(input_tokens + output_tokens + cache_creation_tokens + cache_read_tokens)
               FROM turns
               WHERE ts BETWEEN ?1 AND ?2
               GROUP BY model
               ORDER BY 2 DESC"#,
        )?;
        let rows: Vec<(String, i64)> = stmt
            .query_map(params![from.to_rfc3339(), to.to_rfc3339()], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let total: i64 = rows.iter().map(|(_, t)| *t).sum();
        Ok(rows
            .into_iter()
            .map(|(m, t)| ModelShare {
                model: m,
                tokens: t as u64,
                pct: if total == 0 {
                    0.0
                } else {
                    t as f64 / total as f64
                },
            })
            .collect())
    }

    pub fn upsert_task_metadata(&self, m: &TaskMetadata) -> PulseResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            r#"INSERT INTO task_metadata
                (task_id, enricher, title, status, assignee, url, project_key, issue_type, priority, fetched_at)
                VALUES (?,?,?,?,?,?,?,?,?,?)
                ON CONFLICT(task_id, enricher) DO UPDATE SET
                  title       = excluded.title,
                  status      = excluded.status,
                  assignee    = excluded.assignee,
                  url         = excluded.url,
                  project_key = excluded.project_key,
                  issue_type  = excluded.issue_type,
                  priority    = excluded.priority,
                  fetched_at  = excluded.fetched_at
            "#,
            params![
                m.task_id,
                m.enricher,
                m.title,
                m.status,
                m.assignee,
                m.url,
                m.project_key,
                m.issue_type,
                m.priority,
                m.fetched_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn task_metadata(&self, task_id: &str) -> PulseResult<Option<TaskMetadata>> {
        let conn = self.pool.get()?;
        let row = conn
            .query_row(
                r#"SELECT task_id, enricher, title, status, assignee, url, project_key,
                          issue_type, priority, fetched_at
                   FROM task_metadata
                   WHERE task_id = ?1
                   ORDER BY fetched_at DESC
                   LIMIT 1"#,
                params![task_id],
                map_metadata_row,
            )
            .optional()?;
        Ok(row)
    }

    pub fn unenriched_task_ids(&self, ttl_secs: u64, limit: u32) -> PulseResult<Vec<String>> {
        let conn = self.pool.get()?;
        let cutoff = (Utc::now() - chrono::Duration::seconds(ttl_secs as i64)).to_rfc3339();
        let mut stmt = conn.prepare(
            r#"SELECT DISTINCT t.task_id
               FROM turns t
               LEFT JOIN task_metadata m ON m.task_id = t.task_id
               WHERE t.task_id IS NOT NULL
                 AND (m.fetched_at IS NULL OR m.fetched_at < ?1)
               ORDER BY MAX(t.ts) DESC
               LIMIT ?2"#,
        )?;
        let ids: Vec<String> = stmt
            .query_map(params![cutoff, limit as i64], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(ids)
    }

    pub fn pending_enrichment_count(&self, ttl_secs: u64) -> PulseResult<u64> {
        let conn = self.pool.get()?;
        let cutoff = (Utc::now() - chrono::Duration::seconds(ttl_secs as i64)).to_rfc3339();
        let n: i64 = conn.query_row(
            r#"SELECT COUNT(DISTINCT t.task_id)
               FROM turns t
               LEFT JOIN task_metadata m ON m.task_id = t.task_id
               WHERE t.task_id IS NOT NULL
                 AND (m.fetched_at IS NULL OR m.fetched_at < ?1)"#,
            params![cutoff],
            |r| r.get(0),
        )?;
        Ok(n as u64)
    }
}

fn map_metadata_row(r: &rusqlite::Row) -> rusqlite::Result<TaskMetadata> {
    Ok(TaskMetadata {
        task_id: r.get(0)?,
        enricher: r.get(1)?,
        title: r.get(2)?,
        status: r.get(3)?,
        assignee: r.get(4)?,
        url: r.get(5)?,
        project_key: r.get(6)?,
        issue_type: r.get(7)?,
        priority: r.get(8)?,
        fetched_at: parse_ts(&r.get::<_, String>(9)?),
    })
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            session_id  TEXT PRIMARY KEY,
            cwd         TEXT,
            project     TEXT,
            model       TEXT,
            branch      TEXT,
            provider    TEXT NOT NULL DEFAULT 'claude-code',
            first_seen  TEXT NOT NULL,
            last_seen   TEXT NOT NULL,
            file_path   TEXT NOT NULL,
            file_offset INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS turns (
            message_id    TEXT PRIMARY KEY,
            session_id    TEXT NOT NULL REFERENCES sessions(session_id),
            request_id    TEXT,
            ts            TEXT NOT NULL,
            provider      TEXT NOT NULL DEFAULT 'claude-code',
            model         TEXT,
            branch        TEXT,
            cwd           TEXT,
            input_tokens  INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
            cache_read_tokens     INTEGER NOT NULL DEFAULT 0,
            cost_usd      REAL    NOT NULL DEFAULT 0,
            task_id       TEXT,
            confidence    TEXT    NOT NULL DEFAULT 'low',
            confidence_score REAL NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS meta     (key TEXT PRIMARY KEY, value TEXT NOT NULL);

        CREATE TABLE IF NOT EXISTS task_metadata (
            task_id     TEXT NOT NULL,
            enricher    TEXT NOT NULL,
            title       TEXT,
            status      TEXT,
            assignee    TEXT,
            url         TEXT,
            project_key TEXT,
            issue_type  TEXT,
            priority    TEXT,
            fetched_at  TEXT NOT NULL,
            PRIMARY KEY (task_id, enricher)
        );
        CREATE INDEX IF NOT EXISTS idx_task_metadata_fetched ON task_metadata(fetched_at);
        CREATE INDEX IF NOT EXISTS idx_task_metadata_project ON task_metadata(project_key);
        "#,
    )?;

    // Forward migrations from earlier dev builds. Each ALTER may error with
    // "duplicate column" when the column already exists — that's expected.
    add_column_if_missing(conn, "turns", "provider", "TEXT NOT NULL DEFAULT 'claude-code'")?;
    add_column_if_missing(conn, "sessions", "provider", "TEXT NOT NULL DEFAULT 'claude-code'")?;

    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_turns_ts        ON turns(ts);
        CREATE INDEX IF NOT EXISTS idx_turns_task_id   ON turns(task_id);
        CREATE INDEX IF NOT EXISTS idx_turns_session   ON turns(session_id);
        CREATE INDEX IF NOT EXISTS idx_turns_model     ON turns(model);
        CREATE INDEX IF NOT EXISTS idx_turns_provider  ON turns(provider);
        "#,
    )
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    spec: &str,
) -> rusqlite::Result<()> {
    let exists: bool = conn
        .prepare(&format!("PRAGMA table_info({table})"))?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .any(|c| c == column);
    if !exists {
        conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {spec};"))?;
    }
    Ok(())
}

fn confidence_str(c: AttributionConfidence) -> &'static str {
    match c {
        AttributionConfidence::High => "high",
        AttributionConfidence::Medium => "medium",
        AttributionConfidence::Low => "low",
    }
}

fn confidence_from_str(s: Option<&str>) -> AttributionConfidence {
    match s.unwrap_or("low") {
        "high" => AttributionConfidence::High,
        "medium" => AttributionConfidence::Medium,
        _ => AttributionConfidence::Low,
    }
}

fn parse_ts(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn opt_string(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
