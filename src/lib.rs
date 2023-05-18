use std::sync::Arc;

use axum::{
    extract::MatchedPath,
    http::Request,
    routing::{get, post},
    Router,
};

use shortener::ShorteningStrategy;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing::info_span;

mod config;
mod db;
mod routes;
mod shortener;

pub use config::Config;
pub use db::build_pool;

pub fn app(state: Arc<AppState>) -> Router {
    // Log each request
    let tracing_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(MatchedPath::as_str);

        info_span!(
            "http_request",
            method = ?request.method(),
            matched_path,
        )
    });
    Router::new()
        .route("/shorten", post(routes::post_shorten))
        .route("/s/:code", get(routes::get_url))
        .route("/stats/:code", get(routes::get_stats))
        .with_state(state)
        .layer(tracing_layer)
}

pub struct AppState {
    pub pool: PgPool,
    pub host: String,
    pub strategy: ShorteningStrategy,
}
