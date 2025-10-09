use crate::{sys_bot::bot_openai::ask_openai, sys_core::{core_responses::response_ok, HttpResponse}};
use serde_json::Value;

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
    let mut parts = input.split_whitespace();
    let cmd = parts.next().unwrap_or("").to_string();
    let args = parts.map(|s| s.to_string()).collect();
    (cmd, args)
}

fn execute_command(_cmd: &str, _args: &[String]) -> String {
    // Switch for command execution logic
    let response = match _cmd {
        "help" => command_help(_args),
        "test" => command_test(_args),
        _ => command_not_supported(_cmd, _args),
    };
    response
}

// ----- Command Implementations ----- //

fn command_help(_args: &[String]) -> String {
    r#"{"message":"Available commands: help, test"}"#.to_string()
}

fn command_not_supported(cmd: &str, _args: &[String]) -> String {
    format!(r#"{{"message":"Command not supported: {}"}}"#, cmd)
}

// ----- Tests ----- //
fn command_test(args: &[String]) -> String {
    // Switch on the args for the test target
    let response = match args.get(0).map(String::as_str) {
        Some("echo") => command_test_echo(&args[1..]),
        _ => r#"{"message":"Test what? Available: echo"}"#.to_string(),
    };
    response
}



fn command_test_echo(args: &[String]) -> String {
    let input = args.join(" ");
    let mut result = format!(r#"{{"message":"Echo: {}"}}"#, input);

    if !input.trim().is_empty() {
        // Convert input to Vec<(&str, &str)> as expected by ask_openai
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

