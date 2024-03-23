use crate::config;

struct StreamServerController {

}

pub async fn start_app(conf: config::AppConfig) {
    let span = tracing::span!(tracing::Level::DEBUG, "live_stream_app");
    let _ = span.enter();

    
}