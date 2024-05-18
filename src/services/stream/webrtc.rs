use auth::{self, SimpleTokenAuthenticator};
use streamhub::StreamsHub;
use tracing::instrument::WithSubscriber;
use xwebrtc::webrtc::WebRTCServer;

pub async fn start_server(conf: &crate::config::AppConfig) -> anyhow::Result<()> {
    let listen_port = conf.stream.port;
    let listen_host = conf.stream.host.clone();

    let mut stream_hub = StreamsHub::new(None);
    let sender = stream_hub.get_hub_event_sender();

    let new_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    new_rt.spawn(async move { stream_hub.run().await }.with_current_subscriber());

    let authenticator = SimpleTokenAuthenticator::new("123456".to_string());
    let mut webrtc_server = WebRTCServer::new(
        format!("{}:{}", listen_host, listen_port),
        sender,
        Some(authenticator),
    );

    webrtc_server.run().await
}
