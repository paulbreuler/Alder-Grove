use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use grove_domain::error::DomainError;
use problem_details::ProblemDetails;

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

    #[error("{0}")]
    JsonPayload(#[from] JsonRejection),

    #[error("{0}")]
    PathParam(#[from] PathRejection),
}

impl ApiError {
    pub fn internal(e: impl std::fmt::Display) -> Self {
        Self::Internal(e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, detail) = match &self {
            Self::Domain(DomainError::NotFound { entity, id }) => {
                (StatusCode::NOT_FOUND, format!("{entity} {id} not found"))
            }
            Self::Domain(DomainError::Validation(msg)) => {
                (StatusCode::UNPROCESSABLE_ENTITY, msg.clone())
            }
            Self::Domain(DomainError::Conflict(msg)) => (StatusCode::CONFLICT, msg.clone()),
            // Domain "Unauthorized" = failed authorization (authenticated but
            // not permitted). HTTP 401 is handled by ApiError::Unauthorized
            // (missing/invalid JWT — added when Clerk auth lands).
            Self::Domain(DomainError::Unauthorized(msg)) => (StatusCode::FORBIDDEN, msg.clone()),
            Self::Domain(DomainError::Internal(msg))
            | Self::Database(msg)
            | Self::Internal(msg) => {
                tracing::error!(error = %msg, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            Self::JsonPayload(rejection) => (rejection.status(), rejection.body_text()),
            Self::PathParam(rejection) => (rejection.status(), rejection.body_text()),
        };

        ProblemDetails::from_status_code(status)
            .with_detail(detail)
            .into_response()
    }
}
