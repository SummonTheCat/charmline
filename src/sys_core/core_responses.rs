// ----- Imports ----- //

use crate::sys_core::core_routing::HttpResponse;

// ----- Responses ----- //
pub fn response_ok(content_type: &'static str, body: Vec<u8>) -> HttpResponse {
    HttpResponse {
        status_line: "HTTP/1.1 200 OK",
        content_type,
        body,
    }
}

pub fn response_not_found(msg: &str) -> HttpResponse {
    HttpResponse {
        status_line: "HTTP/1.1 404 NOT FOUND",
        content_type: "text/plain; charset=utf-8",
        body: msg.as_bytes().to_vec(),
    }
}

pub fn response_method_not_allowed() -> HttpResponse {
    HttpResponse {
        status_line: "HTTP/1.1 405 METHOD NOT ALLOWED",
        content_type: "text/plain; charset=utf-8",
        body: b"Method not allowed".to_vec(),
    }
}