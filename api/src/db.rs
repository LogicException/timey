use std::path::Path;
use std::str::FromStr;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

use crate::error::{AppError, AppResult};

pub async fn connect(database_url: &str) -> AppResult<SqlitePool> {
    ensure_sqlite_parent_dir(database_url)?;

    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|err| AppError::Config(format!("DATABASE_URL ungültig: {err}")))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    let max_connections = if is_memory_db(database_url) { 1 } else { 5 };

    SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await
        .map_err(AppError::from)
}

pub async fn migrate(pool: &SqlitePool) -> AppResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|err| AppError::Internal(format!("Migration fehlgeschlagen: {err}")))
}

fn is_memory_db(database_url: &str) -> bool {
    database_url.contains(":memory:") || database_url.contains("mode=memory")
}

fn ensure_sqlite_parent_dir(database_url: &str) -> AppResult<()> {
    let Some(path) = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))
    else {
        return Ok(());
    };

    if path.starts_with(':') || path.contains("mode=memory") {
        return Ok(());
    }

    let file_path = Path::new(path);
    if let Some(parent) = file_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|err| {
            AppError::Config(format!(
                "Datenverzeichnis konnte nicht erstellt werden: {err}"
            ))
        })?;
    }
    Ok(())
}
