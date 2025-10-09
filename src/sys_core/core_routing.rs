// ----- Imports ----- //

use std::path::Path;
use std::sync::Arc;

use crate::sys_bot::bot_handlers::handle_api_command;
use crate::sys_core::core_responses::{response_not_found, response_ok};
use crate::sys_resource::CachedLoader;
use crate::sys_session::session_routing::{handle_session_get, handle_session_sendinput, handle_session_start};

// ----- Structs ----- //

pub struct HttpResponse {
    pub status_line: &'static str,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

// ----- Implementations ----- //

impl HttpResponse {
    pub fn to_bytes(&self) -> Vec<u8> {
        let header = format!(
            "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
            self.status_line,
            self.body.len(),
            self.content_type
        );
        let mut response = header.into_bytes();
        response.extend_from_slice(&self.body);
        response
    }
}

// ----- Routing ----- //

pub fn handle_route(path: &str, loader: &Arc<CachedLoader>, body: &str) -> HttpResponse {
    match path {
        "/" => match loader.load("index.html") {
            Some(file) => response_ok("text/html", file.bytes),
            None => response_not_found("index.html not found"),
        },
        "/api/cmd" => handle_api_command(body),
        "/api/session/start" => handle_session_start(),
        "/api/session/get" => handle_session_get(body),
        "/api/session/sendinput" => handle_session_sendinput(body),
        _ => serve_static(path, loader),
    }
}

// Serve static files (html, css, js, images, etc.)
fn serve_static(path: &str, loader: &Arc<CachedLoader>) -> HttpResponse {
    let path = path.trim_start_matches('/');

    if path.contains("..") {
        return response_not_found("Invalid path");
    }

    let content_type = content_type_for(path);

    match loader.load(path) {
        Some(file) => response_ok(content_type, file.bytes),
        None => response_not_found("File not found"),
    }
}

// ----- Helpers ----- //

fn content_type_for(path: &str) -> &'static str {
    match Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}
