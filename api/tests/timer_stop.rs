use chrono::{TimeZone, Utc};
use timey_api::db;
use timey_api::models::{Role, parse_rfc3339};
use timey_api::services::{catalogs, entries, users, work};

async fn setup() -> (sqlx::SqlitePool, i64, i64) {
    let pool = db::connect("sqlite::memory:").await.expect("db");
    db::migrate(&pool).await.expect("migrate");
    let start = Utc
        .with_ymd_and_hms(2026, 8, 21, 10, 0, 0)
        .single()
        .expect("start");
    let user = users::create_user(&pool, "admin", "password1", Role::Admin, start)
        .await
        .expect("user");
    work::start(&pool, user.id, start).await.expect("work");
    let task_id = catalogs::list_tasks(&pool, user.id, true, false)
        .await
        .expect("tasks")
        .into_iter()
        .find(|row| row.name == "Feature")
        .expect("feature")
        .id;
    (pool, user.id, task_id)
}

fn ts(hour: u32, minute: u32, second: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 21, hour, minute, second)
        .single()
        .expect("valid utc")
}

#[tokio::test]
async fn stop_timer_truncates_end_seconds() {
    let (pool, user_id, task_id) = setup().await;
    entries::start_timer(&pool, user_id, ts(10, 0, 0))
        .await
        .expect("start");

    let stopped = entries::stop_timer(&pool, user_id, task_id, None, ts(11, 0, 45))
        .await
        .expect("stop");
    let end = parse_rfc3339(stopped.end_at.as_deref().expect("end")).expect("parse end");
    assert_eq!(end, ts(11, 0, 0));
}

#[tokio::test]
async fn stop_timer_in_same_minute_bumps_end_to_next_minute() {
    let (pool, user_id, task_id) = setup().await;
    entries::start_timer(&pool, user_id, ts(10, 0, 30))
        .await
        .expect("start");

    let stopped = entries::stop_timer(&pool, user_id, task_id, None, ts(10, 0, 45))
        .await
        .expect("stop");
    let end = parse_rfc3339(stopped.end_at.as_deref().expect("end")).expect("parse end");
    assert_eq!(end, ts(10, 1, 0));
}

#[tokio::test]
async fn adjacent_manual_entry_after_truncated_timer_stop_is_allowed() {
    let (pool, user_id, task_id) = setup().await;
    entries::start_timer(&pool, user_id, ts(10, 0, 0))
        .await
        .expect("start");
    entries::stop_timer(&pool, user_id, task_id, None, ts(11, 0, 45))
        .await
        .expect("stop");

    let next = entries::create_entry(
        &pool,
        user_id,
        entries::NewEntry {
            task_id: Some(task_id),
            project_id: None,
            start_at: ts(11, 0, 0),
            end_at: Some(ts(12, 0, 0)),
        },
        ts(12, 0, 0),
    )
    .await
    .expect("adjacent entry");
    let start = parse_rfc3339(&next.start_at).expect("parse start");
    assert_eq!(start, ts(11, 0, 0));
}
