use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct ServerConf {
    pub host: String,
    pub port: u16,
    pub api_addr: String,
    pub api_secret: String,
}

impl Default for ServerConf {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            api_addr: "http://[::1]:9082".to_string(),
            api_secret: "secret".to_string(),
        }
    }
}

impl ServerConf {
    pub fn get_self_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
