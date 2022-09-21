use axum::{
    error_handling::HandleErrorLayer, handler::Handler, http::StatusCode, routing::post, Router,
    Server,
};
use tower::ServiceBuilder;
use tower_signature::SignatureValidationLayer;
use tracing::Level;

async fn webhook() -> (StatusCode, &'static str) {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented")
}

async fn handle_error(err: axum::BoxError) -> (StatusCode, String) {
    if let Some(err) = err.downcast_ref::<tower_signature::Error>() {
        (err.into(), err.to_string())
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let key = std::env::var("DISCORD_PUBLIC_KEY").unwrap();
    let layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        .layer(SignatureValidationLayer::new(key.as_bytes()).unwrap());
    let handler = webhook.layer(layer);

    let app = Router::new()
        .route("/webhook", post(handler))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    Server::bind(&([127, 0, 0, 1], 3000).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
