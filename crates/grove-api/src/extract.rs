//! Custom extractors that route Axum rejections through ApiError
//! for RFC 9457 Problem Details compliance.

use axum::extract::FromRequest;
use serde::de::DeserializeOwned;

use crate::error::ApiError;

/// A `Json` extractor that maps rejections to `ApiError`.
///
/// Drop-in replacement for `axum::Json` — same usage, but deserialization
/// failures produce `application/problem+json` instead of `text/plain`.
pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let result = axum::Json::<T>::from_request(req, state).await;
        match result {
            Ok(axum::Json(value)) => Ok(Json(value)),
            Err(rejection) => Err(ApiError::JsonPayload(rejection)),
        }
    }
}

// Re-export for response serialization
impl<T: serde::Serialize> axum::response::IntoResponse for Json<T> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}

/// A `Query` extractor that maps rejections to `ApiError`.
///
/// Drop-in replacement for `axum::extract::Query` — same usage, but
/// deserialization failures produce `application/problem+json`.
pub struct Query<T>(pub T);

impl<T, S> axum::extract::FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let result = axum::extract::Query::<T>::from_request_parts(parts, state).await;
        match result {
            Ok(axum::extract::Query(value)) => Ok(Query(value)),
            Err(rejection) => Err(ApiError::QueryParam(rejection)),
        }
    }
}

/// A `Path` extractor that maps rejections to `ApiError`.
pub struct Path<T>(pub T);

impl<T, S> axum::extract::FromRequestParts<S> for Path<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let result = axum::extract::Path::<T>::from_request_parts(parts, state).await;
        match result {
            Ok(axum::extract::Path(value)) => Ok(Path(value)),
            Err(rejection) => Err(ApiError::PathParam(rejection)),
        }
    }
}
