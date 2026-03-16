mod common;

use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn json_body(json: serde_json::Value) -> axum::body::Body {
    axum::body::Body::from(serde_json::to_vec(&json).unwrap())
}

#[tokio::test]
async fn create_and_list_workspaces() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_id = common::unique_org_id();

    let app = grove_api::create_app(state);

    // Create
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_id}/workspaces"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({
                    "name": "Test Workspace",
                    "description": "For testing"
                })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(created["name"], "Test Workspace");
    assert_eq!(created["org_id"], org_id);
    let ws_id = created["id"].as_str().unwrap();

    // List
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/{org_id}/workspaces"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(list.iter().any(|w| w["id"].as_str() == Some(ws_id)));

    // Cleanup
    common::cleanup_org(&pool, &org_id).await;
}

#[tokio::test]
async fn get_nonexistent_workspace_returns_404() {
    let state = common::test_state().await;
    let app = grove_api::create_app(state);
    let fake_id = uuid::Uuid::now_v7();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/no_org/workspaces/{fake_id}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_workspace_rejects_empty_name() {
    let state = common::test_state().await;
    let app = grove_api::create_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/orgs/test_org/workspaces")
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({ "name": "  " })))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn org_isolation_workspaces_not_visible_across_orgs() {
    let state = common::test_state().await;
    let pool = state.pool.clone();
    let org_a = common::unique_org_id();
    let org_b = common::unique_org_id();

    let app = grove_api::create_app(state);

    // Create in org A
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/orgs/{org_a}/workspaces"))
                .header("content-type", "application/json")
                .body(json_body(serde_json::json!({ "name": "A's workspace" })))
                .unwrap(),
        )
        .await
        .unwrap();

    // List from org B — should see nothing
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/{org_b}/workspaces"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(list.is_empty(), "org B should not see org A's workspaces");

    common::cleanup_org(&pool, &org_a).await;
    common::cleanup_org(&pool, &org_b).await;
}
