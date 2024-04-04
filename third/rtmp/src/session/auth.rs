

pub trait Auth {
    fn auth(&self, app: Option<&str>, stream: Option<&str>, query: Option<&str>) -> Result<(), AuthError>;

    fn auth_pull(&self, app: Option<&str>, stream: Option<&str>, query: Option<&str>) -> Result<(), AuthError> {
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

/* impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.value, f)
    }
} */

/* impl Fail for AuthError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.value.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.value.backtrace()
    }
} */

