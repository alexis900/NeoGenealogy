use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::{path::Path, str::FromStr};

use crate::error::StorageError;

pub async fn establish_pool(database_url: &str) -> Result<SqlitePool, StorageError> {
    let opts = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5))
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;
    // Ensure FK enabled for this connection
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;
    Ok(pool)
}

pub async fn establish_pool_from_path(path: &Path) -> Result<SqlitePool, StorageError> {
    let url = format!("sqlite://{}", path.display());
    establish_pool(&url).await
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), StorageError> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

pub async fn in_memory_pool() -> Result<SqlitePool, StorageError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;
    run_migrations(&pool).await?;
    Ok(pool)
}
