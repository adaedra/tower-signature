use crate::{
    error::{Error, Wrapper},
    key::Key,
};
use axum::response::IntoResponse;
use futures::{future::ready, TryFutureExt, TryStreamExt};
use hyper::{
    body::{Body, Bytes, HttpBody},
    Request,
};
use ring::signature::{VerificationAlgorithm, ED25519};
use std::{future::Future, io::Write};
use tower::Service;

pub struct SignatureValidation<Inner, ResBody>
where
    Inner: Service<Request<Body>, Response = ResBody> + Send + Sync + Clone + 'static,
    Inner::Error: IntoResponse + Send,
    Inner::Future: Send,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
{
    pub key: Key,
    pub inner: Inner,
}

impl<Inner, ResBody> Clone for SignatureValidation<Inner, ResBody>
where
    Inner: Service<Request<Body>, Response = ResBody> + Send + Sync + Clone + 'static,
    Inner::Error: IntoResponse + Send,
    Inner::Future: Send,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
{
    fn clone(&self) -> Self {
        SignatureValidation {
            key: self.key.clone(),
            inner: self.inner.clone(),
        }
    }
}

async fn checked(key: Key, req: Request<Body>) -> Result<Request<Body>, Error> {
    let (req, stream) = req.into_parts();

    let timestamp = req
        .headers
        .get("X-Signature-Timestamp")
        .ok_or(Error::MissingSignature)?;
    let signature = req
        .headers
        .get("X-Signature-Ed25519")
        .ok_or(Error::MissingSignature)?;

    let mut body = Vec::new();
    body.write_all(timestamp.as_bytes()).unwrap();

    TryStreamExt::map_err(stream, Error::from)
        .try_for_each(|chunk| ready(body.write_all(&chunk).map_err(Error::from)))
        .await?;

    let signature = hex::decode(signature).unwrap();

    ED25519.verify(
        key.as_slice().into(),
        body.as_slice().into(),
        signature.as_slice().into(),
    )?;

    let actual_body = &body[timestamp.len()..];

    Ok(Request::from_parts(
        req,
        Bytes::copy_from_slice(actual_body).into(),
    ))
}

impl<Inner, ResBody> Service<Request<Body>> for SignatureValidation<Inner, ResBody>
where
    Inner: Service<Request<Body>, Response = ResBody> + Send + Sync + Clone + 'static,
    Inner::Error: IntoResponse + Send,
    Inner::Future: Send,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
{
    type Response = ResBody;
    type Error = Wrapper<Inner::Error>;
    type Future =
        std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let fut = checked(self.key.clone(), req)
            .map_err(Into::into)
            .and_then(move |req| inner.call(req).map_err(Into::into));
        Box::pin(fut)
    }
}
