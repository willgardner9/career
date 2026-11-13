pub mod config;
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use sea_orm::{ConnectionTrait, DatabaseConnection};
use serde_json::{Value, json};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

pub fn app(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(AppState { db })
        .layer(TraceLayer::new_for_http())
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
