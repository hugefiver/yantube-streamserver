use std::collections::HashMap;

pub trait Auth: Send + Sync + Clone {
    fn auth(
        &self, app: Option<&str>, stream: Option<&str>, query: Option<&str>,
    ) -> Result<(), AuthError>;

    fn auth_pull(
        &self, app: Option<&str>, stream: Option<&str>, query: Option<&str>,
    ) -> Result<(), AuthError> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("token is not correct.")]
    TokenIsNotCorrect,
    #[error("no token found.")]
    NoTokenFound,
}

#[derive(Debug, Clone)]
pub struct SimpleTokenAuthenticator {
    token: Option<String>,
}

impl SimpleTokenAuthenticator {
    pub fn new(token: String) -> Self {
        Self { token: Some(token) }
    }

    pub fn new_nonauth() -> Self {
        Self { token: None }
    }
}

fn extract_query(q: &str) -> HashMap<String, String> {
    url::form_urlencoded::parse(q.as_bytes())
        .into_owned()
        .collect()
}

impl Auth for SimpleTokenAuthenticator {
    fn auth(
        &self, app: Option<&str>, stream: Option<&str>, query: Option<&str>,
    ) -> Result<(), AuthError> {
        let span = tracing::span!(tracing::Level::DEBUG, "simple_auth");
        let _ = span.enter();
        tracing::debug!(app, stream, query, "simple auth for session");

        if let Some(auth_token) = &self.token {
            match query {
                Some("") | None => Err(AuthError::NoTokenFound),
                Some(query) => {
                    let query = extract_query(query);
                    tracing::debug!(?query, "session query");
                    match query.get("token") {
                        Some(token) if token == auth_token => Ok(()),
                        None => Err(AuthError::NoTokenFound),
                        _ => Err(AuthError::TokenIsNotCorrect),
                    }
                }
            }
        } else {
            Ok(())
        }
    }
}

impl Auth for () {
    fn auth(
        &self, _app: Option<&str>, _stream: Option<&str>, _query: Option<&str>,
    ) -> Result<(), AuthError> {
        Ok(())
    }
}
