mod common;

use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn json_body(json: serde_json::Value) -> axum::body::Body {
    axum::body::Body::from(serde_json::to_vec(&json).unwrap())
}

/// Helper: create a workspace via HTTP and return (org_id, ws_id)
async fn create_workspace(app: &axum::Router, org_id: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "agent-route-test-ws"
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    created["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn agent_route_crud_lifecycle() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let app = grove_api::create_app(state);
    let ws_id = create_workspace(&app, &org_id).await;

    // Create agent
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces/{ws_id}/agents"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "Claude Code",
                    "provider": "anthropic",
                    "model": "claude-opus-4-20250514",
                    "description": "Code generation agent",
                    "capabilities": ["code_generation", "code_review"],
                    "config": {"max_tokens": 8192}
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(created["name"], "Claude Code");
    assert_eq!(created["provider"], "anthropic");
    assert_eq!(created["status"], "active");
    let agent_id = created["id"].as_str().unwrap();

    // Get agent
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/agents/{agent_id}"
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let agent: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(agent["name"], "Claude Code");

    // List agents
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/{org_id}/workspaces/{ws_id}/agents"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(list.iter().any(|a| a["id"].as_str() == Some(agent_id)));

    // Update agent
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/agents/{agent_id}"
                ))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "Updated Agent",
                    "provider": "anthropic",
                    "status": "disabled"
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["name"], "Updated Agent");
    assert_eq!(updated["status"], "disabled");

    // Delete agent
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/agents/{agent_id}"
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/agents/{agent_id}"
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn agent_route_rejects_empty_name() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let app = grove_api::create_app(state);
    let ws_id = create_workspace(&app, &org_id).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces/{ws_id}/agents"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "  ",
                    "provider": "anthropic"
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn agent_route_returns_404_for_nonexistent_workspace() {
    let state = common::test_state().await;
    let app = grove_api::create_app(state);
    let fake_ws = uuid::Uuid::now_v7();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/no_org/workspaces/{fake_ws}/agents"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
