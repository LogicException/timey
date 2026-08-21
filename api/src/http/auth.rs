use axum::Json;
use axum::extract::State;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::http::extractors::CurrentUser;
use crate::services::{sessions, users};
use crate::state::AppState;

#[derive(Serialize)]
pub struct AuthConfig {
    local: bool,
    oidc: bool,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserView {
    id: i64,
    username: String,
    role: String,
}

pub async fn config(State(state): State<AppState>) -> Json<AuthConfig> {
    Json(AuthConfig {
        local: state.config.auth_local_enabled,
        oidc: state.config.auth_oidc_enabled,
    })
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> AppResult<(CookieJar, Json<UserView>)> {
    if !state.config.auth_local_enabled {
        return Err(AppError::Unprocessable(
            "Lokales Login ist deaktiviert".into(),
        ));
    }
    let now = chrono::Utc::now();
    let user = users::authenticate_local(&state.pool, &body.username, &body.password).await?;
    let session =
        sessions::establish_session(&state.pool, user.id, now, state.config.session_ttl).await?;
    let cookie = session_cookie(&state, &session.token);
    Ok((
        jar.add(cookie),
        Json(UserView {
            id: user.id,
            username: user.username,
            role: user.role,
        }),
    ))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<(CookieJar, Json<serde_json::Value>)> {
    if let Some(cookie) = jar.get(&state.config.cookie_name) {
        sessions::revoke_session(&state.pool, cookie.value()).await?;
    }
    let mut expired = Cookie::new(state.config.cookie_name.clone(), String::new());
    expired.set_path("/");
    expired.set_http_only(true);
    expired.set_max_age(cookie::time::Duration::ZERO);
    Ok((jar.remove(expired), Json(serde_json::json!({ "ok": true }))))
}

pub async fn me(user: CurrentUser) -> Json<UserView> {
    Json(UserView {
        id: user.id,
        username: user.username,
        role: user.role.as_str().to_string(),
    })
}

fn session_cookie(state: &AppState, token: &str) -> Cookie<'static> {
    let mut cookie = Cookie::new(state.config.cookie_name.clone(), token.to_string());
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(state.config.cookie_secure);
    let max_age = i64::try_from(state.config.session_ttl.as_secs()).unwrap_or(i64::MAX);
    cookie.set_max_age(cookie::time::Duration::seconds(max_age));
    cookie
}
