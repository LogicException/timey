use axum::Json;
use axum::extract::State;
use serde::Serialize;
use sqlx::sqlite::SqliteQueryResult;

use crate::error::AppResult;
use crate::state::AppState;

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
}

pub async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

pub async fn health_db(State(state): State<AppState>) -> AppResult<Json<Health>> {
    let _: SqliteQueryResult = sqlx::query("SELECT 1").execute(&state.pool).await?;
    Ok(Json(Health { status: "ok" }))
}
