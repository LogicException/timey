use timey_api::config::Config;
use timey_api::db;
use timey_api::error::AppResult;
use timey_api::http::router;
use timey_api::services::users;
use timey_api::state::AppState;

#[tokio::main]
async fn main() -> AppResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url).await?;
    db::migrate(&pool).await?;

    if let (Some(username), Some(password)) = (
        config.bootstrap_admin_username.clone(),
        config.bootstrap_admin_password.clone(),
    ) {
        users::bootstrap_admin(&pool, &username, &password, chrono::Utc::now()).await?;
    }

    let bind = config.bind;
    let app =
        router(AppState { pool, config }).layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(bind).await.map_err(|err| {
        timey_api::error::AppError::Internal(format!("Bind fehlgeschlagen: {err}"))
    })?;
    tracing::info!(%bind, "timey-api listening");
    axum::serve(listener, app)
        .await
        .map_err(|err| timey_api::error::AppError::Internal(format!("Serverfehler: {err}")))?;
    Ok(())
}
