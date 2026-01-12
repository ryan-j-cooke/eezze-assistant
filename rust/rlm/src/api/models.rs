// here
use axum::{routing::get, Json, Router};
use serde::Serialize;

const EXPERT_RECURSIVE_LOCAL: &str = "qwen2.5:3b";

#[derive(Serialize)]
struct ModelInfo {
    id: String,
    object: String,
    created: i64,
    owned_by: String,
}

#[derive(Serialize)]
struct ModelsResponse {
    object: String,
    data: Vec<ModelInfo>,
}

async fn list_models() -> Json<ModelsResponse> {
    Json(ModelsResponse {
        object: "list".to_string(),
        data: vec![ModelInfo {
            id: EXPERT_RECURSIVE_LOCAL.to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "local".to_string(),
        }],
    })
}

pub fn register_model_routes(app: Router) -> Router {
    app.route("/v1/models", get(list_models))
}