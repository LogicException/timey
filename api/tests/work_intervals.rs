use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{Duration, SecondsFormat, TimeZone, Utc};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use timey_api::config::Config;
use timey_api::db;
use timey_api::http::router;
use timey_api::models::Role;
use timey_api::services::{users, work};
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

fn parse_session_cookie(header: &str) -> Option<&str> {
    header.split(';').next()?.strip_prefix("timey_session=")
}

fn rfc3339(dt: chrono::DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[tokio::test]
async fn create_closed_interval_appears_in_day_list() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let start = "2026-08-20T06:00:00Z";
    let end = "2026-08-20T10:00:00Z";

    let (status, created, _) = ctx
        .request(
            "POST",
            "/api/work-intervals",
            Some(&cookie),
            Some(json!({ "start_at": start, "end_at": end })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{created}");
    let id = created["id"].as_i64().expect("id");
    assert_eq!(created["open"], false);
    assert_eq!(
        chrono::DateTime::parse_from_rfc3339(created["start_at"].as_str().expect("start"))
            .expect("parse created start"),
        chrono::DateTime::parse_from_rfc3339(start).expect("parse start")
    );
    assert_eq!(
        chrono::DateTime::parse_from_rfc3339(created["end_at"].as_str().expect("end"))
            .expect("parse created end"),
        chrono::DateTime::parse_from_rfc3339(end).expect("parse end")
    );

    let (status, body, _) = ctx
        .request(
            "GET",
            "/api/work-sessions?from=2026-08-20&to=2026-08-20",
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let intervals = body[0]["intervals"].as_array().expect("intervals");
    assert_eq!(intervals.len(), 1);
    assert_eq!(intervals[0]["id"], id);
    assert_eq!(intervals[0]["open"], false);
    assert_eq!(
        chrono::DateTime::parse_from_rfc3339(
            intervals[0]["start_at"].as_str().expect("list start")
        )
        .expect("parse list start"),
        chrono::DateTime::parse_from_rfc3339(start).expect("parse start")
    );
    assert_eq!(body[0]["elapsed_seconds"], 4 * 3600);
}

#[tokio::test]
async fn overlapping_interval_is_conflict() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, first, _) = ctx
        .request(
            "POST",
            "/api/work-intervals",
            Some(&cookie),
            Some(json!({
                "start_at": "2026-08-20T06:00:00Z",
                "end_at": "2026-08-20T10:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{first}");

    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/work-intervals",
            Some(&cookie),
            Some(json!({
                "start_at": "2026-08-20T08:00:00Z",
                "end_at": "2026-08-20T09:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
}

#[tokio::test]
async fn adjacent_interval_is_allowed() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, _, _) = ctx
        .request(
            "POST",
            "/api/work-intervals",
            Some(&cookie),
            Some(json!({
                "start_at": "2026-08-20T06:00:00Z",
                "end_at": "2026-08-20T10:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/work-intervals",
            Some(&cookie),
            Some(json!({
                "start_at": "2026-08-20T10:00:00Z",
                "end_at": "2026-08-20T12:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn zero_duration_is_unprocessable() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/work-intervals",
            Some(&cookie),
            Some(json!({
                "start_at": "2026-08-20T06:00:00Z",
                "end_at": "2026-08-20T06:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn future_interval_is_unprocessable() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/work-intervals",
            Some(&cookie),
            Some(json!({
                "start_at": "2099-01-01T08:00:00Z",
                "end_at": "2099-01-01T09:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn spanning_midnight_is_unprocessable() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request(
            "POST",
            "/api/work-intervals",
            Some(&cookie),
            Some(json!({
                "start_at": "2026-08-20T21:30:00Z",
                "end_at": "2026-08-20T22:30:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn patch_running_start_increases_elapsed() {
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

    let (status, listed, _) = ctx
        .request(
            "GET",
            &format!("/api/work-sessions?from={day}&to={day}"),
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    let interval = &listed[0]["intervals"][0];
    assert_eq!(interval["open"], true);
    let id = interval["id"].as_i64().expect("id");
    let start = chrono::DateTime::parse_from_rfc3339(interval["start_at"].as_str().expect("start"))
        .expect("parse start")
        .with_timezone(&Utc);
    let earlier = start - Duration::hours(2);

    let (status, patched, _) = ctx
        .request(
            "PATCH",
            &format!("/api/work-intervals/{id}"),
            Some(&cookie),
            Some(json!({ "start_at": rfc3339(earlier) })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{patched}");

    let (status, after, _) = ctx
        .request(
            "GET",
            &format!("/api/work-sessions?from={day}&to={day}"),
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{after}");
    let elapsed = after[0]["elapsed_seconds"].as_i64().expect("elapsed");
    assert!(
        elapsed >= 2 * 3600,
        "expected at least two hours, got {elapsed}"
    );
}

#[tokio::test]
async fn patch_open_interval_rejects_end_at() {
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
    let (status, listed, _) = ctx
        .request(
            "GET",
            &format!("/api/work-sessions?from={day}&to={day}"),
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    let id = listed[0]["intervals"][0]["id"].as_i64().expect("id");
    let start = listed[0]["intervals"][0]["start_at"].clone();

    let (status, body, _) = ctx
        .request(
            "PATCH",
            &format!("/api/work-intervals/{id}"),
            Some(&cookie),
            Some(json!({
                "start_at": start,
                "end_at": "2026-08-20T10:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn delete_closed_interval_and_keep_neighbor() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, morning, _) = ctx
        .request(
            "POST",
            "/api/work-intervals",
            Some(&cookie),
            Some(json!({
                "start_at": "2026-08-20T06:00:00Z",
                "end_at": "2026-08-20T10:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{morning}");
    let morning_id = morning["id"].as_i64().expect("id");

    let (status, afternoon, _) = ctx
        .request(
            "POST",
            "/api/work-intervals",
            Some(&cookie),
            Some(json!({
                "start_at": "2026-08-20T11:00:00Z",
                "end_at": "2026-08-20T15:00:00Z"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{afternoon}");
    let afternoon_id = afternoon["id"].as_i64().expect("id");

    let (status, deleted, _) = ctx
        .request(
            "DELETE",
            &format!("/api/work-intervals/{morning_id}"),
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{deleted}");

    let (status, body, _) = ctx
        .request(
            "GET",
            "/api/work-sessions?from=2026-08-20&to=2026-08-20",
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let intervals = body[0]["intervals"].as_array().expect("intervals");
    assert_eq!(intervals.len(), 1);
    assert_eq!(intervals[0]["id"], afternoon_id);
}

#[tokio::test]
async fn delete_open_interval_is_unprocessable() {
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
    let (_status, listed, _) = ctx
        .request(
            "GET",
            &format!("/api/work-sessions?from={day}&to={day}"),
            Some(&cookie),
            None,
        )
        .await;
    let id = listed[0]["intervals"][0]["id"].as_i64().expect("id");

    let (status, body, _) = ctx
        .request(
            "DELETE",
            &format!("/api/work-intervals/{id}"),
            Some(&cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn unknown_interval_is_not_found() {
    let ctx = TestCtx::new().await;
    let cookie = ctx.login("admin", "password1").await;
    let (status, body, _) = ctx
        .request("DELETE", "/api/work-intervals/9999", Some(&cookie), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

async fn service_user() -> (sqlx::SqlitePool, i64) {
    let pool = db::connect("sqlite::memory:").await.expect("db");
    db::migrate(&pool).await.expect("migrate");
    let now = chrono::Utc
        .with_ymd_and_hms(2026, 8, 21, 8, 0, 0)
        .single()
        .expect("now");
    let user = users::create_user(&pool, "admin", "password1", Role::Admin, now)
        .await
        .expect("user");
    (pool, user.id)
}

fn ts(hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    chrono::Utc
        .with_ymd_and_hms(2026, 8, 21, hour, minute, 0)
        .single()
        .expect("valid utc")
}

#[tokio::test]
async fn timer_start_is_rejected_over_manual_interval() {
    let (pool, user_id) = service_user().await;
    work::create_interval(&pool, user_id, ts(8, 0), ts(12, 0), ts(18, 0))
        .await
        .expect("create");
    let result = work::start(&pool, user_id, ts(10, 0)).await;
    let err = match result {
        Ok(_) => panic!("expected overlap"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("überschneidet"));
}

#[tokio::test]
async fn deleting_morning_repairs_pause_resume_session() {
    let (pool, user_id) = service_user().await;
    work::start(&pool, user_id, ts(8, 0)).await.expect("start");
    work::pause(&pool, user_id, ts(12, 0)).await.expect("pause");
    work::resume(&pool, user_id, ts(13, 0))
        .await
        .expect("resume");
    work::stop(&pool, user_id, ts(17, 0)).await.expect("stop");

    let days = work::list_for_range(
        &pool,
        user_id,
        chrono::NaiveDate::from_ymd_opt(2026, 8, 21).expect("date"),
        chrono::NaiveDate::from_ymd_opt(2026, 8, 21).expect("date"),
        ts(18, 0),
    )
    .await
    .expect("list");
    assert_eq!(days[0].intervals.len(), 2);
    let morning_id = days[0].intervals[0].id;
    work::delete_interval(&pool, user_id, morning_id, ts(18, 0))
        .await
        .expect("delete");

    let days = work::list_for_range(
        &pool,
        user_id,
        chrono::NaiveDate::from_ymd_opt(2026, 8, 21).expect("date"),
        chrono::NaiveDate::from_ymd_opt(2026, 8, 21).expect("date"),
        ts(18, 0),
    )
    .await
    .expect("list after");
    assert_eq!(days[0].intervals.len(), 1);
    assert_eq!(days[0].intervals[0].start, ts(13, 0));
    assert_eq!(days[0].intervals[0].end, ts(17, 0));
}
