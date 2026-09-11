use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::AppResult;
use crate::http::extractors::CurrentUser;
use crate::services::settings::{self, UserSettings};
use crate::state::AppState;

#[derive(Serialize)]
pub struct SettingsView {
    work_start: String,
    work_end: String,
    default_view: String,
    slot_minutes: Option<i64>,
    default_task_id: Option<i64>,
}

impl From<UserSettings> for SettingsView {
    fn from(settings: UserSettings) -> Self {
        Self {
            work_start: settings.hours.work_start(),
            work_end: settings.hours.work_end(),
            default_view: settings.default_view.as_str().to_string(),
            slot_minutes: settings.slot_minutes.stored(),
            default_task_id: settings.default_task_id.stored(),
        }
    }
}

fn deserialize_double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::<T>::deserialize(deserializer)?))
}

#[derive(Deserialize)]
pub struct PatchSettings {
    work_start: String,
    work_end: String,
    default_view: Option<String>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    slot_minutes: Option<Option<i64>>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    default_task_id: Option<Option<i64>>,
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
        body.slot_minutes,
        body.default_task_id,
    )
    .await?;
    Ok(Json(SettingsView::from(settings)))
}
