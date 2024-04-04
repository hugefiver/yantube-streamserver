use rtmp::{
    relay::{pull_client::PullClient, push_client::PushClient},
    rtmp::RtmpServer,
};
use streamhub::StreamsHub;
use tracing::{debug, error, info};

pub async fn rtmp_server(
    conf: &crate::config::AppConfig,
) -> anyhow::Result<(), Box<dyn std::error::Error>> {
    let listen_port = conf.stream.port;
    let listen_host = conf.stream.host.clone();

    let mut stream_hub = StreamsHub::new(None);
    let sender = stream_hub.get_hub_event_sender();
    tokio::spawn(async move { stream_hub.run().await });

    let mut rtmp_server = RtmpServer::new(
        listen_host + ":" + &listen_port.to_string(),
        sender,
        1,
        None,
    );
    tokio::spawn(async move {
        if let Err(err) = rtmp_server.run().await {
            error!("rtmp server run error: {}", err);
        }
    });

    loop {}
}
