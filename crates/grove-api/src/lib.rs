pub mod config;
pub mod db;
pub mod error;
pub mod extract;
pub mod routes;
pub mod state;

use axum::{routing::get, Router};
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
        .with_state(state)
}
