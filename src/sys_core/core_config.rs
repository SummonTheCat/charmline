use serde::Deserialize;
use std::env;
use std::fs;
use std::sync::OnceLock;

/// Configuration file structure.
/// Now uses `port` (from cfg/config.json)
/// and loads `bot_apikey` from the environment variable BOT_API_KEY.

const BOT_API_KEY_ENV: &str = "CHARMLINE_BOT_KEY";

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "port")]
    pub port: u16,
    #[serde(skip)]
    pub bot_apikey: String,
}

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// Load configuration:
/// - Reads `port` from the JSON config file.
/// - Reads `BOT_API_KEY_ENV` from the environment (required).
pub fn load_config(path: &str) {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read config file {}", path));
    let mut parsed: AppConfig =
        serde_json::from_str(&contents).expect("Invalid config format (expected JSON)");

    let env_key = env::var(BOT_API_KEY_ENV)
        .unwrap_or_else(|_| panic!("Environment variable {} is not set", BOT_API_KEY_ENV));
    parsed.bot_apikey = env_key;

    CONFIG.set(parsed).expect("Config already initialized");
}

pub fn get_config() -> &'static AppConfig {
    CONFIG.get().expect("Config not initialized")
}
