mod common;

use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn json_body(json: serde_json::Value) -> axum::body::Body {
    axum::body::Body::from(serde_json::to_vec(&json).unwrap())
}

/// Helper: create a workspace via HTTP and return ws_id string
async fn create_workspace(app: &axum::Router, org_id: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "guardrail-route-test-ws"
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

fn prohibition_rule() -> serde_json::Value {
    serde_json::json!({
        "type": "prohibition",
        "description": "Do not modify migration files",
        "patterns": ["migrations/"],
        "actions": ["file_modify", "file_delete"]
    })
}

#[tokio::test]
async fn guardrail_route_crud_lifecycle() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let app = grove_api::create_app(state);
    let ws_id = create_workspace(&app, &org_id).await;

    // Create guardrail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces/{ws_id}/guardrails"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "No migration edits",
                    "description": "Prevent accidental migration changes",
                    "category": "prohibition",
                    "scope": "workspace",
                    "enforcement": "enforced",
                    "rule": prohibition_rule()
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(created["name"], "No migration edits");
    assert_eq!(created["category"], "prohibition");
    assert_eq!(created["scope"], "workspace");
    assert_eq!(created["enforcement"], "enforced");
    assert_eq!(created["enabled"], true);
    let guardrail_id = created["id"].as_str().unwrap();

    // Get guardrail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/guardrails/{guardrail_id}"
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let guardrail: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(guardrail["name"], "No migration edits");

    // List guardrails
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/{org_id}/workspaces/{ws_id}/guardrails"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(list.iter().any(|g| g["id"].as_str() == Some(guardrail_id)));

    // Update guardrail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/guardrails/{guardrail_id}"
                ))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "Updated guardrail",
                    "description": "Updated description",
                    "category": "prohibition",
                    "scope": "workspace",
                    "enforcement": "advisory",
                    "rule": prohibition_rule(),
                    "version": 1,
                    "sort_order": 0,
                    "enabled": false
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["name"], "Updated guardrail");
    assert_eq!(updated["enforcement"], "advisory");
    assert_eq!(updated["enabled"], false);

    // Delete guardrail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/guardrails/{guardrail_id}"
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
                    "/orgs/{org_id}/workspaces/{ws_id}/guardrails/{guardrail_id}"
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
async fn guardrail_route_list_with_scope_filter() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let app = grove_api::create_app(state);
    let ws_id = create_workspace(&app, &org_id).await;

    // Create workspace-scoped guardrail
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces/{ws_id}/guardrails"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "Workspace rule",
                    "category": "prohibition",
                    "scope": "workspace",
                    "rule": prohibition_rule()
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    // Create session-scoped guardrail
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces/{ws_id}/guardrails"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "Session rule",
                    "category": "boundary",
                    "scope": "session",
                    "rule": {
                        "type": "boundary",
                        "description": "Stay in src",
                        "allowed_paths": ["src/"],
                        "denied_paths": []
                    }
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    // Filter by scope=workspace&enabled=true
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/guardrails?scope=workspace&enabled=true"
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "Workspace rule");

    // Filter by scope=session
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/orgs/{org_id}/workspaces/{ws_id}/guardrails?scope=session&enabled=true"
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "Session rule");

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn guardrail_route_rejects_empty_name() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();
    let app = grove_api::create_app(state);
    let ws_id = create_workspace(&app, &org_id).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces/{ws_id}/guardrails"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "  ",
                    "category": "prohibition",
                    "rule": prohibition_rule()
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    common::cleanup_org(&pool, &org_id).await;
}
