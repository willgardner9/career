pub mod config;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, Method, StatusCode, header},
    routing::get,
};
use sea_orm::{ConnectionTrait, DatabaseConnection};
use serde_json::{Value, json};
use std::time::Duration;

use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

/// Build the application router.
/// Build the router. Only a browser on one of `cors_allowed_origins` may call the API.
pub fn app(db: DatabaseConnection, cors_allowed_origins: Vec<HeaderValue>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(cors_allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .max_age(Duration::from_secs(3600));

    Router::new()
        .route("/health", get(health))
        .with_state(AppState { db })
        .layer(TraceLayer::new_for_http())
        // CORS is the outer layer, so it answers a preflight request first.
        .layer(cors)
}

async fn health(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    match state.db.execute_unprepared("SELECT 1").await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "ok", "db": "up" }))),
        Err(err) => {
            tracing::error!("health check db error: {err}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "degraded", "db": "down" })),
            )
        }
    }
}
