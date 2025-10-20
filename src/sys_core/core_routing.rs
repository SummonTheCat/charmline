// ----- Imports ----- //

use std::path::Path;
use std::sync::Arc;

use crate::sys_console::handle_api_command;
use crate::sys_core::core_responses::{response_not_found, response_ok};
use crate::sys_dashboard::dashboard_handlers::{
    handle_dashboard_sessions_by_day, handle_dashboard_solutions, handle_dashboard_stats,
    handle_dashboard_tags, handle_dashboard_top_companies,
};
use crate::sys_resource::CachedLoader;
use crate::sys_session::session_handlers::{
    handle_session_get, handle_session_get_artifact, handle_session_list_artifacts,
    handle_session_sendinput, handle_session_start,
};

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
    // Strip query parameters (anything after '?')
    let clean_path = path.split('?').next().unwrap_or("/");

    match clean_path {
        "/" => match loader.load("index.html") {
            Some(file) => response_ok("text/html", file.bytes),
            None => response_not_found("index.html not found"),
        },
        "/api/cmd" => handle_api_command(body),
        "/api/session/start" => handle_session_start(),
        "/api/session/get" => handle_session_get(body),
        "/api/session/sendinput" => handle_session_sendinput(body),
        "/api/session/listartifacts" => handle_session_list_artifacts(body),
        "/api/session/getartifact" => handle_session_get_artifact(body),
        "/api/dashboard/stats" => handle_dashboard_stats(),
        "/api/dashboard/top_companies" => handle_dashboard_top_companies(body),
        "/api/dashboard/tags" => handle_dashboard_tags(),
        "/api/dashboard/solutions" => handle_dashboard_solutions(),
        "/api/dashboard/sessions_by_day" => handle_dashboard_sessions_by_day(body),
        _ => serve_static(clean_path, loader),
    }
}

fn serve_static(path: &str, loader: &Arc<CachedLoader>) -> HttpResponse {
    // Strip query params again for safety in direct calls
    let path = path.split('?').next().unwrap_or("").trim_start_matches('/');
    println!("[serve_static] Requested path: {}", path);

    if path.contains("..") {
        println!("[serve_static] Rejected invalid path with '..'");
        return response_not_found("Invalid path");
    }

    let content_type = content_type_for(path);
    println!("[serve_static] Content type: {}", content_type);

    // Try direct file first
    println!("[serve_static] Attempting direct load: {}", path);
    if let Some(file) = loader.load(path) {
        println!("[serve_static] Found file directly: {}", path);
        return response_ok(content_type, file.bytes);
    } else {
        println!("[serve_static] Direct load failed for: {}", path);
    }

    // Fallback: /pages/{path}.html if no extension present
    if !path.contains('.') {
        let html_path = format!("pages/{}.html", path);
        println!("[serve_static] Attempting fallback: {}", html_path);
        let content_type = "text/html; charset=utf-8";

        if let Some(file) = loader.load(&html_path) {
            println!("[serve_static] Found fallback HTML: {}", html_path);
            return response_ok(content_type, file.bytes);
        } else {
            println!("[serve_static] Fallback failed for: {}", html_path);
        }
    }

    println!("[serve_static] File not found for any path");
    response_not_found("File not found")
}

// ----- Helpers ----- //

fn content_type_for(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
    {
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
