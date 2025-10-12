// ----- Imports ----- //

use rusqlite::{params, Connection, Result};
use serde::Serialize;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

// ----- Structs for Returned Data ----- //

#[derive(Debug, Serialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub unique_callers: usize,
    pub unique_companies: usize,
    pub avg_duration_seconds: f64,
    pub sessions_today: usize,
    pub sessions_this_week: usize,
}

#[derive(Debug, Serialize)]
pub struct CompanyFrequency {
    pub company: String,
    pub session_count: usize,
}

#[derive(Debug, Serialize)]
pub struct TagFrequency {
    pub tag: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct SolutionTypeFrequency {
    pub solution_type: String,
    pub count: usize,
}

// ----- Core Aggregation Functions ----- //

/// Compute general statistics (counts, averages, etc.)
pub fn get_session_stats(conn: &Connection) -> Result<SessionStats> {
    let mut total_sessions = 0usize;
    let mut total_duration_secs = 0i64;
    let mut unique_callers = std::collections::HashSet::new();
    let mut unique_companies = std::collections::HashSet::new();

    let mut stmt = conn.prepare(
        r#"
        SELECT
            session_start,
            session_end,
            caller_number,
            caller_company
        FROM sessions;
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        let start: String = row.get(0)?;
        let end: String = row.get(1)?;
        let caller_number: Option<String> = row.get(2)?;
        let caller_company: Option<String> = row.get(3)?;
        Ok((start, end, caller_number, caller_company))
    })?;

    for row in rows.flatten() {
        total_sessions += 1;
        if let (Ok(start), Ok(end)) = (
            DateTime::parse_from_rfc3339(&row.0),
            DateTime::parse_from_rfc3339(&row.1),
        ) {
            total_duration_secs += (end.timestamp() - start.timestamp()).max(0);
        }

        if let Some(num) = row.2 {
            unique_callers.insert(num);
        }
        if let Some(comp) = row.3 {
            unique_companies.insert(comp);
        }
    }

    let avg_duration = if total_sessions > 0 {
        total_duration_secs as f64 / total_sessions as f64
    } else {
        0.0
    };

    // Compute sessions by date ranges
    let now = Utc::now();
    let today_start = now.date_naive();
    let week_start = today_start - Duration::days(7);

    let sessions_today = conn.query_row(
        r#"
        SELECT COUNT(*) FROM sessions
        WHERE DATE(session_start) = DATE('now', 'localtime');
        "#,
        [],
        |r| r.get::<_, usize>(0),
    ).unwrap_or(0);

    let sessions_this_week = conn.query_row(
        r#"
        SELECT COUNT(*) FROM sessions
        WHERE DATE(session_start) >= DATE('now', '-7 days', 'localtime');
        "#,
        [],
        |r| r.get::<_, usize>(0),
    ).unwrap_or(0);

    Ok(SessionStats {
        total_sessions,
        unique_callers: unique_callers.len(),
        unique_companies: unique_companies.len(),
        avg_duration_seconds: avg_duration,
        sessions_today,
        sessions_this_week,
    })
}

/// Top N companies by session count
pub fn get_top_companies(conn: &Connection, limit: usize) -> Result<Vec<CompanyFrequency>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT caller_company, COUNT(*) as count
        FROM sessions
        WHERE caller_company IS NOT NULL AND caller_company != ''
        GROUP BY caller_company
        ORDER BY count DESC
        LIMIT ?;
        "#,
    )?;

    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok(CompanyFrequency {
            company: row.get(0)?,
            session_count: row.get(1)?,
        })
    })?;

    Ok(rows.filter_map(Result::ok).collect())
}

/// Tag frequency counts (across all sessions)
pub fn get_tag_frequencies(conn: &Connection) -> Result<Vec<TagFrequency>> {
    let mut tag_counts: HashMap<String, usize> = HashMap::new();

    let mut stmt = conn.prepare("SELECT summary_tags FROM sessions;")?;
    let rows = stmt.query_map([], |row| Ok(row.get::<_, Option<String>>(0)?))?;

    for tag_string in rows.flatten().flatten() {
        for tag in tag_string.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            *tag_counts.entry(tag.to_string()).or_insert(0) += 1;
        }
    }

    let mut tags: Vec<TagFrequency> = tag_counts
        .into_iter()
        .map(|(tag, count)| TagFrequency { tag, count })
        .collect();

    tags.sort_by(|a, b| b.count.cmp(&a.count));
    Ok(tags)
}

/// Most common solution types
pub fn get_solution_type_frequencies(conn: &Connection) -> Result<Vec<SolutionTypeFrequency>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT summary_solution_type, COUNT(*) as count
        FROM sessions
        WHERE summary_solution_type IS NOT NULL AND summary_solution_type != ''
        GROUP BY summary_solution_type
        ORDER BY count DESC;
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(SolutionTypeFrequency {
            solution_type: row.get(0)?,
            count: row.get(1)?,
        })
    })?;

    Ok(rows.filter_map(Result::ok).collect())
}

/// Session volume by day (for graphs)
pub fn get_sessions_by_day(conn: &Connection, days: i64) -> Result<Vec<(String, usize)>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT DATE(session_start) as day, COUNT(*) as count
        FROM sessions
        WHERE DATE(session_start) >= DATE('now', ?)
        GROUP BY day
        ORDER BY day ASC;
        "#,
    )?;

    let rows = stmt.query_map(params![format!("-{} days", days)], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
    })?;

    Ok(rows.filter_map(Result::ok).collect())
}
