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

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;
    use crate::error;
    use hyper::{Body, Request, Response};
    use std::convert::Infallible;
    use super::{SignatureValidation, Key};
    use tower::{Service, service_fn};

    #[allow(dead_code)]
    const PRIVATE_KEY: &[u8] = b"3053020101300506032b657004220420479fb153f22a36dc0764ca66b1d46e7c47219a0f1ce14084c3bae771340bf3d3a1230321004cd5fbef483a6c819d93635f5f0d1c57e48fb66a935d8ae5f2e2f0041dab2035";
    const PUBLIC_KEY: &[u8] = b"4cd5fbef483a6c819d93635f5f0d1c57e48fb66a935d8ae5f2e2f0041dab2035";
    const PAYLOAD: &[u8] = b"{}";
    const TIMESTAMP: &[u8] = b"1640995200";
    const SIGNATURE: &[u8] = b"90aaa91f005715c8e82d7ca54b34933cf60c5e9cdbd2880a42e2e41441c32651a7e75aef133192a01141c317161cb060c338748672cb07da45c3aa5a31344601";

    async fn dummy(_: Request<Body>) -> Result<Response<Body>, Infallible> {
        Ok(Response::builder().status(501).body("Not implemented".into()).unwrap())
    }

    fn wrap<S>(inner: S) -> SignatureValidation<S, Response<Body>>
    where
        S: Service<Request<Body>, Response = Response<Body>> + Send + Sync + Clone + 'static,
        S::Error: IntoResponse + Send,
        S::Future: Send,
    {
        SignatureValidation {
            inner,
            key: Key::from(PUBLIC_KEY).unwrap(),
        }
    }

    #[tokio::test]
    async fn test_validates_signature() {
        let req = Request::post("/")
            .header("X-Signature-Timestamp", TIMESTAMP)
            .header("X-Signature-Ed25519", SIGNATURE)
            .body(PAYLOAD.into())
            .unwrap();

        let mut stack = wrap(service_fn(dummy));

        let res = stack.call(req).await.unwrap();
        assert_eq!(501, res.status());
    }

    #[tokio::test]
    async fn test_rejects_invalid_signature() {
        let mut signature = SIGNATURE.to_owned();
        signature[9] = b'f';

        let req = Request::post("/")
            .header("X-Signature-Timestamp", TIMESTAMP)
            .header("X-Signature-Ed25519", &signature[..])
            .body(PAYLOAD.into())
            .unwrap();

        let mut stack = wrap(service_fn(dummy));

        let res = stack.call(req).await;
        assert!(
            matches!(res, Err(error::Wrapper::Own(error::Error::InvalidSignature)))
        );
    }

    #[tokio::test]
    async fn test_requires_timestamp() {
        let req = Request::post("/")
            .header("X-Signature-Ed25519", SIGNATURE)
            .body(PAYLOAD.into())
            .unwrap();

        let mut stack = wrap(service_fn(dummy));

        let res = stack.call(req).await;
        assert!(
            matches!(res, Err(error::Wrapper::Own(error::Error::MissingSignature)))
        );
    }

    #[tokio::test]
    async fn test_requires_signature() {
        let req = Request::post("/")
            .header("X-Signature-Timestamp", TIMESTAMP)
            .body(PAYLOAD.into())
            .unwrap();

        let mut stack = wrap(service_fn(dummy));

        let res = stack.call(req).await;
        assert!(
            matches!(res, Err(error::Wrapper::Own(error::Error::MissingSignature)))
        );
    }
}
