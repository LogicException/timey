use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::services::crypto::{generate_session_token, hash_session_token};

pub struct NewSession {
    pub token: String,
}

pub async fn establish_session(
    pool: &SqlitePool,
    user_id: i64,
    now: DateTime<Utc>,
    ttl: std::time::Duration,
) -> AppResult<NewSession> {
    let token = generate_session_token();
    let token_hash = hash_session_token(&token);
    let ttl =
        Duration::from_std(ttl).map_err(|_| AppError::Internal("SESSION_TTL ungültig".into()))?;
    let expires_at = now + ttl;

    sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&token_hash)
        .bind(user_id)
        .bind(expires_at.to_rfc3339())
        .execute(pool)
        .await?;

    Ok(NewSession { token })
}

pub async fn user_id_for_token(
    pool: &SqlitePool,
    token: &str,
    now: DateTime<Utc>,
) -> AppResult<i64> {
    let token_hash = hash_session_token(token);
    let row: Option<(i64, String)> =
        sqlx::query_as("SELECT user_id, expires_at FROM sessions WHERE token_hash = ?")
            .bind(&token_hash)
            .fetch_optional(pool)
            .await?;

    let Some((user_id, expires_at)) = row else {
        return Err(AppError::Unauthorized);
    };
    let expires = crate::models::parse_rfc3339(&expires_at)
        .map_err(|_| AppError::Internal("Session-Ablauf ungültig".into()))?;
    if expires <= now {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(&token_hash)
            .execute(pool)
            .await?;
        return Err(AppError::Unauthorized);
    }
    Ok(user_id)
}

pub async fn revoke_session(pool: &SqlitePool, token: &str) -> AppResult<()> {
    let token_hash = hash_session_token(token);
    sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
        .bind(&token_hash)
        .execute(pool)
        .await?;
    Ok(())
}
