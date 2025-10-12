// ----- Imports ----- //

use crate::sys_core::core_responses::response_ok;
use crate::sys_core::HttpResponse;
use crate::sys_db::db_sessions::init_database;
use crate::sys_db::db_session_dashboard::*;
use serde_json::json;

// ----- Dashboard Handlers ----- //

/// GET /dashboard/stats
/// Returns overall statistics about all sessions.
pub fn handle_dashboard_stats() -> HttpResponse {
    match init_database()
        .and_then(|conn| get_session_stats(&conn))
    {
        Ok(stats) => {
            let json = serde_json::to_string(&stats).unwrap_or_else(|_| "{}".to_string());
            response_ok("application/json; charset=utf-8", json.into_bytes())
        }
        Err(e) => {
            let err = json!({ "error": format!("Failed to get stats: {}", e) });
            response_ok("application/json; charset=utf-8", err.to_string().into_bytes())
        }
    }
}

/// GET /dashboard/top_companies
/// Optional body: { "limit": 10 }
pub fn handle_dashboard_top_companies(body: &str) -> HttpResponse {
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let limit = parsed.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    match init_database()
        .and_then(|conn| get_top_companies(&conn, limit))
    {
        Ok(rows) => {
            let json = serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string());
            response_ok("application/json; charset=utf-8", json.into_bytes())
        }
        Err(e) => {
            let err = json!({ "error": format!("Failed to get top companies: {}", e) });
            response_ok("application/json; charset=utf-8", err.to_string().into_bytes())
        }
    }
}

/// GET /dashboard/tags
pub fn handle_dashboard_tags() -> HttpResponse {
    match init_database()
        .and_then(|conn| get_tag_frequencies(&conn))
    {
        Ok(rows) => {
            let json = serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string());
            response_ok("application/json; charset=utf-8", json.into_bytes())
        }
        Err(e) => {
            let err = json!({ "error": format!("Failed to get tag frequencies: {}", e) });
            response_ok("application/json; charset=utf-8", err.to_string().into_bytes())
        }
    }
}

/// GET /dashboard/solutions
pub fn handle_dashboard_solutions() -> HttpResponse {
    match init_database()
        .and_then(|conn| get_solution_type_frequencies(&conn))
    {
        Ok(rows) => {
            let json = serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string());
            response_ok("application/json; charset=utf-8", json.into_bytes())
        }
        Err(e) => {
            let err = json!({ "error": format!("Failed to get solution type frequencies: {}", e) });
            response_ok("application/json; charset=utf-8", err.to_string().into_bytes())
        }
    }
}

/// GET /dashboard/sessions_by_day
/// Optional body: { "days": 7 }
pub fn handle_dashboard_sessions_by_day(body: &str) -> HttpResponse {
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let days = parsed.get("days").and_then(|v| v.as_i64()).unwrap_or(7);

    match init_database()
        .and_then(|conn| get_sessions_by_day(&conn, days))
    {
        Ok(rows) => {
            let json = json!({ "sessions_by_day": rows });
            response_ok("application/json; charset=utf-8", json.to_string().into_bytes())
        }
        Err(e) => {
            let err = json!({ "error": format!("Failed to get sessions by day: {}", e) });
            response_ok("application/json; charset=utf-8", err.to_string().into_bytes())
        }
    }
}
