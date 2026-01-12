// here
use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::index::embed::handle_embeddings;
use crate::types::openai::{OpenAIEmbedding, OpenAIEmbeddingResponse};

#[derive(Deserialize)]
struct EmbeddingBody {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    input: serde_json::Value,
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: crate::types::openai::OpenAIError,
}

async fn embeddings_handler(Json(body): Json<EmbeddingBody>) -> Result<Json<OpenAIEmbeddingResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let inputs: Vec<String> = match &body.input {
        serde_json::Value::String(s) => vec![s.clone()],
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    };

    if inputs.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorEnvelope {
                error: crate::types::openai::OpenAIError {
                    message: "Invalid request: input is required".to_string(),
                    r#type: Some("invalid_request_error".to_string()),
                    code: None,
                },
            }),
        ));
    }

    let client = reqwest::Client::new();
    let model_name = body.model.unwrap_or_else(|| "nomic-embed-text".to_string());

    let mut data: Vec<OpenAIEmbedding> = Vec::new();

    for (index, text) in inputs.iter().enumerate() {
        let result = handle_embeddings(&client, text, &model_name, true)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorEnvelope {
                        error: crate::types::openai::OpenAIError {
                            message: err.to_string(),
                            r#type: Some("internal_error".to_string()),
                            code: None,
                        },
                    }),
                )
            })?;

        data.push(OpenAIEmbedding {
            object: "embedding".to_string(),
            embedding: result.vector,
            index,
        });
    }

    let response = OpenAIEmbeddingResponse {
        object: "list".to_string(),
        model: model_name,
        data,
    };

    Ok(Json(response))
}

pub fn register_embedding_routes(app: Router) -> Router {
    app.route("/v1/embeddings", post(embeddings_handler))
}