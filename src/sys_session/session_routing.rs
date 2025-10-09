// ----- Imports ----- //

use crate::{
    sys_bot::{
        bot_instructions::get_instructions, 
        bot_openai::ask_openai, 
        bot_reply::BotReply
    }, 
    sys_core::{
        core_responses::{
            response_not_found, 
            response_ok
        }, 
        HttpResponse
    }, 
    sys_session::session_state::get_session_manager
};

use serde_json::json;

// ----- Session Route Handlers ----- //

pub fn handle_session_start() -> HttpResponse {
    let session = get_session_manager().create_session(300);
    let instructions = get_instructions("cfg/bots/instructions_sales.txt");
    let first_message = get_instructions("cfg/bots/msg_introduction.txt");
    
    let mut session = session;
    session.session_chat = format!("DEVELOPER: {}\n BOT: {}", instructions, first_message);

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
                    get_convo_summary(session);

                    // Remove session after summary
                    let end_id = session.session_id.clone();
                    sessions.remove(&end_id);

                    let response_json = json!({
                        "reply": cleaned_reply.reply_string,
                        "session_ended": true
                    });

                    return response_ok(
                        "application/json; charset=utf-8",
                        response_json.to_string().into_bytes(),
                    );
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

// ----- Conversation Summary Logic ----- //

fn get_convo_summary(session: &crate::sys_session::session_state::Session) {
    println!("========== Conversation Summary ==========");
    println!("Session ID: {}", session.session_id);
    println!("Transcript:\n{}", session.session_chat);

    // --- Generate AI Summary ---
    if let Some(summary) = generate_convo_summary(&session.session_chat) {
        println!("\n--- AI Generated Summary ---\n{}\n", summary);
    } else {
        println!("\n--- AI Generated Summary ---\n[Summary generation failed or unavailable]\n");
    }

    println!("==========================================");
}

fn generate_convo_summary(session_chat: &str) -> Option<String> {
    use crate::sys_bot::bot_openai::ask_openai;

    // --- Load summary instructions ---
    let instructions = get_instructions("cfg/bots/instructions_summary.txt");

    // --- Prepare message context for OpenAI ---
    let messages = vec![
        ("system", instructions.as_str()),
        ("user", session_chat),
    ];

    // --- Request OpenAI summary ---
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