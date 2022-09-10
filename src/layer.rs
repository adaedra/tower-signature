use crate::{key::Key, service::SignatureValidation};
use axum::response::IntoResponse;
use hyper::{
    body::{Body, Bytes, HttpBody},
    Request,
};
use tower::{Layer, Service};

pub struct SignatureValidationLayer {
    key: Key,
}

impl SignatureValidationLayer {
    pub fn new(data: &[u8]) -> Result<SignatureValidationLayer, hex::FromHexError> {
        Key::from(data).map(|key| SignatureValidationLayer { key })
    }
}

impl<Inner, ResBody> Layer<Inner> for SignatureValidationLayer
where
    Inner: Service<Request<Body>, Response = ResBody> + Send + Sync + Clone + 'static,
    Inner::Error: IntoResponse + Send,
    Inner::Future: Send,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
{
    type Service = SignatureValidation<Inner, ResBody>;

    fn layer(&self, inner: Inner) -> Self::Service {
        SignatureValidation {
            key: self.key.clone(),
            inner,
        }
    }
}
