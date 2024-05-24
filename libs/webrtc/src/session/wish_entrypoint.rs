use std::{collections::HashMap, sync::Arc};

use auth::Auth;
use axum::{
    body,
    extract::{self},
    middleware,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use http::StatusCode;
use streamhub::{
    define::StreamHubEventSender,
    utils::{RandomDigitCount, Uuid},
};
use tokio::net::ToSocketAddrs;
use tokio::sync::RwLock;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use super::session::{WebRTCServerSession, WebrtcSessionMapping};

#[derive(Clone)]
pub struct WishEntrypointServer<Addr: ToSocketAddrs, A: Auth + 'static> {
    pub addr: Addr,

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

impl<Addr: ToSocketAddrs, A: Auth> WishEntrypointServer<Addr, A> {
    pub fn new(addr: Addr, event_producer: StreamHubEventSender, auth: Option<A>) -> Self {
        Self {
            addr,
            auth,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            event_producer,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let state = State {
            auth: self.auth.clone(),
            sessions: self.sessions.clone(),
            event_producer: self.event_producer.clone(),
        };

        let whip_router = Router::new()
            .route(
                "/whip",
                post(post_whip_handler)
                    .route_layer(middleware::from_fn_with_state(
                        state.clone(),
                        whip_auth_middleware,
                    ))
                    .delete(delete_whip_handler),
            )
            .with_state(state.clone());

        let whep_router = Router::new()
            .route(
                "/whep",
                post(post_whep_handler)
                    // .delete(delete_whep_handler)
                    .route_layer(middleware::from_fn_with_state(
                        state.clone(),
                        whep_auth_middleware,
                    ))
                    .options(option_cors_all_allow),
            )
            .route_layer(middleware::from_fn(cors_middleware))
            .with_state(state.clone());

        let router = axum::Router::new()
            .merge(whip_router)
            .merge(whep_router)
            .fallback(|| async { StatusCode::NOT_FOUND })
            .with_state(state)
            .layer(tower_http::trace::TraceLayer::new_for_http().make_span_with(|req: &extract::Request| -> tracing::Span {
                tracing::span!(tracing::Level::INFO, "wish_entrypoint", method=%req.method(), uri=%req.uri())
            })
            .on_response(
                tower_http::trace::DefaultOnResponse::new().level(tracing::Level::DEBUG)
             ));

        let listenser = tokio::net::TcpListener::bind(&self.addr).await?;
        axum::serve(listenser, router).await?;
        Ok(())
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EntrypointParrams {
    pub app: Option<String>,
    pub stream: Option<String>,
    pub token: Option<String>,
    pub session_id: Option<String>,
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

async fn cors_middleware(req: extract::Request, next: middleware::Next) -> Response {
    let mut resp = next.run(req).await;
    resp.headers_mut().insert(
        http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        http::HeaderValue::from_static("*"),
    );
    resp
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

async fn whep_auth_middleware<A: Auth>(
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
        if let Err(err) = auth.auth_pull(app, stream, query) {
            log::error!(
                "whep auth error: app={} stream={}: {}",
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

    let session_id = Uuid::new(RandomDigitCount::default());
    let path = format!(
        "{}?app={}&stream={}&session_id={}",
        uri.path(),
        app,
        stream,
        session_id,
    );
    let mut session =
        WebRTCServerSession::new_with_id(app, stream, state.event_producer, session_id);

    match session.publish_whip(path, offer).await {
        Ok(resp) => {
            let mut guard = state.sessions.write().await;
            guard.insert(session.session_id, Arc::new(RwLock::new(session)));
            resp
        }
        Err(err) => {
            log::error!("handle whip post error, {}", err);
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

async fn delete_whip_handler<A: Auth>(
    extract::State(state): extract::State<State<A>>,
    extract::Query(par): extract::Query<EntrypointParrams>,
) -> Response {
    let EntrypointParrams { session_id, .. } = par;

    let session_id = if let Some(session_id) = session_id {
        if let Some(uuid) = Uuid::from_str2(&session_id) {
            uuid
        } else {
            return (StatusCode::BAD_REQUEST, "session_id is not valid").into_response();
        }
    } else {
        return (StatusCode::BAD_REQUEST, "session_id cannot be empty").into_response();
    };

    let guard = state.sessions.read().await;
    if !guard.contains_key(&session_id) {
        return (StatusCode::OK, "session not found").into_response();
    }
    drop(guard);

    let mut guard = state.sessions.write().await;
    let session = if let Some(session) = guard.remove(&session_id) {
        session.clone()
    } else {
        return (StatusCode::OK, "session not found").into_response();
    };

    let guard2 = session.read().await;
    if let Err(err) = guard2.unpublish_whip() {
        log::error!("unpublish whip error: {}", err);
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    ().into_response()
}

async fn post_whep_handler<A: Auth>(
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

    let session_id = Uuid::new(RandomDigitCount::default());
    let path = format!(
        "{}?app={}&stream={}&session_id={}",
        uri.path(),
        app,
        stream,
        session_id,
    );
    let mut session =
        WebRTCServerSession::new_with_id(app, stream, state.event_producer, session_id);

    match session.subscribe_whep(path, offer).await {
        Ok(resp) => {
            let mut guard = state.sessions.write().await;
            guard.insert(session.session_id, Arc::new(RwLock::new(session)));
            resp
        }
        Err(err) => {
            log::error!("handle whip post error, {}", err);
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}
