use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Config(String),
    #[error("nicht angemeldet")]
    Unauthorized,
    #[error("keine Berechtigung")]
    Forbidden,
    #[error("nicht gefunden")]
    NotFound,
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Unprocessable(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("Datenbankfehler")]
    Database(#[from] sqlx::Error),
    #[error("interner Fehler")]
    Internal(String),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Config(_) | Self::Internal(_) | Self::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Unprocessable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if matches!(
            self,
            Self::Database(_) | Self::Internal(_) | Self::Config(_)
        ) {
            tracing::error!(error = %self, "request failed");
        }
        let body = Json(ErrorBody {
            error: self.to_string(),
        });
        (self.status(), body).into_response()
    }
}
