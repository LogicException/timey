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

    let tasks = catalogs::list_tasks(&pool, user.id, true, false)
        .await
        .expect("list");
    let names: Vec<_> = tasks.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(names.iter().filter(|name| **name == "Feature").count(), 1);
    assert_eq!(names.iter().filter(|name| **name == "Meeting").count(), 1);
    assert!(names.contains(&"Termin"));
    assert!(names.contains(&"Qualitätssicherung"));
    assert!(names.contains(&"Wartung"));
    assert!(!names.contains(&"unbestimmt"));
}

#[tokio::test]
async fn seed_creates_exactly_one_unbestimmt_system_task() {
    let pool = db::connect("sqlite::memory:").await.expect("db");
    db::migrate(&pool).await.expect("migrate");
    let now = Utc
        .with_ymd_and_hms(2026, 8, 21, 10, 0, 0)
        .single()
        .unwrap();
    let user = users::create_user(&pool, "admin", "password1", Role::Admin, now)
        .await
        .expect("user");

    catalogs::seed_default_tasks_for_all_users(&pool, now)
        .await
        .expect("reseed");
    catalogs::seed_default_tasks_for_all_users(&pool, now)
        .await
        .expect("second seed");

    let hidden = catalogs::list_tasks(&pool, user.id, true, false)
        .await
        .expect("hidden");
    assert!(
        hidden
            .iter()
            .all(|row| !row.is_system && row.name != "unbestimmt"),
        "system task must stay hidden without include_system"
    );

    let visible = catalogs::list_tasks(&pool, user.id, true, true)
        .await
        .expect("visible");
    let system: Vec<_> = visible.iter().filter(|row| row.is_system).collect();
    assert_eq!(system.len(), 1);
    assert_eq!(system[0].name, "unbestimmt");
    assert!(!system[0].archived);
}

#[tokio::test]
async fn seed_promotes_existing_unbestimmt_name_to_system() {
    let pool = db::connect("sqlite::memory:").await.expect("db");
    db::migrate(&pool).await.expect("migrate");
    let now = Utc
        .with_ymd_and_hms(2026, 8, 21, 10, 0, 0)
        .single()
        .unwrap();
    let user = users::create_user(&pool, "admin", "password1", Role::Admin, now)
        .await
        .expect("user");

    sqlx::query("DELETE FROM tasks WHERE user_id = ? AND is_system = 1")
        .bind(user.id)
        .execute(&pool)
        .await
        .expect("delete system");
    sqlx::query(
        "INSERT INTO tasks (user_id, name, archived, is_system, created_at) VALUES (?, 'Unbestimmt', 0, 0, ?)",
    )
    .bind(user.id)
    .bind(now.to_rfc3339())
    .execute(&pool)
    .await
    .expect("user-named unbestimmt");

    catalogs::seed_default_tasks(&pool, user.id, now)
        .await
        .expect("seed");

    let visible = catalogs::list_tasks(&pool, user.id, true, true)
        .await
        .expect("visible");
    let system: Vec<_> = visible.iter().filter(|row| row.is_system).collect();
    assert_eq!(system.len(), 1);
    assert!(system[0].name.eq_ignore_ascii_case("unbestimmt"));
}
