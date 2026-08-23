use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::{NamedRow, ProjectRow};

const DEFAULT_TASKS: [&str; 7] = [
    "Meeting",
    "E-Mail",
    "Coding",
    "Termin",
    "Feature",
    "Qualitätssicherung",
    "Wartung",
];

pub async fn seed_default_tasks(
    pool: &SqlitePool,
    user_id: i64,
    now: DateTime<Utc>,
) -> AppResult<()> {
    let created_at = now.to_rfc3339();
    for name in DEFAULT_TASKS {
        sqlx::query(
            "INSERT OR IGNORE INTO tasks (user_id, name, archived, created_at) VALUES (?, ?, 0, ?)",
        )
        .bind(user_id)
        .bind(name)
        .bind(&created_at)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn seed_default_tasks_for_all_users(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> AppResult<()> {
    let user_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM users")
        .fetch_all(pool)
        .await?;
    for user_id in user_ids {
        seed_default_tasks(pool, user_id, now).await?;
    }
    Ok(())
}

pub async fn list_projects(
    pool: &SqlitePool,
    include_archived: bool,
) -> AppResult<Vec<ProjectRow>> {
    if include_archived {
        Ok(sqlx::query_as::<_, ProjectRow>(
            "SELECT id, name, archived, created_by, created_at FROM projects ORDER BY name COLLATE NOCASE",
        )
        .fetch_all(pool)
        .await?)
    } else {
        Ok(sqlx::query_as::<_, ProjectRow>(
            "SELECT id, name, archived, created_by, created_at FROM projects WHERE archived = 0 ORDER BY name COLLATE NOCASE",
        )
        .fetch_all(pool)
        .await?)
    }
}

pub async fn create_project(
    pool: &SqlitePool,
    name: &str,
    created_by: i64,
    now: DateTime<Utc>,
) -> AppResult<ProjectRow> {
    let name = validate_name(name)?;
    let created_at = now.to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO projects (name, archived, created_by, created_at) VALUES (?, 0, ?, ?)",
    )
    .bind(&name)
    .bind(created_by)
    .bind(&created_at)
    .execute(pool)
    .await;

    match result {
        Ok(done) => get_project(pool, done.last_insert_rowid()).await,
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(AppError::Conflict("Projektname bereits vergeben".into()))
        }
        Err(err) => Err(err.into()),
    }
}

pub async fn get_project(pool: &SqlitePool, id: i64) -> AppResult<ProjectRow> {
    sqlx::query_as::<_, ProjectRow>(
        "SELECT id, name, archived, created_by, created_at FROM projects WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn set_project_archived(
    pool: &SqlitePool,
    id: i64,
    archived: bool,
) -> AppResult<ProjectRow> {
    let done = sqlx::query("UPDATE projects SET archived = ? WHERE id = ?")
        .bind(archived)
        .bind(id)
        .execute(pool)
        .await?;
    if done.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    get_project(pool, id).await
}

pub async fn list_tasks(
    pool: &SqlitePool,
    user_id: i64,
    include_archived: bool,
) -> AppResult<Vec<NamedRow>> {
    let sql = if include_archived {
        "SELECT id, user_id, name, archived, created_at FROM tasks WHERE user_id = ? ORDER BY name COLLATE NOCASE"
    } else {
        "SELECT id, user_id, name, archived, created_at FROM tasks WHERE user_id = ? AND archived = 0 ORDER BY name COLLATE NOCASE"
    };
    Ok(sqlx::query_as::<_, NamedRow>(sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?)
}

pub async fn create_task(
    pool: &SqlitePool,
    user_id: i64,
    name: &str,
    now: DateTime<Utc>,
) -> AppResult<NamedRow> {
    let name = validate_name(name)?;
    let created_at = now.to_rfc3339();
    let result =
        sqlx::query("INSERT INTO tasks (user_id, name, archived, created_at) VALUES (?, ?, 0, ?)")
            .bind(user_id)
            .bind(&name)
            .bind(&created_at)
            .execute(pool)
            .await;

    match result {
        Ok(done) => get_task(pool, user_id, done.last_insert_rowid()).await,
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(AppError::Conflict("Task bereits vorhanden".into()))
        }
        Err(err) => Err(err.into()),
    }
}

pub async fn get_task(pool: &SqlitePool, user_id: i64, id: i64) -> AppResult<NamedRow> {
    sqlx::query_as::<_, NamedRow>(
        "SELECT id, user_id, name, archived, created_at FROM tasks WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn set_task_archived(
    pool: &SqlitePool,
    user_id: i64,
    id: i64,
    archived: bool,
) -> AppResult<NamedRow> {
    let done = sqlx::query("UPDATE tasks SET archived = ? WHERE id = ? AND user_id = ?")
        .bind(archived)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    if done.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    get_task(pool, user_id, id).await
}

pub async fn rename_task(
    pool: &SqlitePool,
    user_id: i64,
    id: i64,
    name: &str,
) -> AppResult<NamedRow> {
    let name = validate_name(name)?;
    let result = sqlx::query("UPDATE tasks SET name = ? WHERE id = ? AND user_id = ?")
        .bind(&name)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await;

    match result {
        Ok(done) if done.rows_affected() == 0 => Err(AppError::NotFound),
        Ok(_) => get_task(pool, user_id, id).await,
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(AppError::Conflict("Task bereits vorhanden".into()))
        }
        Err(err) => Err(err.into()),
    }
}

fn validate_name(name: &str) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 120 {
        return Err(AppError::Unprocessable(
            "Name muss zwischen 1 und 120 Zeichen haben".into(),
        ));
    }
    Ok(trimmed.to_string())
}
