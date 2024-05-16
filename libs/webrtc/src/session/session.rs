use std::{collections::HashMap, sync::Arc};

use crate::whip::handle_whip;

use super::{
    errors::{SessionError, SessionErrorValue},
    WebRTCStreamHandler,
};
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
    utils::{RandomDigitCount, Uuid},
};
use tokio::sync::{oneshot, RwLock};
use webrtc::peer_connection::{sdp::session_description::RTCSessionDescription, RTCPeerConnection};

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
        Self::new_with_id(
            app_name,
            stream_name,
            event_sender,
            Uuid::new(RandomDigitCount::default()),
        )
    }

    pub async fn publish_whip(
        &mut self, path: String,
        offer: RTCSessionDescription,
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

        let sender = event_result_receiver.await??;

        match handle_whip(offer, sender.0, sender.1).await {
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

    pub fn unpublish_whip(
        &mut self,
    ) -> Result<(), SessionError> {
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
