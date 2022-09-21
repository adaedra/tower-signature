use hyper::http::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while reading body: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("Error while reading body: {0}")]
    Io(#[from] std::io::Error),
    #[error("Missing signature headers")]
    MissingSignature,
    #[error("Invalid signature")]
    InvalidSignature,
}

impl From<&Error> for StatusCode {
    fn from(e: &Error) -> Self {
        match e {
            Error::MissingSignature => StatusCode::BAD_REQUEST,
            Error::InvalidSignature => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(_: ring::error::Unspecified) -> Self {
        Error::InvalidSignature
    }
}
