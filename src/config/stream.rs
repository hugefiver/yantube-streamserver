use std::path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct StreamConf {
    pub host: String,
    pub port: u16,

    pub temp_hls_path: String,
    pub hls_fragment_seconds: i32,
    pub hls_fragment_max_count: i32,
}

impl Default for StreamConf {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8081,
            temp_hls_path: dirs::template_dir()
                .unwrap_or(path::PathBuf::from("."))
                .join("stream-hls")
                .to_str()
                .expect("failed to get default hls template path")
                .to_string(),
            hls_fragment_seconds: 1,
            hls_fragment_max_count: 10,
        }
    }
}
