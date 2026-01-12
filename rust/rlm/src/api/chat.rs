// here
use axum::{http::StatusCode, routing::post, Json, Router};
use serde_json::json;
use uuid::Uuid;

use crate::llm::ollama::ollama_chat;
use crate::llm::types::{LLMChatRequest, LLMChatResponse, LLMProvider};
use crate::orchestrator::escalate::EscalationPolicy;
use crate::orchestrator::r#loop::{run_orchestrator, LoopOptions};
use crate::types::chat::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ChatRole};
use crate::types::config::{ModelConfig, ModelProvider};
use crate::utils::{logger, timer};

use reqwest::Client;

struct OllamaProvider {
    client: Client,
}

#[async_trait::async_trait]
impl LLMProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn chat(&self, request: LLMChatRequest) -> anyhow::Result<LLMChatResponse> {
        let content = ollama_chat(&self.client, &request.model, &request.messages, request.stream).await?;
        Ok(LLMChatResponse {
            content,
            model: request.model.name.clone(),
            finish_reason: Some("stop".to_string()),
        })
    }
}

async fn handle_chat_completion(body: ChatCompletionRequest) -> anyhow::Result<ChatCompletionResponse> {
    let prompt = build_prompt_from_messages(&body.messages);

    let raw_model = body.model.clone();
    let client_model_id = raw_model.clone();
    let ollama_model_name = raw_model.clone();

    logger::info(
        "chat.completion.handle.start",
        Some(json!({
            "model": raw_model,
            "clientModelId": client_model_id,
            "ollamaModelName": ollama_model_name,
            "messages": body.messages.len(),
            "temperature": body.temperature,
            "maxTokens": body.max_tokens,
        })),
    );

    let initial_model = ModelConfig {
        name: ollama_model_name.clone(),
        provider: ModelProvider::Ollama,
        temperature: body.temperature,
        max_tokens: body.max_tokens,
    };

    let verifier_model = ModelConfig {
        name: ollama_model_name.clone(),
        provider: ModelProvider::Ollama,
        temperature: Some(0.0),
        max_tokens: Some(256),
    };

    let provider = OllamaProvider {
        client: Client::new(),
    };

    let escalation_policy = EscalationPolicy {
        max_attempts: 2,
        ladder: vec![initial_model.clone()],
    };

    let result = run_orchestrator(
        &prompt,
        &[],
        LoopOptions {
            provider: &provider,
            initial_model: initial_model.clone(),
            verifier_model: verifier_model.clone(),
            escalation_policy,
            max_retries: Some(2),
            min_confidence: Some(0.75),
        },
    )
    .await?;

    logger::info(
        "chat.completion.handle.result",
        Some(json!({
            "requestedModel": raw_model,
            "clientModelId": client_model_id,
            "ollamaModelName": ollama_model_name,
            "model": result.model,
            "attempts": result.attempts,
            "confidence": result.confidence,
        })),
    );

    let api_response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: (chrono::Utc::now().timestamp()) as i64,
        model: raw_model,
        choices: vec![crate::types::chat::ChatCompletionChoice {
            index: 0,
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: result.content,
            },
            finish_reason: crate::types::chat::FinishReason::Stop,
        }],
    };

    Ok(api_response)
}

fn build_prompt_from_messages(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .map(|m| format!("{}: {}", format_role(&m.role), m.content))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_role(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::System => "SYSTEM",
        ChatRole::User => "USER",
        ChatRole::Assistant => "ASSISTANT",
    }
}

pub fn register_chat_routes(app: Router) -> Router {
    app.route("/v1/chat/completions", post(chat_completions_handler))
}

async fn chat_completions_handler(
    Json(body): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, (StatusCode, String)> {
    if body.messages.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid request: messages are required".to_string(),
        ));
    }

    let (result, duration_ms) = timer::measure(|| handle_chat_completion(body)).await;

    match result {
        Ok(response) => {
            logger::info(
                "chat.completion.success",
                Some(json!({ "latencyMs": duration_ms })),
            );
            Ok(Json(response))
        }
        Err(err) => {
            logger::error("chat.completion.failed", Some(json!({ "error": err.to_string() })));
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ))
        }
    }
}