use chrono::{TimeZone, Utc};
use timey_api::db;
use timey_api::models::Role;
use timey_api::services::settings;
use timey_api::services::users;

#[tokio::test]
async fn get_or_default_inserts_missing_row() {
    let pool = db::connect("sqlite::memory:").await.expect("db");
    db::migrate(&pool).await.expect("migrate");
    let now = Utc
        .with_ymd_and_hms(2026, 8, 21, 10, 0, 0)
        .single()
        .unwrap();
    let user = users::create_user(&pool, "admin", "password1", Role::Admin, now)
        .await
        .expect("user");

    sqlx::query("DELETE FROM user_settings WHERE user_id = ?")
        .bind(user.id)
        .execute(&pool)
        .await
        .expect("delete");

    let hours = settings::get_or_default(&pool, user.id)
        .await
        .expect("defaults");
    assert_eq!(hours.work_start(), "07:30");
    assert_eq!(hours.work_end(), "16:15");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_settings WHERE user_id = ?")
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count, 1);
}
