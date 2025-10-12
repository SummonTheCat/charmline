// ----- Imports ----- //

use crate::{
    sys_bot::{bot_instructions::get_instructions, bot_openai::ask_openai, bot_reply::BotReply},
    sys_core::{
        HttpResponse,
        core_responses::{response_not_found, response_ok},
    },
    sys_session::session_state::{Session, SessionArtifact, SessionSummary, get_session_manager},
    sys_db::db_sessions::{SessionRow, init_database, insert_session}
};

use chrono::{DateTime, Utc};
use serde_json::json;
use std::{collections::HashMap, sync::MutexGuard, time::SystemTime};

// ----- Session Route Handlers ----- //

const SESSION_TIMEOUT_SECS: u64 = 300; // 5 minutes

pub fn handle_session_start() -> HttpResponse {
    let session = get_session_manager().create_session(SESSION_TIMEOUT_SECS);
    let first_message = get_instructions("cfg/bots/msg_introduction.txt");

    let mut session = session;
    session.session_chat = format!("Bot: {}", first_message.to_string());

    get_session_manager().update_session(session.clone());

    let json = json!({
        "session_id": session.session_id,
        "expires_in": session.time_remaining(),
        "chat": first_message
    });

    response_ok(
        "application/json; charset=utf-8",
        json.to_string().into_bytes(),
    )
}

pub fn handle_session_get(body: &str) -> HttpResponse {
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let session_id = parsed
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match get_session_manager().get_session(session_id) {
        Some(s) => {
            let json = json!({
                "session_id": s.session_id,
                "expires_in": s.time_remaining(),
                "chat": s.session_chat
            });
            response_ok(
                "application/json; charset=utf-8",
                json.to_string().into_bytes(),
            )
        }
        None => response_not_found("Session not found"),
    }
}

pub fn handle_session_sendinput(body: &str) -> HttpResponse {
    // Parse input JSON
    let input_data = InputData::from_json(body);

    let manager = get_session_manager();
    let mut sessions = manager.sessions.lock().unwrap();

    if let Some(session) = sessions.get_mut(&input_data.session_id) {
        let system_prompt = get_instructions("cfg/bots/instructions_sales.txt");

        let messages = vec![
            ("system", system_prompt.as_str()),
            ("user", session.session_chat.as_str()),
            ("user", &input_data.input),
        ];

        match ask_openai(messages) {
            Ok(reply) => {
                let cleaned_reply = BotReply::parse_reply(&reply);

                // Update chat history (keep full version including tags for internal context)
                session.session_chat = format!(
                    "{}\n\nUser: {}\nBot: {}\n",
                    session.session_chat.trim_end(),
                    &input_data.input,
                    cleaned_reply.reply_string
                );

                // --- Handle ENDCALL logic ---
                if cleaned_reply.is_endcall {
                    let session_clone = session.clone();
                    return end_convo(&mut sessions, &session_clone, cleaned_reply.reply_string);
                }

                // Debug log the history
                println!("--- Updated Session Chat ---\n{}", session.session_chat);

                // Normal continuation response
                let json = json!({ "reply": cleaned_reply.reply_string, "session_ended": false });
                response_ok(
                    "application/json; charset=utf-8",
                    json.to_string().into_bytes(),
                )
            }

            Err(err) => {
                let json = json!({
                    "error": format!("OpenAI Error: {}", err)
                });
                response_ok(
                    "application/json; charset=utf-8",
                    json.to_string().into_bytes(),
                )
            }
        }
    } else {
        response_not_found("Session not found")
    }
}

pub fn handle_session_list_artifacts(_body: &str) -> HttpResponse {
    // Connect to database
    let conn = match crate::sys_db::db_sessions::init_database() {
        Ok(c) => c,
        Err(e) => {
            return response_ok(
                "application/json; charset=utf-8",
                json!({ "error": format!("Failed to open database: {}", e) })
                    .to_string()
                    .into_bytes(),
            );
        }
    };

    match crate::sys_db::db_sessions::get_all_sessions(&conn) {
        Ok(rows) => {
            let artifacts: Vec<_> = rows
                .into_iter()
                .map(|r| {
                    let duration_seconds = {
                        if let (Ok(start), Ok(end)) = (
                            DateTime::parse_from_rfc3339(&r.session_start),
                            DateTime::parse_from_rfc3339(&r.session_end),
                        ) {
                            (end.timestamp() - start.timestamp()).max(0)
                        } else {
                            0
                        }
                    };

                    json!({
                        "session_id": r.session_id,
                        "company": r.caller_company.unwrap_or_default(),
                        "start_time": r.session_start,
                        "end_time": r.session_end,
                        "duration_seconds": duration_seconds
                    })
                })
                .collect();

            let json = json!({ "artifacts": artifacts });
            response_ok(
                "application/json; charset=utf-8",
                json.to_string().into_bytes(),
            )
        }
        Err(e) => response_ok(
            "application/json; charset=utf-8",
            json!({ "error": format!("Failed to query sessions: {}", e) })
                .to_string()
                .into_bytes(),
        ),
    }
}

pub fn handle_session_get_artifact(body: &str) -> HttpResponse {
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let session_id = parsed
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Connect to database
    let conn = match crate::sys_db::db_sessions::init_database() {
        Ok(c) => c,
        Err(e) => {
            return response_ok(
                "application/json; charset=utf-8",
                json!({ "error": format!("Failed to open database: {}", e) })
                    .to_string()
                    .into_bytes(),
            );
        }
    };

    // Fetch session by ID
    match crate::sys_db::db_sessions::get_session_by_id(&conn, session_id) {
        Ok(Some(row)) => {
            let json = json!({
                "sessionId": row.session_id,
                "sessionTranscript": row.session_transcript,
                "sessionStart": row.session_start,
                "sessionEnd": row.session_end,
                "summary": {
                    "callerName": row.caller_name.unwrap_or_default(),
                    "callerNumber": row.caller_number.unwrap_or_default(),
                    "company": row.caller_company.unwrap_or_default(),
                    "solutionType": row.summary_solution_type.unwrap_or_default(),
                    "projectDetails": row.summary_project_details.unwrap_or_default(),
                    "additionalNotes": row.summary_additional_notes.unwrap_or_default(),
                    "tags": row.summary_tags
                        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect::<Vec<_>>())
                        .unwrap_or_default(),
                }
            });

            response_ok(
                "application/json; charset=utf-8",
                json.to_string().into_bytes(),
            )
        }
        Ok(None) => response_not_found("Session not found in database"),
        Err(e) => response_ok(
            "application/json; charset=utf-8",
            json!({ "error": format!("Database error: {}", e) })
                .to_string()
                .into_bytes(),
        ),
    }
}

// ----- Conversation End / Summary Logic ----- //

pub fn end_convo(sessions: &mut MutexGuard<HashMap<String, Session>>, session: &Session, final_reply: String) -> HttpResponse {
    println!("========== Conversation End ==========");
    println!("Session ID: {}", session.session_id);

    // Clone for thread
    let session_clone = session.clone();

    // Remove session immediately
    sessions.remove(&session.session_id);

    // Spawn background thread for summary + DB save
    std::thread::spawn(move || {
        if let Err(e) = spawn_end_convo_async(session_clone) {
            eprintln!("Async end_convo error: {}", e);
        }
    });

    // Respond immediately (don’t block on summary or DB)
    let response_json = json!({
        "reply": final_reply,
        "session_ended": true
    });

    println!("Responding to client (non-blocking summary)...");
    response_ok(
        "application/json; charset=utf-8",
        response_json.to_string().into_bytes(),
    )
}

fn spawn_end_convo_async(session: Session) -> Result<(), String> {
    println!(
        "(Async) Generating session summary for {}",
        session.session_id
    );

    // Generate summary from transcript
    let summary_json_str = generate_convo_summary(&session.session_chat);
    let summary = summary_json_str
        .as_ref()
        .and_then(|s| serde_json::from_str::<SessionSummary>(s).ok())
        .unwrap_or_else(|| {
            eprintln!("(Async) Failed to parse AI summary — using fallback");
            SessionSummary {
                caller_name: "".into(),
                caller_number: "".into(),
                company: "".into(),
                solution_type: "".into(),
                project_details: "".into(),
                additional_notes: "".into(),
                tags: vec![],
            }
        });

    // Record session times
    let session_end = SystemTime::now();
    let session_start = session_end
        .checked_sub(
            session
                .session_timeout
                .duration_since(session.session_start),
        )
        .unwrap_or(session_end);

    let start_str = DateTime::<Utc>::from(session_start).to_rfc3339();
    let end_str = DateTime::<Utc>::from(session_end).to_rfc3339();

    // Build artifact
    let artifact = SessionArtifact {
        session_id: session.session_id.clone(),
        session_transcript: session.session_chat.clone(),
        session_start: start_str.clone(),
        session_end: end_str.clone(),
        summary: summary.clone(),
    };

    // --- DB Save ---
    if let Ok(conn) = init_database() {
        let db_row = SessionRow {
            session_id: artifact.session_id.clone(),
            session_transcript: artifact.session_transcript.clone(),
            session_start: start_str.clone(),
            session_end: end_str.clone(),
            caller_name: Some(summary.caller_name.clone()),
            caller_number: Some(summary.caller_number.clone()),
            caller_company: Some(summary.company.clone()),
            summary_solution_type: Some(summary.solution_type.clone()),
            summary_project_details: Some(summary.project_details.clone()),
            summary_additional_notes: Some(summary.additional_notes.clone()),
            summary_tags: Some(summary.tags.join(",")),
        };

        if let Err(e) = insert_session(&conn, &db_row) {
            eprintln!("(Async) DB insert failed: {}", e);
        } else {
            println!("(Async) Session {} saved to DB", artifact.session_id);
        }
    } else {
        eprintln!("(Async) Failed to open DB connection");
    }

    Ok(())
}

fn generate_convo_summary(session_chat: &str) -> Option<String> {
    use crate::sys_bot::bot_instructions::get_instructions;
    use crate::sys_bot::bot_openai::ask_openai;

    let instructions = get_instructions("cfg/bots/instructions_summary.txt");
    let messages = vec![("system", instructions.as_str()), ("user", session_chat)];

    match ask_openai(messages) {
        Ok(summary) => Some(summary.trim().to_string()),
        Err(err) => {
            eprintln!("OpenAI Summary Error: {}", err);
            None
        }
    }
}

// ----- Input Data Structure ----- //

struct InputData {
    session_id: String,
    input: String,
}

impl InputData {
    fn from_json(body: &str) -> Self {
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let session_id = parsed
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let input = parsed
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        InputData { session_id, input }
    }
}
