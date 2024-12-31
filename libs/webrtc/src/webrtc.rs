use streamhub::define::StreamHubEventSender;

use auth::Auth;
use webrtc::ice_transport::ice_server::RTCIceServer;

use std::net::SocketAddr;

use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref ICE_SERVERS: Vec<RTCIceServer> = vec![
    // RTCIceServer {
    //     urls: vec![
    //         "stun:10.15.0.65:3478".to_string(),
    //         "turn:10.15.0.65:3478?transport=udp".to_string(),
    //         "turn:10.15.0.65:3478?transport=tcp".to_string(),
    //     ],
    //     username: "public".to_string(),
    //     credential: "123456".to_string(),
    // },
    RTCIceServer {
        urls: vec![
            "stun:stun.l.google.com:19302".to_string(),
            // "stun:stun.qq.com:3478".to_string(),
            "stun:stun.syncthing.net:3478".to_string(),
        ],
        ..Default::default()
    },
];}

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
