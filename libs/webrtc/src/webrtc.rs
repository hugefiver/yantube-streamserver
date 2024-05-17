use streamhub::define::StreamHubEventSender;

use auth::Auth;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::session::WebrtcSessionMapping;

pub struct WebRTCServer<A: Auth + Clone + 'static> {
    address: String,
    event_producer: StreamHubEventSender,
    auth: Option<A>,
}

impl<A: Auth + Clone + 'static> WebRTCServer<A> {
    pub fn new(address: String, event_producer: StreamHubEventSender, auth: Option<A>) -> Self {
        Self {
            address,
            event_producer,
            auth,
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let socket_addr: SocketAddr = self.address.parse().unwrap();

        log::info!("WebRTC server listening on http://{}/", socket_addr);
        crate::session::WishEntrypointServer::new(
            self.address.clone(),
            self.event_producer.clone(),
            self.auth.clone(),
        )
        .run()
        .await?;
        Ok(())
    }
}
