use std::{collections::HashMap, sync::Arc};

use crate::{whep::handle_whep, whip::handle_whip};

use super::{
    errors::{SessionError, SessionErrorValue},
    WebRTCStreamHandler,
};
use anyhow::anyhow;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use http::{header, StatusCode};
use streamhub::{
    define::{
        NotifyInfo, PublishType, PublisherInfo, StreamHubEvent, StreamHubEventSender,
        SubscribeType, SubscriberInfo,
    },
    stream::StreamIdentifier,
    utils::Uuid,
};
use tokio::sync::{broadcast, oneshot, RwLock};
use webrtc::peer_connection::{
    peer_connection_state::RTCPeerConnectionState, sdp::session_description::RTCSessionDescription,
    RTCPeerConnection,
};

pub type WebrtcSessionMapping = HashMap<Uuid, Arc<RwLock<WebRTCServerSession>>>;

#[derive(Clone)]
pub struct WebRTCServerSession {
    event_sender: StreamHubEventSender,
    stream_handler: Arc<WebRTCStreamHandler>,

    pub app_name: String,
    pub stream_name: String,

    pub session_id: Uuid,
    pub peer_connection: Option<Arc<RTCPeerConnection>>,
}

impl WebRTCServerSession {
    pub fn new_with_id(
        app_name: String, stream_name: String, event_sender: StreamHubEventSender, session_id: Uuid,
    ) -> Self {
        Self {
            event_sender,
            stream_handler: Arc::new(WebRTCStreamHandler::default()),
            app_name,
            stream_name,
            session_id,
            peer_connection: None,
        }
    }

    pub fn new(app_name: String, stream_name: String, event_sender: StreamHubEventSender) -> Self {
        Self::new_with_id(app_name, stream_name, event_sender, Uuid::new())
    }

    pub async fn publish_whip(
        &mut self, path: String, offer: RTCSessionDescription,
    ) -> Result<Response, SessionError> {
        let (event_result_sender, event_result_receiver) = oneshot::channel();

        let publish_event = StreamHubEvent::Publish {
            identifier: StreamIdentifier::WebRTC {
                app_name: self.app_name.clone(),
                stream_name: self.stream_name.clone(),
            },
            result_sender: event_result_sender,
            info: self.get_publisher_info(),
            stream_handler: self.stream_handler.clone(),
        };

        if self.event_sender.send(publish_event).is_err() {
            return Err(SessionError {
                value: SessionErrorValue::StreamHubEventSendErr,
            });
        }

        let (frame_sender, packet_sender) = match event_result_receiver.await?? {
            (Some(a), Some(b), _c) => (a, b),
            _ => return Ok(StatusCode::SERVICE_UNAVAILABLE.into_response()),
        };

        match handle_whip(offer, frame_sender, packet_sender).await {
            Ok((session_description, peer_connection)) => {
                self.peer_connection = Some(peer_connection);

                let response = Response::builder()
                    .status(StatusCode::CREATED)
                    .header(header::CONTENT_TYPE, "application/sdp")
                    .header(header::LOCATION, path)
                    .body(Body::from(session_description.sdp))?;
                Ok(response)
            }
            Err(err) => {
                log::error!("handle whip err: {}", err);

                Ok(StatusCode::SERVICE_UNAVAILABLE.into_response())
            }
        }
    }

    pub fn unpublish_whip(&self) -> Result<(), SessionError> {
        let unpublish_event = StreamHubEvent::UnPublish {
            identifier: StreamIdentifier::WebRTC {
                app_name: self.app_name.clone(),
                stream_name: self.stream_name.clone(),
            },
            info: self.get_publisher_info(),
        };

        if self.event_sender.send(unpublish_event).is_err() {
            return Err(SessionError {
                value: SessionErrorValue::StreamHubEventSendErr,
            });
        }

        Ok(())
    }

    pub async fn subscribe_whep(
        &mut self, path: String, offer: RTCSessionDescription,
    ) -> Result<Response, SessionError> {
        let subscriber_info = self.get_subscriber_info();

        let (event_result_sender, event_result_receiver) = oneshot::channel();

        let subscribe_event = StreamHubEvent::Subscribe {
            identifier: StreamIdentifier::WebRTC {
                app_name: self.app_name.clone(),
                stream_name: self.stream_name.clone(),
            },
            info: subscriber_info.clone(),
            result_sender: event_result_sender,
        };

        if self.event_sender.send(subscribe_event).is_err() {
            return Err(SessionError {
                value: SessionErrorValue::StreamHubEventSendErr,
            });
        }

        let receiver = event_result_receiver.await??.0.packet_receiver.unwrap();

        let (pc_state_sender, mut pc_state_receiver) = broadcast::channel(1);

        let response = match handle_whep(offer, receiver, pc_state_sender).await {
            Ok((session_description, peer_connection)) => {
                let pc_clone = peer_connection.clone();

                let app_name_out = self.app_name.clone();
                let stream_name_out = self.stream_name.clone();
                let subscriber_info_out = subscriber_info.clone();
                let sender_out = self.event_sender.clone();

                tokio::spawn(async move {
                    loop {
                        if let Ok(state) = pc_state_receiver.recv().await {
                            log::info!("state: {}", state);
                            match state {
                                RTCPeerConnectionState::Disconnected
                                | RTCPeerConnectionState::Failed => {
                                    if let Err(err) = pc_clone.close().await {
                                        log::error!("peer connection close error: {}", err);
                                    }
                                }
                                RTCPeerConnectionState::Closed => {
                                    if let Err(err) = Self::unsubscribe_whep(
                                        app_name_out,
                                        stream_name_out,
                                        subscriber_info_out,
                                        sender_out,
                                    ) {
                                        log::error!("unsubscribe whep error: {}", err);
                                    }
                                    break;
                                }
                                _ => {}
                            }
                        } else {
                            log::info!("recv");
                        }
                    }
                });

                self.peer_connection = Some(peer_connection);

                Response::builder()
                    .header(header::CONTENT_TYPE, "application/sdp")
                    .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, "Location")
                    .header(header::LOCATION, path)
                    .body(Body::from(session_description.sdp))?
            }
            Err(err) => {
                log::error!("handle whep err: {}", err);
                StatusCode::SERVICE_UNAVAILABLE.into_response()
            }
        };
        Ok(response)
    }

    fn unsubscribe_whep(
        app_name: String, stream_name: String, subscriber_info: SubscriberInfo,
        sender: StreamHubEventSender,
    ) -> Result<(), SessionError> {
        let unsubscribe_event = StreamHubEvent::UnSubscribe {
            identifier: StreamIdentifier::WebRTC {
                app_name,
                stream_name,
            },
            info: subscriber_info,
        };

        if sender.send(unsubscribe_event).is_err() {
            return Err(SessionError {
                value: SessionErrorValue::StreamHubEventSendErr,
            });
        }
        Ok(())
    }

    pub async fn patch_whep(&self, offer: RTCSessionDescription) -> anyhow::Result<()> {
        let Some(pc) = &self.peer_connection else {
            return Err(anyhow!("peer connection not found"));
        };

        pc.set_remote_description(offer).await?;

        Ok(())
    }

    fn get_subscriber_info(&self) -> SubscriberInfo {
        let id = self.session_id;

        SubscriberInfo {
            id,
            sub_type: SubscribeType::PlayerWebrtc,
            sub_data_type: streamhub::define::SubDataType::Packet,
            notify_info: NotifyInfo {
                request_url: String::from(""),
                remote_addr: String::from(""),
            },
        }
    }

    fn get_publisher_info(&self) -> PublisherInfo {
        let id = self.session_id;

        PublisherInfo {
            id,
            pub_type: PublishType::PushWebRTC,
            pub_data_type: streamhub::define::PubDataType::Both,
            notify_info: NotifyInfo {
                request_url: String::from(""),
                remote_addr: String::from(""),
            },
        }
    }
}
