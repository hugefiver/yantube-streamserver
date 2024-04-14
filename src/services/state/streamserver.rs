use crate::pb::streamserver::stream_server_client::StreamServerClient;
use crate::pb::streamserver::StreamServerRegisterRequest;
use tracing::{debug, info, span, trace, warn, Level};

pub async fn register_to_apiserver(host: &str) -> anyhow::Result<()> {
    info!("register to apiserver");
    let mut client = StreamServerClient::connect("http://[::1]:9082").await?;
    let request = tonic::Request::new(StreamServerRegisterRequest {
        host: host.to_string(),
    });

    let response = client.register(request).await?;
    info!("register success: {:?}", response.get_ref().success);

    Ok(())
}


pub async fn unregister_to_apiserver(host: &str) -> anyhow::Result<()> {
    info!("unregister to apiserver");
    let mut client = StreamServerClient::connect("http://[::1]:9082").await?;
    let request = tonic::Request::new(StreamServerRegisterRequest {
        host: host.to_string(),
    });

    let response = client.unregister(request).await?;
    info!("unregister success: {:?}", response.get_ref().success);

    Ok(())
}


