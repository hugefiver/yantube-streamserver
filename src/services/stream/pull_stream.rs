use rsmpeg::avformat::AVIOContext;
use tokio::select;
use tracing::{debug, error, info};

pub async fn rtmp_server(
    conf: &crate::config::AppConfig,
) -> anyhow::Result<Box<dyn std::error::Error>> {
    let listen_port = conf.stream.port;
    let listen_host = conf.stream.host.clone();

    let mut listener = tokio::net::TcpListener::bind((listen_host, listen_port)).await?;
    loop {
        select! {
            tcp_stream = listener.accept() => {
                if let Ok((stream, addr)) = tcp_stream {
                    debug!("new connection from {:?}", addr);

                } else {

                    error!("failed to accept connection");
                }

            }
        }
    }
}
