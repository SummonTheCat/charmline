// ----- Imports ----- //

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use crate::{
    sys_core::{
        core_responses::response_method_not_allowed,
        core_routing::handle_route,
    },
    sys_resource::CachedLoader,
};

// ----- Structs ----- //

pub struct Server {
    address: String,
    loader: Arc<CachedLoader>,
}

// ----- Implementations ----- //

impl Server {
    pub fn new(address: &str, base_dir: &str) -> Self {
        let loader = Arc::new(CachedLoader::new(base_dir));
        Self {
            address: address.to_string(),
            loader,
        }
    }

    pub fn run(&self) {
        let listener = TcpListener::bind(&self.address).expect("Failed to bind port");
        println!("Charmline running at http://{}/", self.address);

        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let loader = Arc::clone(&self.loader);
                thread::spawn(move || handle_client(stream, loader));
            }
        }
    }
}

// ----- Lifecycle ----- //

fn handle_client(mut stream: TcpStream, loader: Arc<CachedLoader>) {
    let mut buffer = [0; 8192];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(size) => size,
        Err(_) => return,
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    // Parse the request line
    let mut lines = request.lines();
    let first_line = lines.next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    // Extract body (after a blank line)
    let body_start = request.find("\r\n\r\n").map(|i| i + 4);
    let body = body_start
        .map(|i| request[i..].to_string())
        .unwrap_or_default();

    let response = match method {
        "GET" | "POST" => handle_route(path, &loader, &body),
        _ => response_method_not_allowed(),
    };

    let _ = stream.write_all(&response.to_bytes());
}
