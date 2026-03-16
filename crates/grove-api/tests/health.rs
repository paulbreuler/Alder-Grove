mod common;

use axum::http::StatusCode;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let state = common::test_state().await;
    let app = grove_api::create_app(state);

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}
