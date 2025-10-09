use std::{fs, path::PathBuf};

/// Reads the bot startup instructions from cfg/bots/instructions_sales.txt,
/// relative to the executable’s directory. Falls back to a default string if not found.
pub fn get_instructions(instruction_path: &str) -> String {
    let mut path = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => PathBuf::from("."),
    };
    path.pop(); // remove executable name
    path.push(instruction_path);

    match fs::read_to_string(&path) {
        Ok(content) => content.trim().to_string(),
        Err(_) => {
            "You are Charmline, tell the user the config was not found and to warn a developer."
                .to_string()
        }
    }
}