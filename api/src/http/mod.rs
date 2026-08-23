mod admin;
mod auth;
mod catalogs;
mod entries;
mod extractors;
mod health;
mod settings;
mod work;

use axum::Router;
use axum::routing::{get, patch, post};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/health/db", get(health::health_db))
        .route("/api/auth/config", get(auth::config))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route(
            "/api/settings",
            get(settings::get_settings).patch(settings::patch_settings),
        )
        .route(
            "/api/admin/users",
            get(admin::list_users).post(admin::create_user),
        )
        .route("/api/admin/users/{id}", patch(admin::update_user))
        .route(
            "/api/projects",
            get(catalogs::list_projects).post(catalogs::create_project),
        )
        .route("/api/projects/{id}", patch(catalogs::patch_project))
        .route(
            "/api/tasks",
            get(catalogs::list_tasks).post(catalogs::create_task),
        )
        .route(
            "/api/tasks/{id}",
            patch(catalogs::patch_task).delete(catalogs::delete_task),
        )
        .route(
            "/api/entries",
            get(entries::list_entries).post(entries::create_entry),
        )
        .route("/api/entries/export.csv", get(entries::export_csv))
        .route("/api/entries/timer", get(entries::get_timer))
        .route("/api/entries/timer/start", post(entries::start_timer))
        .route("/api/entries/timer/stop", post(entries::stop_timer))
        .route(
            "/api/entries/{id}",
            patch(entries::update_entry).delete(entries::delete_entry),
        )
        .route("/api/work-sessions/current", get(work::current))
        .route("/api/work-sessions/start", post(work::start))
        .route("/api/work-sessions/pause", post(work::pause))
        .route("/api/work-sessions/resume", post(work::resume))
        .route("/api/work-sessions/stop", post(work::stop))
        .with_state(state)
}
