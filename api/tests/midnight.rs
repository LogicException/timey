use chrono::{Duration, TimeZone, Utc};
use timey_api::db;
use timey_api::models::{EntryStatus, Role};
use timey_api::services::{entries, midnight, users};

#[tokio::test]
async fn midnight_close_marks_running_entry_without_task() {
    let pool = db::connect("sqlite::memory:").await.expect("db");
    db::migrate(&pool).await.expect("migrate");
    let start = Utc
        .with_ymd_and_hms(2026, 8, 21, 10, 0, 0)
        .single()
        .unwrap();
    let user = users::create_user(&pool, "admin", "password1", Role::Admin, start)
        .await
        .expect("user");

    entries::start_timer(&pool, user.id, start)
        .await
        .expect("start");

    let next_day = start + Duration::hours(14);
    midnight::close_stale_for_user(&pool, user.id, next_day)
        .await
        .expect("close");

    let from = chrono::NaiveDate::from_ymd_opt(2026, 8, 21).unwrap();
    let rows = entries::list_entries(
        &pool,
        user.id,
        from,
        from,
        &entries::EntryFilters::default(),
    )
    .await
    .expect("list");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, EntryStatus::NeedsTask.as_str());
    assert!(rows[0].end_at.is_some());
}
