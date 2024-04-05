use xwebrtc::webrtc::WebRTCServer;
use streamhub::StreamsHub;

pub async fn start_server(conf: &crate::config::AppConfig) -> anyhow::Result<()> {
    let listen_port = conf.stream.port;
    let listen_host = conf.stream.host.clone();

    let mut stream_hub = StreamsHub::new(None);
    let sender = stream_hub.get_hub_event_sender();
    tokio::spawn(async move { stream_hub.run().await });

    let mut webrtc_server = WebRTCServer::new(
        format!("{}:{}", listen_host, listen_port),
        sender,
        None,
    );

    webrtc_server.run().await.map_err(anyhow::Error::new)
}
