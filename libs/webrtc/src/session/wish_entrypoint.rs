use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    sync::Arc,
};

use auth::Auth;
use axum::{
    body,
    extract::{self, FromRequest, Query, Request},
    middleware,
    response::{IntoResponse, Response},
    routing::{any, post},
    Json, RequestExt, Router,
};
use http::{StatusCode, Uri};
use streamhub::define::StreamHubEventSender;
use tokio::sync::RwLock;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use super::session::{WebRTCServerSession, WebrtcSessionMapping};

#[derive(Clone)]
struct WishEntrypointServer<A: Auth + 'static> {
    pub host: String,
    pub port: u16,

    pub auth: Option<A>,

    pub sessions: Arc<RwLock<WebrtcSessionMapping>>,
    pub event_producer: StreamHubEventSender,
}

#[derive(Clone)]
struct State<A: Auth> {
    pub auth: Option<A>,
    pub sessions: Arc<RwLock<WebrtcSessionMapping>>,
    pub event_producer: StreamHubEventSender,
}

impl<A: Auth> WishEntrypointServer<A> {
    pub fn new(
        host: String, port: u16, event_producer: StreamHubEventSender, auth: Option<A>,
    ) -> Self {
        Self {
            host,
            port,
            auth,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            event_producer,
        }
    }
}

impl<A: Auth> WishEntrypointServer<A> {
    pub async fn run(&self) -> anyhow::Result<()> {
        let state = State {
            auth: self.auth.clone(),
            sessions: self.sessions.clone(),
            event_producer: self.event_producer.clone(),
        };

        let whip_router = Router::new()
            .route(
                "/whip",
                post(post_whip_handler).route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    whip_auth_middleware,
                )),
            )
            .with_state(state.clone());

        // TODO: implement WHEP entrypoint

        let router = axum::Router::new()
            .merge(whip_router)
            .fallback(|| async { StatusCode::NOT_FOUND })
            .with_state(state);

        let listenser = tokio::net::TcpListener::bind((self.host.as_str(), self.port)).await?;
        axum::serve(listenser, router).await?;
        Ok(())
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EntrypointParrams {
    pub app: Option<String>,
    pub stream: Option<String>,
    pub token: Option<String>,
}

#[axum::debug_handler]
async fn option_cors_all_allow() -> Result<Response, StatusCode> {
    let mut resp = Response::builder();
    let headers = resp
        .headers_mut()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let extra_headers = vec![
        (http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
        (http::header::ACCESS_CONTROL_ALLOW_METHODS, "*"),
        (http::header::ACCESS_CONTROL_ALLOW_HEADERS, "*"),
    ]
    .into_iter()
    .filter_map(|(k, v)| {
        let v = v.parse();
        match v {
            Ok(v) => Some((k, v)),
            Err(_) => None,
        }
    });
    headers.extend(extra_headers);

    resp.body(body::Body::empty())
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR))
}

async fn whip_auth_middleware<A: Auth>(
    extract::State(state): extract::State<State<A>>,
    extract::Query(par): extract::Query<EntrypointParrams>, req: extract::Request,
    next: middleware::Next,
) -> Response {
    let app = par.app.as_deref();
    let stream = par.stream.as_deref();

    if matches!(app, Some("") | None) || matches!(stream, Some("") | None) {
        return (StatusCode::BAD_REQUEST, "app or stream cannot be empty").into_response();
    }

    if let Some(auth) = state.auth {
        let query = req.uri().query();
        if let Err(err) = auth.auth(app, stream, query) {
            log::error!(
                "whip auth error: app={} stream={}: {}",
                app.unwrap_or(""),
                stream.unwrap_or(""),
                err
            );
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };
    next.run(req).await
}

async fn post_whip_handler<A: Auth>(
    extract::State(state): extract::State<State<A>>,
    extract::Query(par): extract::Query<EntrypointParrams>, uri: extract::OriginalUri,
    sdp_data: String,
) -> Response {
    let EntrypointParrams { app, stream, .. } = par;
    let app = app.unwrap_or_default();
    let stream = stream.unwrap_or_default();

    if sdp_data.is_empty() {
        return (StatusCode::BAD_REQUEST, "sdp data is empty").into_response();
    }

    let offer = match RTCSessionDescription::offer(sdp_data) {
        Err(err) => {
            log::error!("whip sdp offer error: {}", err);
            return StatusCode::BAD_REQUEST.into_response();
        }
        Ok(offer) => offer,
    };

    let mut session = WebRTCServerSession::new(state.event_producer);

    let path = format!(
        "{}?app={}&stream={}&session_id={}",
        uri.path(),
        app,
        stream,
        session.session_id,
    );
    match session.publish_whip(app, stream, path, offer).await {
        Ok(resp) => {
            let mut guard = state.sessions.write().await;
            guard.insert(session.session_id, Arc::new(RwLock::new(session)));
            resp
        }
        Err(err) => {
            log::error!("handle whip post error, {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
