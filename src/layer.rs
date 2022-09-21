use crate::{key::Key, service::SignatureValidation};
use hyper::{
    body::{Body, Bytes, HttpBody},
    Request,
};
use std::error::Error as StdError;
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
    Inner::Error: Into<Box<dyn StdError + Sync + Send + 'static>>,
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
