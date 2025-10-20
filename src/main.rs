// ----- Modules ----- //

mod sys_resource;
mod sys_console;
mod sys_bot;
mod sys_core;
mod sys_session;
mod sys_db;
mod sys_dashboard;

// ----- Imports ----- //

use crate::{sys_core::{load_config, Server}, sys_session::session_state::{get_session_manager, init_session_manager}};

// ----- Lifecycle ----- //

fn main() {
    load_config("cfg/config.json");
    init_session_manager(); // ← Init global session manager
    std::thread::spawn(|| {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            get_session_manager().tick(); // ← Purge expired sessions
        }
    });

    // Get the port from the config
    let port = crate::sys_core::core_config::get_config().port;

    let address = format!("127.0.0.1:{}", port);
    let server = Server::new(&address, "static");
    server.run();
}
