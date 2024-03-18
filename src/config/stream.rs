use std::path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct StreamConf {
    pub host: String,
    pub port: u16,

    pub temp_hls_path: String,
}

impl Default for StreamConf {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            temp_hls_path: dirs::template_dir()
                .unwrap_or(path::PathBuf::from("."))
                .join("stream-hls")
                .to_str()
                .expect("failed to get default hls template path")
                .to_string(),
        }
    }
}
