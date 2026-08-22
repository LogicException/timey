use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use crate::domain::working_hours::WorkingHours;
use crate::error::AppResult;
use crate::http::extractors::CurrentUser;
use crate::services::settings;
use crate::state::AppState;

#[derive(Serialize)]
pub struct SettingsView {
    work_start: String,
    work_end: String,
}

impl From<WorkingHours> for SettingsView {
    fn from(hours: WorkingHours) -> Self {
        Self {
            work_start: hours.work_start(),
            work_end: hours.work_end(),
        }
    }
}

#[derive(Deserialize)]
pub struct PatchSettings {
    work_start: String,
    work_end: String,
}

pub async fn get_settings(
    user: CurrentUser,
    State(state): State<AppState>,
) -> AppResult<Json<SettingsView>> {
    let hours = settings::get_or_default(&state.pool, user.id).await?;
    Ok(Json(SettingsView::from(hours)))
}

pub async fn patch_settings(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<PatchSettings>,
) -> AppResult<Json<SettingsView>> {
    let hours = settings::update(&state.pool, user.id, &body.work_start, &body.work_end).await?;
    Ok(Json(SettingsView::from(hours)))
}
