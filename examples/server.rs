use axum::{handler::Handler, http::StatusCode, routing::post, Router, Server};
use tower_signature::SignatureValidatorLayer;
use tracing::Level;

async fn webhook() -> (StatusCode, &'static str) {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let key = std::env::var("DISCORD_PUBLIC_KEY").unwrap();
    let app = Router::new()
        .route(
            "/webhook",
            post(webhook.layer(SignatureValidatorLayer::new(key.as_bytes()).unwrap())),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http());

    Server::bind(&([127, 0, 0, 1], 3000).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
