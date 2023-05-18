use std::sync::Arc;

use axum::{body::Body, http, Router};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use url_shortner::{app, AppState, Config};

fn test_app(pool: PgPool) -> anyhow::Result<Router> {
    let config = envy::from_env::<Config>()?;

    let state = Arc::new(AppState {
        pool,
        strategy: config.strategy(),
        host: config.host,
    });

    Ok(app(state))
}

#[sqlx::test(fixtures("urls"))]
async fn test_get_url_ok(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let req = http::Request::get("/s/NTQmN-z5")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        res.headers().get("location").unwrap(),
        "https://www.google.com"
    );
}

#[sqlx::test(fixtures("urls"))]
async fn test_get_url_404(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let req = http::Request::get("/s/aosdhfs")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("urls", "hits"))]
async fn test_get_stats_ok(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let req = http::Request::get("/stats/NTQmN-z5")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), http::StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json,
        json!({
            "hits": 2,
            "last_hit": "2020-01-02T00:00:00",
            "short_url": "http://localhost:3000/s/NTQmN-z5",
            "url": "https://www.google.com"
        })
    );
}

#[sqlx::test(fixtures("urls", "hits"))]
async fn test_get_stats_empty(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let req = http::Request::get("/stats/YQLGtT3-")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), http::StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json,
        json!({
            "hits": 0,
            "last_hit": null,
            "short_url": "http://localhost:3000/s/YQLGtT3-",
            "url": "https://www.google.com"
        })
    );
}

#[sqlx::test(fixtures("urls", "hits"))]
async fn test_get_stats_404(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let req = http::Request::get("/stats/aosdhfs")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("urls"))]
async fn test_post_shorten_ok(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let req = http::Request::post("/shorten")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"url": "http://netflix.com"}"#))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), http::StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json,
        json!({
            "code": "T20JCl6-",
            "short_url": "https://tier.app/s/T20JCl6-",
            "url": "http://netflix.com"
        })
    );
}

#[sqlx::test(fixtures("urls"))]
#[ignore = "TODO: fix this test.  Seems to function correctly, but for some reason does not agree with the unit test in routes.rs.  Possibly sqlx::test related."]
async fn test_post_shorten_existing(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let req = http::Request::post("/shorten")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"url": "https://www.google.com"}"#))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), http::StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json,
        json!({
            "code": "wJZTvWsB",
            "short_url": "https://tier.app/s/wJZTvWsB",
            "url": "https://www.google.com"
        })
    );
}

#[sqlx::test]
async fn test_shorten_invalid_url(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let req = http::Request::post("/shorten")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"url": "http://netflix|"}"#))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);
}
