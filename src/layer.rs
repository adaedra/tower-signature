use crate::{key::Key, service::SignatureValidator};
use axum::response::IntoResponse;
use hyper::{
    body::{Body, Bytes, HttpBody},
    Request,
};
use tower::{Layer, Service};

pub struct SignatureValidatorLayer {
    key: Key,
}

impl SignatureValidatorLayer {
    pub fn new(data: &[u8]) -> Result<SignatureValidatorLayer, hex::FromHexError> {
        Key::from(data).map(|key| SignatureValidatorLayer { key })
    }
}

impl<Inner, ResBody> Layer<Inner> for SignatureValidatorLayer
where
    Inner: Service<Request<Body>, Response = ResBody> + Send + Sync + Clone + 'static,
    Inner::Error: IntoResponse + Send,
    Inner::Future: Send,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
{
    type Service = SignatureValidator<Inner, ResBody>;

    fn layer(&self, inner: Inner) -> Self::Service {
        SignatureValidator {
            key: self.key.clone(),
            inner,
        }
    }
}
