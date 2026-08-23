use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use timey_api::config::Config;
use timey_api::db;
use timey_api::http::router;
use timey_api::models::Role;
use timey_api::services::users;
use timey_api::state::AppState;
use tower::ServiceExt;

struct TestCtx {
    app: axum::Router,
}

impl TestCtx {
    async fn new() -> Self {
        let pool = db::connect("sqlite::memory:").await.expect("memory db");
        db::migrate(&pool).await.expect("migrate");
        let now = chrono::Utc::now();
        users::create_user(&pool, "admin", "password1", Role::Admin, now)
            .await
            .expect("admin");
        let state = AppState {
            pool,
            config: Config::for_tests("sqlite::memory:".into()),
        };
        Self { app: router(state) }
    }

    async fn request(
        &self,
        method: &str,
        uri: &str,
        cookie: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value, Option<String>) {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(cookie) = cookie {
            builder = builder.header("cookie", format!("timey_session={cookie}"));
        }
        let request = if let Some(body) = body {
            builder
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request")
        } else {
            builder.body(Body::empty()).expect("request")
        };

        let response = self.app.clone().oneshot(request).await.expect("response");
        let status = response.status();
        let set_cookie = response
            .headers()
            .get("set-cookie")
            .and_then(|value| value.to_str().ok())
            .and_then(parse_session_cookie)
            .map(str::to_string);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| json!({ "raw": String::from_utf8_lossy(&bytes) }))
        };
        (status, json, set_cookie)
    }

    async fn login(&self, username: &str, password: &str) -> String {
        let (status, _, cookie) = self
            .request(
                "POST",
                "/api/auth/login",
                None,
                Some(json!({ "username": username, "password": password })),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        cookie.expect("session cookie")
    }
}

async fn start_work(ctx: &TestCtx, cookie: &str) {
    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/start",
            Some(cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

fn parse_session_cookie(header: &str) -> Option<&str> {
    header.split(';').next()?.strip_prefix("timey_session=")
}

#[tokio::test]
async fn health_returns_ok() {
    let ctx = TestCtx::new().await;
    let (status, body, _) = ctx.request("GET", "/api/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn health_db_returns_ok() {
    let ctx = TestCtx::new().await;
    let (status, body, _) = ctx.request("GET", "/api/health/db", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn auth_config_exposes_local_login() {
    let ctx = TestCtx::new().await;
    let (status, body, _) = ctx.request("GET", "/api/auth/config", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["local"], true);
    assert_eq!(body["oidc"], false);
}

#[tokio::test]
async fn login_and_me_roundtrip() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request("GET", "/api/auth/me", Some(&cookie), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["username"], "admin");
    assert_eq!(body["role"], "admin");
}

#[tokio::test]
async fn me_without_cookie_is_unauthorized() {
    let ctx = TestCtx::new().await;
    let (status, _, _) = ctx.request("GET", "/api/auth/me", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_can_create_user_and_seed_tasks() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/admin/users",
            Some(&cookie),
            Some(json!({
                "username": "enrico",
                "password": "password1",
                "role": "user"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["username"], "enrico");

    let user_cookie = ctx.login("enrico", "password1").await;
    let (status, tasks, _) = ctx
        .request("GET", "/api/tasks", Some(&user_cookie), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = tasks
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|row| row["name"].as_str())
        .collect();
    assert!(names.contains(&"Meeting"));
    assert!(names.contains(&"E-Mail"));
    assert!(names.contains(&"Coding"));
    assert!(names.contains(&"Termin"));
    assert!(names.contains(&"Feature"));
    assert!(names.contains(&"Qualitätssicherung"));
    assert!(names.contains(&"Wartung"));
}

#[tokio::test]
async fn non_admin_cannot_create_users() {
    let ctx = TestCtx::new().await;
    let admin = ctx.login("admin", "password1").await;
    let _ = ctx
        .request(
            "POST",
            "/api/admin/users",
            Some(&admin),
            Some(json!({
                "username": "enrico",
                "password": "password1",
                "role": "user"
            })),
        )
        .await;
    let user = ctx.login("enrico", "password1").await;
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/admin/users",
            Some(&user),
            Some(json!({
                "username": "other",
                "password": "password1",
                "role": "user"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn overlapping_entries_are_rejected() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let task_id = tasks[0]["id"].as_i64().expect("id");

    let (status, first, _) = ctx
        .request(
            "POST",
            "/api/entries",
            Some(&cookie),
            Some(json!({
                "task_id": task_id,
                "start_at": "2026-08-21T07:00:00Z",
                "end_at": "2026-08-21T08:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{first}");

    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/entries",
            Some(&cookie),
            Some(json!({
                "task_id": task_id,
                "start_at": "2026-08-21T07:30:00Z",
                "end_at": "2026-08-21T08:30:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
}

#[tokio::test]
async fn adjacent_entries_are_allowed() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let task_id = tasks[0]["id"].as_i64().expect("id");

    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/entries",
            Some(&cookie),
            Some(json!({
                "task_id": task_id,
                "start_at": "2026-08-21T07:00:00Z",
                "end_at": "2026-08-21T08:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/entries",
            Some(&cookie),
            Some(json!({
                "task_id": task_id,
                "start_at": "2026-08-21T08:00:00Z",
                "end_at": "2026-08-21T09:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn entry_spanning_days_is_rejected() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let task_id = tasks[0]["id"].as_i64().expect("id");

    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/entries",
            Some(&cookie),
            Some(json!({
                "task_id": task_id,
                "start_at": "2026-08-21T21:30:00Z",
                "end_at": "2026-08-21T22:30:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn timer_start_without_work_session_is_rejected() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/entries/timer/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"], "Arbeitszeit läuft nicht");
}

#[tokio::test]
async fn timer_start_while_work_paused_is_rejected() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/pause",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/entries/timer/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"], "Arbeitszeit läuft nicht");
}

#[tokio::test]
async fn work_pause_rejected_while_timer_running() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/entries/timer/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/pause",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"], "Zuerst den laufenden Eintrag stoppen");
}

#[tokio::test]
async fn work_stop_rejected_while_timer_running() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/entries/timer/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/stop",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"], "Zuerst den laufenden Eintrag stoppen");
}

#[tokio::test]
async fn play_timer_starts_without_task_and_stop_requires_task() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    start_work(&ctx, &cookie).await;
    let (status, running, _) = ctx
        .request(
            "POST",
            "/api/entries/timer/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{running}");
    assert_eq!(running["status"], "running");
    assert!(running["task_id"].is_null());

    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let task_id = tasks[0]["id"].as_i64().expect("id");
    let (status, stopped, _) = ctx
        .request(
            "POST",
            "/api/entries/timer/stop",
            Some(&cookie),
            Some(json!({ "task_id": task_id })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{stopped}");
    assert_eq!(stopped["status"], "complete");
    assert_eq!(stopped["task_id"], task_id);
}

#[tokio::test]
async fn running_timer_can_be_discarded_without_saving() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    start_work(&ctx, &cookie).await;
    let (status, running, _) = ctx
        .request(
            "POST",
            "/api/entries/timer/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{running}");
    let id = running["id"].as_i64().expect("id");

    let (status, _, _) = ctx
        .request("DELETE", &format!("/api/entries/{id}"), Some(&cookie), None)
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, timer, _) = ctx
        .request("GET", "/api/entries/timer", Some(&cookie), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(timer.is_null());
}

#[tokio::test]
async fn work_session_pause_and_resume() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, started, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{started}");
    assert_eq!(started["status"], "running");

    let (status, paused, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/pause",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{paused}");
    assert_eq!(paused["status"], "paused");

    let (status, resumed, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/resume",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{resumed}");
    assert_eq!(resumed["status"], "running");
}

#[tokio::test]
async fn work_sessions_range_returns_today_after_start() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, started, _) = ctx
        .request(
            "POST",
            "/api/work-sessions/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{started}");
    let day = started["local_date"].as_str().expect("local_date");

    let (status, body, _) = ctx
        .request(
            "GET",
            &format!("/api/work-sessions?from={day}&to={day}"),
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let days = body.as_array().expect("array");
    assert_eq!(days.len(), 1);
    assert_eq!(days[0]["local_date"], day);
    assert!(days[0]["elapsed_seconds"].as_i64().expect("seconds") >= 0);
}

#[tokio::test]
async fn work_sessions_range_is_empty_without_sessions() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "GET",
            "/api/work-sessions?from=2026-01-01&to=2026-01-02",
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!([]));
}

#[tokio::test]
async fn work_sessions_range_rejects_inverted_dates() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "GET",
            "/api/work-sessions?from=2026-08-22&to=2026-08-21",
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn csv_export_contains_header() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "GET",
            "/api/entries/export.csv?from=2026-08-21&to=2026-08-21",
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let raw = body["raw"].as_str().unwrap_or_default();
    assert!(raw.contains("Start,Ende,Dauer,Task,Projekt"));
    assert!(!raw.contains("Aufgabe"));
    assert!(raw.contains("Summe,,0:00,,"));
}

#[tokio::test]
async fn projects_are_shared_between_users() {
    let ctx = TestCtx::new().await;
    let admin = ctx.login("admin", "password1").await;
    let _ = ctx
        .request(
            "POST",
            "/api/admin/users",
            Some(&admin),
            Some(json!({
                "username": "enrico",
                "password": "password1",
                "role": "user"
            })),
        )
        .await;
    let (status, project, _) = ctx
        .request(
            "POST",
            "/api/projects",
            Some(&admin),
            Some(json!({ "name": "Kunde XYZ" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{project}");

    let user = ctx.login("enrico", "password1").await;
    let (status, projects, _) = ctx.request("GET", "/api/projects", Some(&user), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(projects[0]["name"], "Kunde XYZ");
}

#[tokio::test]
async fn non_admin_cannot_create_projects() {
    let ctx = TestCtx::new().await;
    let admin = ctx.login("admin", "password1").await;
    let _ = ctx
        .request(
            "POST",
            "/api/admin/users",
            Some(&admin),
            Some(json!({
                "username": "enrico",
                "password": "password1",
                "role": "user"
            })),
        )
        .await;
    let user = ctx.login("enrico", "password1").await;
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/projects",
            Some(&user),
            Some(json!({ "name": "Geheimes Projekt" })),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn settings_default_after_login() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request("GET", "/api/settings", Some(&cookie), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["work_start"], "07:30");
    assert_eq!(body["work_end"], "16:15");
    assert_eq!(body["default_view"], "day");
}

#[tokio::test]
async fn settings_without_cookie_is_unauthorized() {
    let ctx = TestCtx::new().await;
    let (status, _, _) = ctx.request("GET", "/api/settings", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn settings_patch_roundtrip() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "PATCH",
            "/api/settings",
            Some(&cookie),
            Some(json!({
                "work_start": "08:00",
                "work_end": "17:00",
                "default_view": "week"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["work_start"], "08:00");
    assert_eq!(body["work_end"], "17:00");
    assert_eq!(body["default_view"], "week");

    let (status, body, _) = ctx
        .request("GET", "/api/settings", Some(&cookie), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["work_start"], "08:00");
    assert_eq!(body["work_end"], "17:00");
    assert_eq!(body["default_view"], "week");
}

#[tokio::test]
async fn settings_patch_rejects_invalid_range() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "PATCH",
            "/api/settings",
            Some(&cookie),
            Some(json!({
                "work_start": "16:15",
                "work_end": "07:30"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"], "Beginn muss vor dem Ende liegen");
}

#[tokio::test]
async fn settings_patch_rejects_invalid_format() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "PATCH",
            "/api/settings",
            Some(&cookie),
            Some(json!({
                "work_start": "7:30",
                "work_end": "16:15"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(
        body["error"],
        "Arbeitszeit muss im Format HH:MM angegeben werden"
    );
}

#[tokio::test]
async fn settings_are_isolated_per_user() {
    let ctx = TestCtx::new().await;
    let admin = ctx.login("admin", "password1").await;
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/admin/users",
            Some(&admin),
            Some(json!({
                "username": "enrico",
                "password": "password1",
                "role": "user"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _, _) = ctx
        .request(
            "PATCH",
            "/api/settings",
            Some(&admin),
            Some(json!({
                "work_start": "09:00",
                "work_end": "18:00"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let user = ctx.login("enrico", "password1").await;
    let (status, body, _) = ctx.request("GET", "/api/settings", Some(&user), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["work_start"], "07:30");
    assert_eq!(body["work_end"], "16:15");
    assert_eq!(body["default_view"], "day");
}

#[tokio::test]
async fn settings_patch_rejects_invalid_default_view() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "PATCH",
            "/api/settings",
            Some(&cookie),
            Some(json!({
                "work_start": "08:00",
                "work_end": "17:00",
                "default_view": "month"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"], "Standardansicht muss day oder week sein");
}

#[tokio::test]
async fn settings_patch_without_default_view_keeps_stored_value() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, _, _) = ctx
        .request(
            "PATCH",
            "/api/settings",
            Some(&cookie),
            Some(json!({
                "work_start": "08:00",
                "work_end": "17:00",
                "default_view": "week"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body, _) = ctx
        .request(
            "PATCH",
            "/api/settings",
            Some(&cookie),
            Some(json!({
                "work_start": "09:00",
                "work_end": "18:00"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["work_start"], "09:00");
    assert_eq!(body["work_end"], "18:00");
    assert_eq!(body["default_view"], "week");
}

#[tokio::test]
async fn list_tasks_hides_system_unless_requested() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;

    let (status, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = tasks
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|row| row["name"].as_str())
        .collect();
    assert!(!names.contains(&"unbestimmt"));
    assert!(
        tasks
            .as_array()
            .expect("array")
            .iter()
            .all(|row| row["system"] == false)
    );

    let (status, tasks, _) = ctx
        .request("GET", "/api/tasks?include_system=true", Some(&cookie), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    let unbestimmt = tasks
        .as_array()
        .expect("array")
        .iter()
        .find(|row| row["name"] == "unbestimmt")
        .expect("system task");
    assert_eq!(unbestimmt["system"], true);
    assert_eq!(unbestimmt["archived"], false);
}

fn task_id_by_name(tasks: &Value, name: &str) -> i64 {
    tasks
        .as_array()
        .expect("array")
        .iter()
        .find(|row| row["name"] == name)
        .and_then(|row| row["id"].as_i64())
        .expect("task id")
}

#[tokio::test]
async fn patch_task_renames() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let id = task_id_by_name(&tasks, "Coding");

    let (status, body, _) = ctx
        .request(
            "PATCH",
            &format!("/api/tasks/{id}"),
            Some(&cookie),
            Some(json!({ "name": "Coding reviewen" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["name"], "Coding reviewen");
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn patch_task_rename_conflict() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let id = task_id_by_name(&tasks, "Coding");

    let (status, body, _) = ctx
        .request(
            "PATCH",
            &format!("/api/tasks/{id}"),
            Some(&cookie),
            Some(json!({ "name": "E-Mail" })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
}

#[tokio::test]
async fn patch_task_empty_name_is_unprocessable() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let id = task_id_by_name(&tasks, "Coding");

    let (status, body, _) = ctx
        .request(
            "PATCH",
            &format!("/api/tasks/{id}"),
            Some(&cookie),
            Some(json!({ "name": "   " })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn patch_task_empty_body_is_unprocessable() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let id = task_id_by_name(&tasks, "Coding");

    let (status, body, _) = ctx
        .request(
            "PATCH",
            &format!("/api/tasks/{id}"),
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn patch_task_foreign_id_is_not_found() {
    let ctx = TestCtx::new().await;
    let admin = ctx.login("admin", "password1").await;
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/admin/users",
            Some(&admin),
            Some(json!({
                "username": "enrico",
                "password": "password1",
                "role": "user"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let user = ctx.login("enrico", "password1").await;
    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&user), None).await;
    let foreign_id = task_id_by_name(&tasks, "Coding");

    let (status, _, _) = ctx
        .request(
            "PATCH",
            &format!("/api/tasks/{foreign_id}"),
            Some(&admin),
            Some(json!({ "name": "Fremd" })),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_task_reassigns_entries_and_removes_task() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let coding_id = task_id_by_name(&tasks, "Coding");
    let (_, system_tasks, _) = ctx
        .request("GET", "/api/tasks?include_system=true", Some(&cookie), None)
        .await;
    let unbestimmt_id = task_id_by_name(&system_tasks, "unbestimmt");
    start_work(&ctx, &cookie).await;

    let (status, complete, _) = ctx
        .request(
            "POST",
            "/api/entries",
            Some(&cookie),
            Some(json!({
                "task_id": coding_id,
                "start_at": "2026-08-21T07:00:00Z",
                "end_at": "2026-08-21T08:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{complete}");
    let complete_id = complete["id"].as_i64().expect("complete id");

    let (status, running, _) = ctx
        .request(
            "POST",
            "/api/entries/timer/start",
            Some(&cookie),
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{running}");
    let running_id = running["id"].as_i64().expect("running id");
    let (status, assigned, _) = ctx
        .request(
            "PATCH",
            &format!("/api/entries/{running_id}"),
            Some(&cookie),
            Some(json!({
                "task_id": coding_id,
                "start_at": running["start_at"]
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{assigned}");

    let (status, body, _) = ctx
        .request(
            "DELETE",
            &format!("/api/tasks/{coding_id}"),
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        tasks
            .as_array()
            .expect("array")
            .iter()
            .all(|row| row["name"] != "Coding")
    );

    let (status, entries, _) = ctx
        .request(
            "GET",
            "/api/entries?from=2026-08-21&to=2026-08-21",
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{entries}");
    let complete_row = entries
        .as_array()
        .expect("array")
        .iter()
        .find(|row| row["id"] == complete_id)
        .expect("complete entry");
    assert_eq!(complete_row["task_id"], unbestimmt_id);
    assert_eq!(complete_row["task_name"], "unbestimmt");

    let (status, timer, _) = ctx
        .request("GET", "/api/entries/timer", Some(&cookie), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{timer}");
    assert_eq!(timer["id"], running_id);
    assert_eq!(timer["task_id"], unbestimmt_id);
    assert_eq!(timer["status"], "running");
}

#[tokio::test]
async fn delete_system_task_is_conflict() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx
        .request("GET", "/api/tasks?include_system=true", Some(&cookie), None)
        .await;
    let id = task_id_by_name(&tasks, "unbestimmt");

    let (status, body, _) = ctx
        .request("DELETE", &format!("/api/tasks/{id}"), Some(&cookie), None)
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
}

#[tokio::test]
async fn patch_system_task_is_conflict() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx
        .request("GET", "/api/tasks?include_system=true", Some(&cookie), None)
        .await;
    let id = task_id_by_name(&tasks, "unbestimmt");

    let (status, body, _) = ctx
        .request(
            "PATCH",
            &format!("/api/tasks/{id}"),
            Some(&cookie),
            Some(json!({ "name": "anders" })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");

    let (status, body, _) = ctx
        .request(
            "PATCH",
            &format!("/api/tasks/{id}"),
            Some(&cookie),
            Some(json!({ "archived": true })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
}

#[tokio::test]
async fn reserved_task_name_is_unprocessable() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;

    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/tasks",
            Some(&cookie),
            Some(json!({ "name": "  Unbestimmt  " })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let id = task_id_by_name(&tasks, "Coding");
    let (status, body, _) = ctx
        .request(
            "PATCH",
            &format!("/api/tasks/{id}"),
            Some(&cookie),
            Some(json!({ "name": "unbestimmt" })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn create_entry_rejects_system_task() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx
        .request("GET", "/api/tasks?include_system=true", Some(&cookie), None)
        .await;
    let id = task_id_by_name(&tasks, "unbestimmt");

    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/entries",
            Some(&cookie),
            Some(json!({
                "task_id": id,
                "start_at": "2026-08-21T07:00:00Z",
                "end_at": "2026-08-21T08:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn update_entry_keeps_existing_system_task() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (_, tasks, _) = ctx.request("GET", "/api/tasks", Some(&cookie), None).await;
    let coding_id = task_id_by_name(&tasks, "Coding");
    let (_, system_tasks, _) = ctx
        .request("GET", "/api/tasks?include_system=true", Some(&cookie), None)
        .await;
    let unbestimmt_id = task_id_by_name(&system_tasks, "unbestimmt");

    let (status, created, _) = ctx
        .request(
            "POST",
            "/api/entries",
            Some(&cookie),
            Some(json!({
                "task_id": coding_id,
                "start_at": "2026-08-21T07:00:00Z",
                "end_at": "2026-08-21T08:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{created}");
    let entry_id = created["id"].as_i64().expect("entry id");

    let (status, _, _) = ctx
        .request(
            "DELETE",
            &format!("/api/tasks/{coding_id}"),
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, updated, _) = ctx
        .request(
            "PATCH",
            &format!("/api/entries/{entry_id}"),
            Some(&cookie),
            Some(json!({
                "task_id": unbestimmt_id,
                "start_at": "2026-08-21T07:00:00Z",
                "end_at": "2026-08-21T09:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["task_id"], unbestimmt_id);
    assert!(
        updated["end_at"]
            .as_str()
            .is_some_and(|value| value.starts_with("2026-08-21T09:00:00"))
    );
}
