use std::sync::Arc;

use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Registry,
};
use url_shortener::{app, build_pool, AppState, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = envy::from_env::<Config>()?;

    // Set up logging
    Registry::default()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "url_shortener=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = build_pool(config.database_url.as_str()).await?;

    let state = Arc::new(AppState {
        pool,
        strategy: config.strategy(),
        host: config.host,
    });

    let app = app(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
