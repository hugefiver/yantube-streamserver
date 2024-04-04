use std::collections::HashMap;

use rtmp::session::auth;

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

fn extract_query(mut q: &str) -> HashMap<String, String> {
    // if q.starts_with('?') {
    //     q = &q[1..];
    // }

    url::form_urlencoded::parse(q.as_bytes())
        .into_owned()
        .collect()
}

impl auth::Auth for SimpleTokenAuthenticator {
    fn auth(
        &self, app: Option<&str>, stream: Option<&str>, query: Option<&str>,
    ) -> Result<(), auth::AuthError> {
        let span = tracing::span!(tracing::Level::DEBUG, "rtmp_auth");
        let _ = span.enter();
        tracing::debug!(app, stream, query, "simple auth for rtmp session");

        if let Some(auth_token) = &self.token {
            match query {
                Some("") | None=> Err(auth::AuthError::NoTokenFound),
                Some(query) => {
                    let query = extract_query(query);
                    tracing::debug!(?query, "rtmp session query");
                    match query.get("token") {
                        Some(token) if token == auth_token => Ok(()),
                        None => Err(auth::AuthError::NoTokenFound),
                        _ => Err(auth::AuthError::TokenIsNotCorrect),
                    }
                },
            }
        } else {
            Ok(())
        }
    }
}
