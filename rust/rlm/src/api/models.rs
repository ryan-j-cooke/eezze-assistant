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

/// SSE-compatible status event for VS Code UI.
#[derive(Serialize)]
pub struct StatusEvent {
    #[serde(rename = "type")]
    pub event_type: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<&'static str>,
}

/// Helper to create a status SSE chunk (consistent with chat endpoint format).
pub fn make_status_sse(message: String, phase: Option<&'static str>, step: Option<&'static str>) -> String {
    let ev = StatusEvent {
        event_type: "status",
        message,
        phase,
        step,
    };
    format!("data: {}\n\n", serde_json::to_string(&ev).unwrap_or_default())
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