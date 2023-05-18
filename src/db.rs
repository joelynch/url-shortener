use chrono::NaiveDateTime;
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};

// Could be a config option
const MAX_CONNECTIONS: u32 = 10;

pub async fn build_pool(db_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect(db_url)
        .await?;
    Ok(pool)
}

// Models matching the database schema in db.up.sql
#[derive(Debug, FromRow)]
pub struct Url {
    pub id: i32,
    pub code: String,
    pub url: String,
    pub short_url: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, FromRow)]
pub struct Hit {
    pub id: i32,
    pub url_id: i32,
    pub created_at: NaiveDateTime,
}
