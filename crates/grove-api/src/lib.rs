pub mod config;
pub mod db;
pub mod error;
pub mod extract;
pub mod routes;
pub mod state;

use axum::{Router, routing::get};
use problem_details::ProblemDetails;
use state::AppState;

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(routes::health::health))
        .route(
            "/orgs/{org_id}/workspaces",
            get(routes::workspace::list).post(routes::workspace::create),
        )
        .route(
            "/orgs/{org_id}/workspaces/{ws_id}",
            get(routes::workspace::get)
                .put(routes::workspace::update)
                .delete(routes::workspace::delete),
        )
        .fallback(|| async {
            ProblemDetails::from_status_code(axum::http::StatusCode::NOT_FOUND)
                .with_detail("route not found")
        })
        .with_state(state)
}
