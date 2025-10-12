use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// Represents a stored session row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRow {
    pub session_id: String,
    pub session_transcript: String,
    pub session_start: String,
    pub session_end: String,
    pub caller_name: Option<String>,
    pub caller_number: Option<String>,
    pub caller_company: Option<String>,
    pub summary_solution_type: Option<String>,
    pub summary_project_details: Option<String>,
    pub summary_additional_notes: Option<String>,
    pub summary_tags: Option<String>, // Stored as comma-separated string or JSON array
}

/// Initialize database and create table if it doesn’t exist
pub fn init_database() -> Result<Connection> {
    // Locate [exe]/data/sessiondata.db
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    let data_dir = exe_dir.join("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    }

    let db_path = data_dir.join("sessiondata.db");
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            session_id TEXT PRIMARY KEY,
            session_transcript TEXT NOT NULL,
            session_start TEXT NOT NULL,
            session_end TEXT NOT NULL,
            caller_name TEXT,
            caller_number TEXT,
            caller_company TEXT,
            summary_solution_type TEXT,
            summary_project_details TEXT,
            summary_additional_notes TEXT,
            summary_tags TEXT
        );
        "#,
    )?;

    Ok(conn)
}

/// Insert or replace a session entry
pub fn insert_session(conn: &Connection, session: &SessionRow) -> Result<()> {
    conn.execute(
        r#"
        INSERT OR REPLACE INTO sessions (
            session_id,
            session_transcript,
            session_start,
            session_end,
            caller_name,
            caller_number,
            caller_company,
            summary_solution_type,
            summary_project_details,
            summary_additional_notes,
            summary_tags
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        "#,
        params![
            session.session_id,
            session.session_transcript,
            session.session_start,
            session.session_end,
            session.caller_name,
            session.caller_number,
            session.caller_company,
            session.summary_solution_type,
            session.summary_project_details,
            session.summary_additional_notes,
            session.summary_tags,
        ],
    )?;
    Ok(())
}

/// Fetch a session by its ID
pub fn get_session_by_id(conn: &Connection, id: &str) -> Result<Option<SessionRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            session_id,
            session_transcript,
            session_start,
            session_end,
            caller_name,
            caller_number,
            caller_company,
            summary_solution_type,
            summary_project_details,
            summary_additional_notes,
            summary_tags
        FROM sessions WHERE session_id = ?;
        "#,
    )?;

    let result = stmt.query_row(params![id], |row| {
        Ok(SessionRow {
            session_id: row.get(0)?,
            session_transcript: row.get(1)?,
            session_start: row.get(2)?,
            session_end: row.get(3)?,
            caller_name: row.get(4)?,
            caller_number: row.get(5)?,
            caller_company: row.get(6)?,
            summary_solution_type: row.get(7)?,
            summary_project_details: row.get(8)?,
            summary_additional_notes: row.get(9)?,
            summary_tags: row.get(10)?,
        })
    }).optional()?;

    Ok(result)
}

/// Fetch all sessions
pub fn get_all_sessions(conn: &Connection) -> Result<Vec<SessionRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            session_id,
            session_transcript,
            session_start,
            session_end,
            caller_name,
            caller_number,
            caller_company,
            summary_solution_type,
            summary_project_details,
            summary_additional_notes,
            summary_tags
        FROM sessions;
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(SessionRow {
            session_id: row.get(0)?,
            session_transcript: row.get(1)?,
            session_start: row.get(2)?,
            session_end: row.get(3)?,
            caller_name: row.get(4)?,
            caller_number: row.get(5)?,
            caller_company: row.get(6)?,
            summary_solution_type: row.get(7)?,
            summary_project_details: row.get(8)?,
            summary_additional_notes: row.get(9)?,
            summary_tags: row.get(10)?,
        })
    })?;

    Ok(rows.filter_map(Result::ok).collect())
}
