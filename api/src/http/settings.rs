use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::http::extractors::CurrentUser;
use crate::services::settings::{self, UserSettings};
use crate::state::AppState;

#[derive(Serialize)]
pub struct SettingsView {
    work_start: String,
    work_end: String,
    default_view: String,
}

impl From<UserSettings> for SettingsView {
    fn from(settings: UserSettings) -> Self {
        Self {
            work_start: settings.hours.work_start(),
            work_end: settings.hours.work_end(),
            default_view: settings.default_view.as_str().to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct PatchSettings {
    work_start: String,
    work_end: String,
    default_view: Option<String>,
}

pub async fn get_settings(
    user: CurrentUser,
    State(state): State<AppState>,
) -> AppResult<Json<SettingsView>> {
    let settings = settings::get_or_default(&state.pool, user.id).await?;
    Ok(Json(SettingsView::from(settings)))
}

pub async fn patch_settings(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<PatchSettings>,
) -> AppResult<Json<SettingsView>> {
    let settings = settings::update(
        &state.pool,
        user.id,
        &body.work_start,
        &body.work_end,
        body.default_view.as_deref(),
    )
    .await?;
    Ok(Json(SettingsView::from(settings)))
}
