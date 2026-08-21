use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    pub session_ttl: Duration,
    pub cookie_secure: bool,
    pub cookie_name: String,
    pub bootstrap_admin_username: Option<String>,
    pub bootstrap_admin_password: Option<String>,
    pub auth_local_enabled: bool,
    pub auth_oidc_enabled: bool,
}

impl Config {
    pub fn from_env() -> AppResult<Self> {
        let bind: SocketAddr = env::var("BIND")
            .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
            .parse()
            .map_err(|_| AppError::Config("BIND ist ungültig".into()))?;

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            let path = PathBuf::from("data/timey.db");
            format!("sqlite://{}", path.display())
        });

        let session_hours: u64 = env::var("SESSION_TTL_HOURS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(12);

        let cookie_secure = env::var("COOKIE_SECURE")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        Ok(Self {
            bind,
            database_url,
            session_ttl: Duration::from_secs(session_hours.saturating_mul(3600)),
            cookie_secure,
            cookie_name: "timey_session".into(),
            bootstrap_admin_username: env::var("BOOTSTRAP_ADMIN_USERNAME").ok(),
            bootstrap_admin_password: env::var("BOOTSTRAP_ADMIN_PASSWORD").ok(),
            auth_local_enabled: env_flag("AUTH_LOCAL_ENABLED", true),
            auth_oidc_enabled: env_flag("AUTH_OIDC_ENABLED", false),
        })
    }

    pub fn for_tests(database_url: String) -> Self {
        Self {
            bind: "127.0.0.1:0".parse().expect("test bind"),
            database_url,
            session_ttl: Duration::from_secs(3600),
            cookie_secure: false,
            cookie_name: "timey_session".into(),
            bootstrap_admin_username: None,
            bootstrap_admin_password: None,
            auth_local_enabled: true,
            auth_oidc_enabled: false,
        }
    }
}

fn env_flag(name: &str, default: bool) -> bool {
    env::var(name)
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}
