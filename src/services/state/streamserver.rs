use crate::{config::server::ServerConf, pb::streamserver::stream_server_client::StreamServerClient};
use crate::pb::streamserver::StreamServerRegisterRequest;
use tracing::{info};

pub async fn register_to_apiserver(cfg: &ServerConf) -> anyhow::Result<()> {
    info!("register to apiserver");
    let mut client = StreamServerClient::connect(cfg.api_addr.clone()).await?;
    let request = tonic::Request::new(StreamServerRegisterRequest {
        host: cfg.get_self_addr(),
        secret: cfg.api_secret.clone(),
    });

    let response = client.register(request).await?;
    info!("register success: {:?}", response.get_ref().success);

    Ok(())
}


pub async fn unregister_to_apiserver(cfg: &ServerConf) -> anyhow::Result<()> {
    info!("unregister to apiserver");
    let mut client = StreamServerClient::connect(cfg.api_addr.clone()).await?;
    let request = tonic::Request::new(StreamServerRegisterRequest {
        host: cfg.get_self_addr(),
        secret: cfg.api_secret.clone(),
    });

    let response = client.unregister(request).await?;
    info!("unregister success: {:?}", response.get_ref().success);

    Ok(())
}


