use serde::Deserialize;
use std::fs;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "botAPIKey")]
    pub bot_apikey: String,
}

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub fn load_config(path: &str) {
    let contents = fs::read_to_string(path).expect(&format!("Failed to read config file {}", path));

    let parsed: AppConfig = serde_json::from_str(&contents).expect("Invalid config format");

    CONFIG.set(parsed).expect("Config already initialized");
}

pub fn get_config() -> &'static AppConfig {
    CONFIG.get().expect("Config not initialized")
}
