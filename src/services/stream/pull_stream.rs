
use streamhub::define::StreamHubEventSender;
use streamhub::StreamsHub;
use tracing::{error, info};
use auth::auth::SimpleTokenAuthenticator;

#[derive(Debug)]
struct RtmpSessionContext {
    pub stream: tokio::net::TcpStream,
    pub sender: StreamHubEventSender,
}

pub async fn rtmp_server(conf: &crate::config::AppConfig) -> anyhow::Result<()> {
    let listen_port = conf.stream.port;
    let listen_host = conf.stream.host.clone();

    /* let mut rtmp_server = RtmpServer::new(
        listen_host + ":" + &listen_port.to_string(),
        sender,
        2,
    ); */

    start_rtmp_server_loop(listen_host, listen_port).await
}

async fn start_rtmp_server_loop(host: String, port: u16) -> anyhow::Result<()>
{
    let mut stream_hub = StreamsHub::new(None);
    let sender = stream_hub.get_hub_event_sender();
    tokio::spawn(async move { stream_hub.run().await });

    let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    info!("rtmp server listening on rtmp://{}:{}", &host, port);

    loop {
        match listener.accept().await {
            Ok((stream, client_addr)) => {
                info!("new client connected: {}", client_addr);

                let ctx = RtmpSessionContext {
                    stream,
                    sender: sender.clone(),
                };
                tokio::spawn(async move {
                    handle_rtmp_session(ctx).await
                });
            }
            Err(err) => {
                error!("rtmp server accept error: {}", err);
            }
        }
    }

}

async fn handle_rtmp_session(ctx: RtmpSessionContext) -> anyhow::Result<()> {
    let stream = ctx.stream;
    let sender = ctx.sender;

    let authenticator = SimpleTokenAuthenticator::new("123456".to_string());
    let mut rtmp_session = rtmp::session::server_session::ServerSession::new(stream, sender, 2, Some(authenticator));

    rtmp_session.run().await.map_err(anyhow::Error::new)
}
