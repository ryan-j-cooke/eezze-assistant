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

/// Helper to create and emit a status message as a thinking chunk (so VS Code can display it separately from final content).
/// The `emit` callback is called with the SSE string so the caller can stream it immediately.
pub fn make_status_sse<F>(message: String, _phase: Option<&'static str>, _step: Option<&'static str>, emit: F)
where
    F: FnOnce(String),
{
    // Use a temporary ID for thinking chunks; real chat chunks will use the final ID later
    let temp_id = "status-temp";
    let sse = crate::utils::stream::make_thinking_chunk(temp_id, "status", message);
    emit(sse);
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