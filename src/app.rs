use crate::{config, services};

pub async fn start_app(conf: &config::AppConfig) {
    let span = tracing::span!(tracing::Level::DEBUG, "live_stream_app");
    let _ = span.enter();

    // services::stream::pull_stream::rtmp_server(&conf)
    //     .await
    //     .expect("TODO: panic message");

    services::stream::webrtc::start_server(conf)
        .await
        .expect("TODO: panic message");
}
