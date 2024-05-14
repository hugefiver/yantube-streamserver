use streamhub::errors::StreamHubError;
use thiserror::Error;
use {
    auth::AuthError,
    bytesio::bytes_errors::BytesReadError,
    bytesio::{bytes_errors::BytesWriteError, bytesio_errors::BytesIOError},
    std::fmt,
    std::str::Utf8Error,
    tokio::sync::oneshot::error::RecvError,
    webrtc::error::Error as RTCError,
};

#[derive(Debug, Error)]
pub struct SessionError {
    pub value: SessionErrorValue,
}

#[derive(Debug, Error)]
pub enum SessionErrorValue {
    #[error("net io error: {0}")]
    BytesIOError(BytesIOError),
    #[error("bytes read error: {0}")]
    BytesReadError(BytesReadError),
    #[error("bytes write error: {0}")]
    BytesWriteError(BytesWriteError),
    #[error("Utf8Error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("event execute error: {0}")]
    ChannelError(StreamHubError),
    #[error("webrtc error: {0}")]
    RTCError(#[from] RTCError),
    #[error("tokio: oneshot receiver err: {0}")]
    RecvError(#[from] RecvError),
    #[error("Auth err: {0}")]
    AuthError(#[from] AuthError),
    #[error("stream hub event send error")]
    StreamHubEventSendErr,
    #[error("cannot receive frame data from stream hub")]
    CannotReceiveFrameData,
    #[error("Http Request path error")]
    HttpRequestPathError,
    #[error("Not supported")]
    HttpRequestNotSupported,
    #[error("Empty sdp data")]
    HttpRequestEmptySdp,
    #[error("Cannot find Content-Length")]
    HttpRequestNoContentLength,
    #[error("Channel receive error")]
    ChannelRecvError,
    #[error("Http error: {0}")]
    HttpError(#[from] http::Error),
}

impl From<BytesIOError> for SessionErrorValue {
    fn from(value: BytesIOError) -> Self {
        Self::BytesIOError(value)
    }
}

impl From<BytesReadError> for SessionErrorValue {
    fn from(value: BytesReadError) -> Self {
        SessionErrorValue::BytesReadError(value)
    }
}

impl From<BytesWriteError> for SessionErrorValue {
    fn from(value: BytesWriteError) -> Self {
        SessionErrorValue::BytesWriteError(value)
    }
}

impl From<StreamHubError> for SessionErrorValue {
    fn from(value: StreamHubError) -> Self {
        SessionErrorValue::ChannelError(value)
    }
}

// impl From<RTCError> for SessionError {
//     fn from(error: RTCError) -> Self {
//         SessionError {
//             value: SessionErrorValue::RTCError(error),
//         }
//     }
// }

// impl From<BytesIOError> for SessionError {
//     fn from(error: BytesIOError) -> Self {
//         SessionError {
//             value: SessionErrorValue::BytesIOError(error),
//         }
//     }
// }

// impl From<BytesReadError> for SessionError {
//     fn from(error: BytesReadError) -> Self {
//         SessionError {
//             value: SessionErrorValue::BytesReadError(error),
//         }
//     }
// }

// impl From<BytesWriteError> for SessionError {
//     fn from(error: BytesWriteError) -> Self {
//         SessionError {
//             value: SessionErrorValue::BytesWriteError(error),
//         }
//     }
// }

// impl From<Utf8Error> for SessionError {
//     fn from(error: Utf8Error) -> Self {
//         SessionError {
//             value: SessionErrorValue::Utf8Error(error),
//         }
//     }
// }

// impl From<StreamHubError> for SessionError {
//     fn from(error: StreamHubError) -> Self {
//         SessionError {
//             value: SessionErrorValue::ChannelError(error),
//         }
//     }
// }

// impl From<RecvError> for SessionError {
//     fn from(error: RecvError) -> Self {
//         SessionError {
//             value: SessionErrorValue::RecvError(error),
//         }
//     }
// }

// impl From<AuthError> for SessionError {
//     fn from(error: AuthError) -> Self {
//         SessionError {
//             value: SessionErrorValue::AuthError(error),
//         }
//     }
// }

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.value, f)
    }
}

impl<E: Into<SessionErrorValue>> From<E> for SessionError {
    fn from(value: E) -> Self {
        SessionError {
            value: value.into(),
        }
    }
}
