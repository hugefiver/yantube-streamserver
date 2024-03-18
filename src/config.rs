pub mod log;
pub mod server;
pub mod stream;

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case", default)]
pub struct AppConfig {
    pub debug: bool,

    pub log: log::LogConf,
    pub server: server::ServerConf,
    pub stream: stream::StreamConf,
}

impl AppConfig {
    pub fn load_from_config(config: config::Config) -> anyhow::Result<Self> {
        config
            .try_deserialize()
            .with_context(|| "Failed to deserialize config")
    }

    pub fn load_from_file(file_path: Option<&str>, env_prefix: &str) -> anyhow::Result<Self> {
        let conf = config::Config::builder()
            .with(|c| {
                if let Some(f) = file_path {
                    c.add_source(config::File::with_name(f))
                } else{
                    c
                }
            })
            .add_source(config::Environment::with_prefix(env_prefix).separator("_"))
            .build()
            .with_context(|| "Failed to load config")?;
        Self::load_from_config(conf)
    }
}

trait With<T> {
    fn with<F>(self, f: F) -> T
    where
        F: Fn(Self) -> T,
        Self: Sized;
}

impl<T, O> With<T> for O {
    fn with<F>(self, f: F) -> T
    where
        F: Fn(Self) -> T,
    {
        f(self)
    }
}
