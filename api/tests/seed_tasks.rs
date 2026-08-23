use chrono::{TimeZone, Utc};
use timey_api::db;
use timey_api::models::Role;
use timey_api::services::catalogs;
use timey_api::services::users;

#[tokio::test]
async fn default_task_seed_fills_missing_names_without_duplicates() {
    let pool = db::connect("sqlite::memory:").await.expect("db");
    db::migrate(&pool).await.expect("migrate");
    let now = Utc
        .with_ymd_and_hms(2026, 8, 21, 10, 0, 0)
        .single()
        .unwrap();
    let user = users::create_user(&pool, "admin", "password1", Role::Admin, now)
        .await
        .expect("user");

    sqlx::query("DELETE FROM tasks WHERE user_id = ? AND name = 'Feature'")
        .bind(user.id)
        .execute(&pool)
        .await
        .expect("delete");

    catalogs::seed_default_tasks_for_all_users(&pool, now)
        .await
        .expect("reseed");
    catalogs::seed_default_tasks_for_all_users(&pool, now)
        .await
        .expect("second seed");

    let tasks = catalogs::list_tasks(&pool, user.id, true)
        .await
        .expect("list");
    let names: Vec<_> = tasks.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(names.iter().filter(|name| **name == "Feature").count(), 1);
    assert_eq!(names.iter().filter(|name| **name == "Meeting").count(), 1);
    assert!(names.contains(&"Termin"));
    assert!(names.contains(&"Qualitätssicherung"));
    assert!(names.contains(&"Wartung"));
}
