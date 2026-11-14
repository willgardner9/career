use axum::{
    body::Body,
    http::{HeaderValue, Method, Request, header},
};
use sea_orm::{DatabaseBackend, MockDatabase};
use tower::ServiceExt;

const ALLOWED_ORIGIN: &str = "https://app.dearest.rsvp";

fn app() -> axum::Router {
    let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    dearest::app(db, vec![HeaderValue::from_static(ALLOWED_ORIGIN)])
}

#[tokio::test]
async fn allowed_origin_gets_cors_header() {
    let response = app()
        .oneshot(
            Request::get("/health")
                .header(header::ORIGIN, ALLOWED_ORIGIN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        ALLOWED_ORIGIN
    );
}

#[tokio::test]
async fn other_origin_gets_no_cors_header() {
    let response = app()
        .oneshot(
            Request::get("/health")
                .header(header::ORIGIN, "https://evil.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        !response
            .headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
    );
}

#[tokio::test]
async fn preflight_request_is_allowed() {
    let response = app()
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/health")
                .header(header::ORIGIN, ALLOWED_ORIGIN)
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
    assert_eq!(
        response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        ALLOWED_ORIGIN
    );
}
