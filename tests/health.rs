use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn get_health(db: sea_orm::DatabaseConnection) -> (StatusCode, Value) {
    let response = dearest::app(db, vec![])
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn health_returns_ok_when_db_is_up() {
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 0,
        }])
        .into_connection();

    let (status, body) = get_health(db).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "status": "ok", "db": "up" }));
}

#[tokio::test]
async fn health_returns_503_when_db_is_down() {
    // No mocked results: the query fails.
    let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();

    let (status, body) = get_health(db).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, json!({ "status": "degraded", "db": "down" }));
}
