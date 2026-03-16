use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use grove_domain::error::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    Domain(#[from] DomainError),

    #[error("{0}")]
    Database(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("{0}")]
    Internal(String),
}

impl ApiError {
    pub fn internal(e: impl std::fmt::Display) -> Self {
        Self::Internal(e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, detail) = match &self {
            Self::Domain(DomainError::NotFound { entity, id }) => (
                StatusCode::NOT_FOUND,
                format!("{entity} {id} not found"),
            ),
            Self::Domain(DomainError::Validation(msg)) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                msg.clone(),
            ),
            Self::Domain(DomainError::Conflict(msg)) => (StatusCode::CONFLICT, msg.clone()),
            Self::Domain(DomainError::Unauthorized(_)) => {
                (StatusCode::FORBIDDEN, "forbidden".into())
            }
            Self::Domain(DomainError::Internal(_)) | Self::Database(_) | Self::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".into())
            }
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
        };

        let body = serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail,
        });

        let mut response = (status, axum::Json(body)).into_response();
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}
