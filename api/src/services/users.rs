use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::{Role, UserRow};
use crate::services::catalogs::seed_default_tasks;
use crate::services::crypto::{hash_password, verify_password};

pub async fn count_users(pool: &SqlitePool) -> AppResult<i64> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn bootstrap_admin(
    pool: &SqlitePool,
    username: &str,
    password: &str,
    now: DateTime<Utc>,
) -> AppResult<Option<UserRow>> {
    if count_users(pool).await? > 0 {
        return Ok(None);
    }
    let user = create_user(pool, username, password, Role::Admin, now).await?;
    tracing::info!(username = user.username, "bootstrap-admin angelegt");
    Ok(Some(user))
}

pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
    role: Role,
    now: DateTime<Utc>,
) -> AppResult<UserRow> {
    let username = validate_username(username)?;
    validate_password(password)?;
    let password_hash = hash_password(password)?;
    let created_at = now.to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, auth_provider, role, disabled, created_at)
         VALUES (?, ?, 'local', ?, 0, ?)",
    )
    .bind(&username)
    .bind(&password_hash)
    .bind(role.as_str())
    .bind(&created_at)
    .execute(pool)
    .await;

    match result {
        Ok(done) => {
            let id = done.last_insert_rowid();
            seed_default_tasks(pool, id, now).await?;
            get_user(pool, id).await
        }
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(AppError::Conflict("Benutzername bereits vergeben".into()))
        }
        Err(err) => Err(err.into()),
    }
}

pub async fn get_user(pool: &SqlitePool, id: i64) -> AppResult<UserRow> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, username, password_hash, auth_provider, oidc_subject, role, disabled, created_at
         FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn list_users(pool: &SqlitePool) -> AppResult<Vec<UserRow>> {
    Ok(sqlx::query_as::<_, UserRow>(
        "SELECT id, username, password_hash, auth_provider, oidc_subject, role, disabled, created_at
         FROM users ORDER BY username COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn find_by_username(pool: &SqlitePool, username: &str) -> AppResult<Option<UserRow>> {
    Ok(sqlx::query_as::<_, UserRow>(
        "SELECT id, username, password_hash, auth_provider, oidc_subject, role, disabled, created_at
         FROM users WHERE username = ? COLLATE NOCASE",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?)
}

pub async fn authenticate_local(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> AppResult<UserRow> {
    let user = find_by_username(pool, username)
        .await?
        .ok_or_else(|| AppError::Unauthorized)?;

    if user.disabled {
        return Err(AppError::Forbidden);
    }
    if user.auth_provider != "local" {
        return Err(AppError::Unprocessable(
            "Dieser Benutzer kann sich nicht lokal anmelden".into(),
        ));
    }
    let Some(hash) = user.password_hash.as_deref() else {
        return Err(AppError::Unauthorized);
    };
    if !verify_password(password, hash)? {
        return Err(AppError::Unauthorized);
    }
    Ok(user)
}

pub async fn update_user(
    pool: &SqlitePool,
    id: i64,
    password: Option<&str>,
    role: Option<Role>,
    disabled: Option<bool>,
) -> AppResult<UserRow> {
    let mut user = get_user(pool, id).await?;

    if let Some(password) = password {
        validate_password(password)?;
        user.password_hash = Some(hash_password(password)?);
    }
    if let Some(role) = role {
        user.role = role.as_str().to_string();
    }
    if let Some(disabled) = disabled {
        user.disabled = disabled;
    }

    sqlx::query("UPDATE users SET password_hash = ?, role = ?, disabled = ? WHERE id = ?")
        .bind(&user.password_hash)
        .bind(&user.role)
        .bind(user.disabled)
        .bind(id)
        .execute(pool)
        .await?;

    get_user(pool, id).await
}

fn validate_username(username: &str) -> AppResult<String> {
    let trimmed = username.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(AppError::Unprocessable(
            "Benutzername muss zwischen 1 und 64 Zeichen haben".into(),
        ));
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-')
    {
        return Err(AppError::Unprocessable(
            "Benutzername darf nur Buchstaben, Zahlen, Punkt, Unterstrich und Bindestrich enthalten"
                .into(),
        ));
    }
    Ok(trimmed.to_string())
}

fn validate_password(password: &str) -> AppResult<()> {
    if password.len() < 8 {
        return Err(AppError::Unprocessable(
            "Passwort muss mindestens 8 Zeichen haben".into(),
        ));
    }
    Ok(())
}
