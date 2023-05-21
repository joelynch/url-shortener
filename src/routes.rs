use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
    Json,
};

use chrono::NaiveDateTime;
use sqlx::{query, query_as, PgPool};
use tracing::{error, info};

use crate::AppState;
use crate::{db::Url, shortener::Shortener};

#[derive(Debug, serde::Deserialize)]
pub struct PostShortenBody {
    pub url: String,
}

#[derive(Debug, serde::Serialize)]
pub struct PostShortenResponse {
    pub url: String,
    pub short_url: String,
    pub code: String,
}

/// POST /shorten
/// Returns a shortened URL for the provided url that will redirect to the original URL.
pub async fn post_shorten(
    State(state): State<Arc<AppState>>,
    Json(url): Json<PostShortenBody>,
) -> Result<Json<PostShortenResponse>, StatusCode> {
    if !valid_url(&url.url) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut shortener = Shortener::new(state.strategy, url.url.clone());
    loop {
        let code = shortener.next_shortened();
        let short_url = format!("{}/s/{}", state.host, code);
        match insert_url(&state.pool, &url.url, &code, &short_url).await {
            Ok(url) => {
                let response = PostShortenResponse {
                    url: url.url,
                    short_url: url.short_url,
                    code: url.code,
                };
                info!("succeeded! {:?}", response);
                return Ok(Json(response));
            }
            Err(sqlx::Error::Database(db_err)) => {
                if db_err.constraint().is_some() {
                    info!("Code {} already exists, trying again", code);
                    continue;
                } else {
                    error!("Database error: {:?}", db_err);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
            Err(err) => {
                error!("Database error: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
}

async fn insert_url(
    pool: &PgPool,
    url: &str,
    code: &str,
    short_url: &str,
) -> Result<Url, sqlx::Error> {
    query_as("INSERT INTO urls (code, url, short_url) VALUES ($1, $2, $3) RETURNING *")
        .bind(code)
        .bind(url)
        .bind(short_url)
        .fetch_one(pool)
        .await
}

/// GET /s/:code
/// Redirects to the original URL for the provided code.
pub async fn get_url(
    Path(code): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Redirect, StatusCode> {
    let url: Url = query_as("SELECT * FROM urls WHERE code = $1")
        .bind(&code)
        .fetch_one(&state.pool)
        .await
        .map_err(sqlx_err_to_status_code)?;

    query("INSERT INTO hits (url_id) VALUES ($1)")
        .bind(url.id)
        .execute(&state.pool)
        .await
        .map_err(sqlx_err_to_status_code)?;

    Ok(Redirect::to(&url.url))
}

#[derive(serde::Serialize)]
pub struct StatsResponse {
    pub url: String,
    pub short_url: String,
    pub hits: i64,
    pub last_hit: Option<NaiveDateTime>,
}

/// GET /stats/:code
/// Returns the number of hits and the most recent hit for the provided code.
pub async fn get_stats(
    Path(code): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<StatsResponse>, StatusCode> {
    let url: Url = query_as("SELECT * FROM urls WHERE code = $1")
        .bind(code)
        .fetch_one(&state.pool)
        .await
        .map_err(sqlx_err_to_status_code)?;

    let (hits, last_hit): (i64, Option<NaiveDateTime>) =
        query_as("SELECT COUNT(*), MAX(created_at) FROM hits WHERE url_id = $1")
            .bind(url.id)
            .fetch_one(&state.pool)
            .await
            .map_err(sqlx_err_to_status_code)?;

    Ok(Json(StatsResponse {
        url: url.url,
        short_url: url.short_url,
        hits,
        last_hit,
    }))
}

fn valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

fn sqlx_err_to_status_code(e: sqlx::Error) -> StatusCode {
    match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        e => {
            error!("Database error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_url() {
        assert_eq!(valid_url("http://www.google.com"), true);
        assert_eq!(valid_url("http://www.goo|gle.com"), false);
    }
}
