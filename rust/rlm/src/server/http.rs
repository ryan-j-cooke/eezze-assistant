
use axum::{routing::get, Json, Router};
use serde_json::json;

use crate::server::routes::register_api_routes;

pub fn create_server() -> Router {
    let app = Router::new().route("/health", get(health_handler));

    register_api_routes(app)
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

