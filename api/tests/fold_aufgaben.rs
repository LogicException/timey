use sqlx::SqlitePool;
use timey_api::db;

const INIT_SQL: &str = include_str!("../migrations/0001_init.sql");
const SETTINGS_SQL: &str = include_str!("../migrations/0002_user_settings.sql");
const FOLD_SQL: &str = include_str!("../migrations/0003_drop_aufgaben.sql");

async fn exec_script(pool: &SqlitePool, sql: &str) {
    for raw in sql.split(';') {
        let code: String = raw
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");
        if code.is_empty() {
            continue;
        }
        sqlx::query(&code)
            .execute(pool)
            .await
            .unwrap_or_else(|err| panic!("SQL failed for `{code}`: {err}"));
    }
}

async fn table_exists(pool: &SqlitePool, name: &str) -> bool {
    let found: Option<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(name)
            .fetch_optional(pool)
            .await
            .expect("sqlite_master");
    found.is_some()
}

async fn column_exists(pool: &SqlitePool, table: &str, column: &str) -> bool {
    let count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = ?"
    ))
    .bind(column)
    .fetch_one(pool)
    .await
    .expect("pragma_table_info");
    count > 0
}

async fn insert_named(pool: &SqlitePool, table: &str, user_id: i64, name: &str) -> i64 {
    let sql = format!(
        "INSERT INTO {table} (user_id, name, archived, created_at) VALUES (?, ?, 0, '2026-08-21T10:00:00Z')"
    );
    sqlx::query(&sql)
        .bind(user_id)
        .bind(name)
        .execute(pool)
        .await
        .expect("insert named")
        .last_insert_rowid()
}

#[tokio::test]
async fn fold_concatenates_used_pairs_and_drops_aufgaben() {
    let pool = db::connect("sqlite::memory:").await.expect("db");
    exec_script(&pool, INIT_SQL).await;
    exec_script(&pool, SETTINGS_SQL).await;

    sqlx::query(
        "INSERT INTO users (username, password_hash, auth_provider, role, disabled, created_at)
         VALUES ('anna', NULL, 'local', 'user', 0, '2026-08-21T10:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("user");
    let user_id = 1_i64;

    let email = insert_named(&pool, "tasks", user_id, "E-Mail").await;
    let meeting = insert_named(&pool, "tasks", user_id, "Meeting").await;
    let existing_combo = insert_named(&pool, "tasks", user_id, "E-Mail schreiben").await;
    let schreiben = insert_named(&pool, "aufgaben", user_id, "schreiben").await;
    let lesen = insert_named(&pool, "aufgaben", user_id, "lesen").await;
    insert_named(&pool, "aufgaben", user_id, "ungenutzt").await;

    sqlx::query(
        "INSERT INTO entries (user_id, task_id, project_id, aufgabe_id, start_at, end_at, status, created_at)
         VALUES
           (?, ?, NULL, ?, '2026-08-21T08:00:00Z', '2026-08-21T09:00:00Z', 'complete', '2026-08-21T10:00:00Z'),
           (?, ?, NULL, NULL, '2026-08-21T09:00:00Z', '2026-08-21T10:00:00Z', 'complete', '2026-08-21T10:00:00Z'),
           (?, NULL, NULL, ?, '2026-08-21T10:00:00Z', '2026-08-21T11:00:00Z', 'needs_task', '2026-08-21T10:00:00Z'),
           (?, ?, NULL, ?, '2026-08-21T11:00:00Z', '2026-08-21T12:00:00Z', 'complete', '2026-08-21T10:00:00Z')",
    )
    .bind(user_id)
    .bind(email)
    .bind(schreiben)
    .bind(user_id)
    .bind(meeting)
    .bind(user_id)
    .bind(lesen)
    .bind(user_id)
    .bind(email)
    .bind(lesen)
    .execute(&pool)
    .await
    .expect("entries");

    exec_script(&pool, FOLD_SQL).await;

    assert!(
        !table_exists(&pool, "aufgaben").await,
        "aufgaben table should be gone"
    );
    assert!(
        !column_exists(&pool, "entries", "aufgabe_id").await,
        "entries.aufgabe_id should be gone"
    );

    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM tasks WHERE user_id = ? ORDER BY name COLLATE NOCASE")
            .bind(user_id)
            .fetch_all(&pool)
            .await
            .expect("tasks");
    assert!(names.contains(&"E-Mail".to_string()));
    assert!(names.contains(&"Meeting".to_string()));
    assert!(names.contains(&"E-Mail schreiben".to_string()));
    assert!(names.contains(&"E-Mail lesen".to_string()));
    assert!(!names.iter().any(|name| name == "ungenutzt"));
    assert!(!names.iter().any(|name| name == "schreiben"));
    assert_eq!(
        names
            .iter()
            .filter(|name| *name == "E-Mail schreiben")
            .count(),
        1
    );

    let pair_task: i64 = sqlx::query_scalar(
        "SELECT e.task_id FROM entries e WHERE e.start_at = '2026-08-21T08:00:00Z'",
    )
    .fetch_one(&pool)
    .await
    .expect("pair entry");
    assert_eq!(pair_task, existing_combo);

    let meeting_task: i64 = sqlx::query_scalar(
        "SELECT e.task_id FROM entries e WHERE e.start_at = '2026-08-21T09:00:00Z'",
    )
    .fetch_one(&pool)
    .await
    .expect("meeting entry");
    assert_eq!(meeting_task, meeting);

    let orphan_task: Option<i64> = sqlx::query_scalar(
        "SELECT e.task_id FROM entries e WHERE e.start_at = '2026-08-21T10:00:00Z'",
    )
    .fetch_one(&pool)
    .await
    .expect("orphan entry");
    assert_eq!(orphan_task, None);

    let lesen_name: String = sqlx::query_scalar(
        "SELECT t.name FROM entries e JOIN tasks t ON t.id = e.task_id
         WHERE e.start_at = '2026-08-21T11:00:00Z'",
    )
    .fetch_one(&pool)
    .await
    .expect("lesen entry");
    assert_eq!(lesen_name, "E-Mail lesen");
}

#[tokio::test]
async fn migrate_on_fresh_db_has_no_aufgaben() {
    let pool = db::connect("sqlite::memory:").await.expect("db");
    db::migrate(&pool).await.expect("migrate");
    assert!(!table_exists(&pool, "aufgaben").await);
    assert!(!column_exists(&pool, "entries", "aufgabe_id").await);
}
