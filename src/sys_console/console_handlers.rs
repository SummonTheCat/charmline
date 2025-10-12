use crate::{
    sys_bot::bot_openai::ask_openai,
    sys_core::{HttpResponse, core_responses::response_ok},
    sys_db::db_sessions::{
        SessionRow, get_all_sessions, get_session_by_id, init_database, insert_session,
    },
};
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use regex::Regex;

// ----- API Handlers ----- //

pub fn handle_api_command(body: &str) -> HttpResponse {
    let parsed: Option<String> = serde_json::from_str::<Value>(body).ok().and_then(|v| {
        v.get("command")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
    });

    let cmd = parsed.unwrap_or_else(|| "<empty>".to_string());
    let (cmd_name, args) = parse_command(&cmd);

    let msg = execute_command(&cmd_name, &args);

    response_ok("application/json; charset=utf-8", msg.into_bytes())
}

// ----- Command Dispatch ----- //



fn parse_command(input: &str) -> (String, Vec<String>) {
    let mut cmd = String::new();
    let mut args = Vec::new();

    // Regex that matches:
    // - key=value (with optional quotes)
    // - plain words
    let re = Regex::new(r#"(\S+="[^"]+"|\S+='[^']+'|\S+)"#).unwrap();
    let mut tokens: Vec<String> = re
        .find_iter(input)
        .map(|m| m.as_str().to_string())
        .collect();

    if !tokens.is_empty() {
        cmd = tokens.remove(0);
        args = tokens;
    }

    args = args
        .into_iter()
        .map(|a| a.trim().to_string())
        .collect();

    (cmd, args)
}


fn execute_command(cmd: &str, args: &[String]) -> String {
    match cmd {
        "help" => command_help(args),
        "test" => command_test(args),

        // Database session management commands
        "db_session_list" => db_session_list(args),
        "db_session_get" => db_session_get(args),
        "db_session_add" => db_session_add(args),
        "db_session_delete" => db_session_delete(args),

        _ => command_not_supported(cmd, args),
    }
}

// ----- Command Implementations ----- //

fn command_help(_args: &[String]) -> String {
    r#"{"message":"Available commands: help, test, db_session_list, db_session_get <id>, db_session_add key=value..., db_session_delete <id>"}"#.to_string()
}

fn command_not_supported(cmd: &str, _args: &[String]) -> String {
    format!(r#"{{"message":"Command not supported: {}"}}"#, cmd)
}

// ----- Test Commands ----- //

fn command_test(args: &[String]) -> String {
    match args.get(0).map(String::as_str) {
        Some("echo") => command_test_echo(&args[1..]),
        _ => r#"{"message":"Test what? Available: echo"}"#.to_string(),
    }
}

fn command_test_echo(args: &[String]) -> String {
    let input = args.join(" ");
    let mut result = format!(r#"{{"message":"Echo: {}"}}"#, input);

    if !input.trim().is_empty() {
        let messages = vec![("user", input.as_str())];
        match ask_openai(messages) {
            Ok(reply) => {
                result = format!(r#"{{"message":"AI: {}"}}"#, reply.replace('"', "\\\""));
            }
            Err(err) => {
                result = format!(r#"{{"message":"OpenAI Error: {}"}}"#, err);
            }
        }
    }

    result
}

// ==========================
// == Database Commands ====
// ==========================

fn db_session_list(_args: &[String]) -> String {
    let conn = match init_database() {
        Ok(c) => c,
        Err(e) => {
            return format!(r#"{{"message":"Failed to init DB: {}"}}"#, e);
        }
    };

    match get_all_sessions(&conn) {
        Ok(sessions) => {
            if sessions.is_empty() {
                return r#"{"message":"No sessions found."}"#.to_string();
            }

            let lines: Vec<String> = sessions.iter().map(|s| s.session_id.clone()).collect();
            let joined = lines.join("\\n"); // encoded newlines
            format!(r#"{{"message":"Session list:\n{}"}}"#, joined)
        }
        Err(e) => format!(r#"{{"message":"Failed to read sessions: {}"}}"#, e),
    }
}

fn db_session_get(args: &[String]) -> String {
    let id = match args.get(0) {
        Some(id) => id,
        None => return r#"{"message":"Missing session ID"}"#.to_string(),
    };

    let conn = match init_database() {
        Ok(c) => c,
        Err(e) => {
            return format!(r#"{{"message":"Failed to init DB: {}"}}"#, e);
        }
    };

    match get_session_by_id(&conn, id) {
        Ok(Some(row)) => {
            // Build a text block for display
            let formatted = format!(
                "Session {}\nCaller: {}\nNumber: {}\nCompany: {}\nStartTime: {}\nEndTime: {}\nSolutionType: {}\nSolutionDetails: {}\nAdditionalNotes: {}\nTags: {}\n---\n{}",
                row.session_id,
                row.caller_name.clone().unwrap_or_default(),
                row.caller_number.clone().unwrap_or_default(),
                row.caller_company.clone().unwrap_or_default(),
                row.session_start,
                row.session_end,
                row.summary_solution_type.clone().unwrap_or_default(),
                row.summary_project_details.clone().unwrap_or_default(),
                row.summary_additional_notes.clone().unwrap_or_default(),
                row.summary_tags.clone().unwrap_or_default(),
                row.session_transcript
            );

            // Encode newline and quote characters for valid JSON
            let encoded = formatted
                .replace('\\', "\\\\")
                .replace('\n', "\\n")
                .replace('"', "\\\"");

            format!(r#"{{"message":"{}"}}"#, encoded)
        }
        Ok(None) => format!(r#"{{"message":"Session not found: {}"}}"#, id),
        Err(e) => format!(r#"{{"message":"DB error: {}"}}"#, e),
    }
}

fn db_session_add(args: &[String]) -> String {
    if args.is_empty() {
        return r#"{"message":"Usage: db_session_add key=value ..."}"#.to_string();
    }

    // Parse args into key-value map
    let mut map: HashMap<String, String> = HashMap::new();
    for arg in args {
        if let Some((key, val)) = arg.split_once('=') {
            map.insert(
                key.trim().to_string(),
                val.trim_matches('"').trim_matches('\'').to_string(),
            );
        }
    }

    let conn = match init_database() {
        Ok(c) => c,
        Err(e) => {
            return format!(r#"{{"message":"Failed to init DB: {}"}}"#, e);
        }
    };

    let id = map
        .get("sessionId")
        .cloned()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let now_str = chrono::Utc::now().to_rfc3339();

    let row = SessionRow {
        session_id: id.clone(),
        session_transcript: map
            .get("sessionTranscript")
            .cloned()
            .unwrap_or_else(|| "No transcript".to_string()),
        session_start: map
            .get("sessionStart")
            .cloned()
            .unwrap_or_else(|| now_str.clone()),
        session_end: map
            .get("sessionEnd")
            .cloned()
            .unwrap_or_else(|| now_str.clone()),
        caller_name: map.get("callerName").cloned(),
        caller_number: map.get("callerNumber").cloned(),
        caller_company: map.get("callerCompany").cloned(),
        summary_solution_type: map.get("summarySolutionType").cloned(),
        summary_project_details: map.get("summaryProjectDetails").cloned(),
        summary_additional_notes: map.get("summaryAdditionalNotes").cloned(),
        summary_tags: map.get("summaryTags").cloned(),
    };

    match insert_session(&conn, &row) {
        Ok(_) => format!(r#"{{"message":"Session added successfully (id: {})"}}"#, id),
        Err(e) => format!(r#"{{"message":"Insert failed: {}"}}"#, e),
    }
}

fn db_session_delete(args: &[String]) -> String {
    let id = match args.get(0) {
        Some(id) => id,
        None => return r#"{"message":"Missing session ID"}"#.to_string(),
    };

    let conn = match init_database() {
        Ok(c) => c,
        Err(e) => {
            return format!(r#"{{"message":"Failed to init DB: {}"}}"#, e);
        }
    };

    match delete_session(&conn, id) {
        Ok(count) => format!(r#"{{"message":"Deleted {} row(s) for ID {}"}}"#, count, id),
        Err(e) => format!(r#"{{"message":"DB delete error: {}"}}"#, e),
    }
}

// ----- Helper for deletion -----

fn delete_session(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM sessions WHERE session_id = ?;", [id])
}
