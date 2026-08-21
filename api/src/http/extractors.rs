use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::cookie::CookieJar;

use crate::error::{AppError, AppResult};
use crate::models::{Role, UserRow};
use crate::services::{midnight, sessions, users};
use crate::state::AppState;

#[derive(Clone)]
pub struct CurrentUser {
    pub id: i64,
    pub username: String,
    pub role: Role,
}

impl CurrentUser {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }
}

pub fn require_admin(user: &CurrentUser) -> AppResult<()> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn token_from_jar(jar: &CookieJar, cookie_name: &str) -> Option<String> {
    jar.get(cookie_name)
        .map(|cookie| cookie.value().to_string())
}

async fn load_user(state: &AppState, token: &str) -> AppResult<CurrentUser> {
    let now = chrono::Utc::now();
    let user_id = sessions::user_id_for_token(&state.pool, token, now).await?;
    midnight::close_stale_for_user(&state.pool, user_id, now).await?;
    let user = users::get_user(&state.pool, user_id).await?;
    if user.disabled {
        return Err(AppError::Unauthorized);
    }
    to_current(user)
}

fn to_current(user: UserRow) -> AppResult<CurrentUser> {
    let role =
        Role::parse(&user.role).ok_or_else(|| AppError::Internal("Unbekannte Rolle".into()))?;
    Ok(CurrentUser {
        id: user.id,
        username: user.username,
        role,
    })
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let token =
            token_from_jar(&jar, &state.config.cookie_name).ok_or(AppError::Unauthorized)?;
        load_user(state, &token).await
    }
}
