use axum::response::IntoResponse;
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

#[derive(Debug)]
pub enum Wrapper<E>
where
    E: IntoResponse + Send + 'static,
{
    Own(Error),
    Inner(E),
}

impl<E> From<E> for Wrapper<E>
where
    E: IntoResponse + Send + 'static,
{
    fn from(e: E) -> Self {
        Wrapper::Inner(e)
    }
}

impl<E> From<Error> for Wrapper<E>
where
    E: IntoResponse + Send + 'static,
{
    fn from(e: Error) -> Self {
        Wrapper::Own(e)
    }
}

impl<E> IntoResponse for Wrapper<E>
where
    E: IntoResponse + Send + 'static,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Wrapper::Own(err) => {
                let status = match err {
                    Error::MissingSignature => StatusCode::BAD_REQUEST,
                    Error::InvalidSignature => StatusCode::UNAUTHORIZED,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };

                (status, format!("{}", err)).into_response()
            }
            Wrapper::Inner(err) => err.into_response(),
        }
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(_: ring::error::Unspecified) -> Self {
        Error::InvalidSignature
    }
}
