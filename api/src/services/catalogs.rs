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

pub async fn list_user_items(
    pool: &SqlitePool,
    table: UserItemTable,
    user_id: i64,
    include_archived: bool,
) -> AppResult<Vec<NamedRow>> {
    let sql = match (table, include_archived) {
        (UserItemTable::Task, true) => {
            "SELECT id, user_id, name, archived, created_at FROM tasks WHERE user_id = ? ORDER BY name COLLATE NOCASE"
        }
        (UserItemTable::Task, false) => {
            "SELECT id, user_id, name, archived, created_at FROM tasks WHERE user_id = ? AND archived = 0 ORDER BY name COLLATE NOCASE"
        }
        (UserItemTable::Aufgabe, true) => {
            "SELECT id, user_id, name, archived, created_at FROM aufgaben WHERE user_id = ? ORDER BY name COLLATE NOCASE"
        }
        (UserItemTable::Aufgabe, false) => {
            "SELECT id, user_id, name, archived, created_at FROM aufgaben WHERE user_id = ? AND archived = 0 ORDER BY name COLLATE NOCASE"
        }
    };
    Ok(sqlx::query_as::<_, NamedRow>(sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?)
}

pub async fn create_user_item(
    pool: &SqlitePool,
    table: UserItemTable,
    user_id: i64,
    name: &str,
    now: DateTime<Utc>,
) -> AppResult<NamedRow> {
    let name = validate_name(name)?;
    let created_at = now.to_rfc3339();
    let sql = match table {
        UserItemTable::Task => {
            "INSERT INTO tasks (user_id, name, archived, created_at) VALUES (?, ?, 0, ?)"
        }
        UserItemTable::Aufgabe => {
            "INSERT INTO aufgaben (user_id, name, archived, created_at) VALUES (?, ?, 0, ?)"
        }
    };
    let result = sqlx::query(sql)
        .bind(user_id)
        .bind(&name)
        .bind(&created_at)
        .execute(pool)
        .await;

    match result {
        Ok(done) => get_user_item(pool, table, user_id, done.last_insert_rowid()).await,
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => Err(AppError::Conflict(
            format!("{} bereits vorhanden", table.label()),
        )),
        Err(err) => Err(err.into()),
    }
}

pub async fn get_user_item(
    pool: &SqlitePool,
    table: UserItemTable,
    user_id: i64,
    id: i64,
) -> AppResult<NamedRow> {
    let sql = match table {
        UserItemTable::Task => {
            "SELECT id, user_id, name, archived, created_at FROM tasks WHERE id = ? AND user_id = ?"
        }
        UserItemTable::Aufgabe => {
            "SELECT id, user_id, name, archived, created_at FROM aufgaben WHERE id = ? AND user_id = ?"
        }
    };
    sqlx::query_as::<_, NamedRow>(sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn set_user_item_archived(
    pool: &SqlitePool,
    table: UserItemTable,
    user_id: i64,
    id: i64,
    archived: bool,
) -> AppResult<NamedRow> {
    let sql = match table {
        UserItemTable::Task => "UPDATE tasks SET archived = ? WHERE id = ? AND user_id = ?",
        UserItemTable::Aufgabe => "UPDATE aufgaben SET archived = ? WHERE id = ? AND user_id = ?",
    };
    let done = sqlx::query(sql)
        .bind(archived)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    if done.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    get_user_item(pool, table, user_id, id).await
}

#[derive(Debug, Clone, Copy)]
pub enum UserItemTable {
    Task,
    Aufgabe,
}

impl UserItemTable {
    fn label(self) -> &'static str {
        match self {
            Self::Task => "Task",
            Self::Aufgabe => "Aufgabe",
        }
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
