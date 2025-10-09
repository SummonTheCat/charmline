use crate::sys_core::get_config;
use serde::{Deserialize, Serialize};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const OPENAI_MODEL: &str = "gpt-3.5-turbo";
const OPENAI_MAX_TOKENS: u32 = 512;

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

pub fn ask_openai(messages: Vec<(&str, &str)>) -> Result<String, String> {
    let config = get_config();
    let client = reqwest::blocking::Client::new();

    let req_messages: Vec<OpenAIMessage> = messages
        .into_iter()
        .map(|(role, content)| OpenAIMessage {
            role: role.to_string(),
            content: content.to_string(),
        })
        .collect();

    let request_body = OpenAIRequest {
        model: OPENAI_MODEL.to_string(),
        messages: req_messages,
        max_tokens: OPENAI_MAX_TOKENS,
    };

    let response = client
        .post(OPENAI_API_URL)
        .bearer_auth(&config.bot_apikey)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .map_err(|e| format!("HTTP error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("OpenAI error: {}", response.status()));
    }

    let parsed: OpenAIResponse = response
        .json()
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(parsed
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "(no response)".to_string()))
}
