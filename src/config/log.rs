use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct LogConf {
    pub level: String,
    pub file: String,
    pub rotate: bool,
}

impl Default for LogConf {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: "".to_string(),
            rotate: false,
        }
    }
}

pub fn str_to_level(s: &str) -> tracing::Level {
    match s.to_lowercase().as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        _ => tracing::Level::INFO,
    }
}
