// here
use axum::{
    body::Body,
    http::{HeaderValue, StatusCode},
    response::Response,
    routing::post,
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::llm::ollama::ollama_chat;
use crate::llm::types::{LLMChatRequest, LLMChatResponse, LLMProvider};
use crate::orchestrator::escalate::EscalationPolicy;
use crate::orchestrator::r#loop::{run_recursive_session, RecursiveOptions};
use crate::types::chat::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ChatRole};
use crate::types::config::{ModelConfig, ModelProvider};
use crate::utils::{logger, timer};
use crate::utils::stream::format_sse_event;
use crate::utils::lang::classify_text;
use crate::api::models::make_status_sse;

use reqwest::Client;
use std::sync::{Arc, Mutex};

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

async fn handle_chat_completion(body: ChatCompletionRequest) -> anyhow::Result<Vec<String>> {
    let prompt = build_prompt_from_messages(&body.messages);

    // Detect language only for the initial prompt (first user message)
    let language = if let Some(first_user_message) = body.messages.iter().find(|m| matches!(m.role, ChatRole::User)) {
        classify_text(&first_user_message.content).await
    } else {
        "en".to_string()
    };

    logger::info(
        "chat.completion.language_detected",
        Some(json!({ "language": language })),
    );

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

    // Load phase-specific models from user-level eezze config, falling back to defaults.
    let ee_cfg = crate::eezze_config::load_config().unwrap_or_default();

    // Main answering model: honour the client-requested model string.
    let initial_model = ModelConfig {
        name: ollama_model_name.clone(),
        provider: ModelProvider::Ollama,
        temperature: body.temperature,
        max_tokens: body.max_tokens,
    };

    // Planning model: smaller/faster model.
    let planning_model = ModelConfig {
        name: ee_cfg.expert_fast_model.clone(),
        provider: ModelProvider::Ollama,
        temperature: body.temperature,
        max_tokens: body.max_tokens,
    };

    // Verifier & revision model: small reviewer model.
    let verifier_model = ModelConfig {
        name: ee_cfg.expert_reviewer_model.clone(),
        provider: ModelProvider::Ollama,
        temperature: Some(0.0),
        max_tokens: Some(256),
    };

    let revision_model = verifier_model.clone();

    let provider = OllamaProvider {
        client: Client::new(),
    };

    let escalation_policy = EscalationPolicy {
        max_attempts: 2,
        ladder: vec![initial_model.clone()],
    };

    // Buffer to collect status SSE chunks
    let status_chunks: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let status_chunks_clone = Arc::clone(&status_chunks);

    let result = run_recursive_session(
        &prompt,
        &[],
        RecursiveOptions {
            provider: &provider,
            planning_model: planning_model.clone(),
            initial_model: initial_model.clone(),
            verifier_model: verifier_model.clone(),
            revision_model: revision_model.clone(),
            escalation_policy,
            max_retries: Some(2),
            min_confidence: Some(0.75),
            on_status: Some(Box::new(move |chunk: String| {
                if let Ok(mut guard) = status_chunks_clone.lock() {
                    guard.push(chunk);
                }
            })),
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

    // Build the final OpenAI-compatible response
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

    // Return buffered status chunks followed by the final response chunks
    let mut out = Vec::new();
    if let Ok(guard) = status_chunks.lock() {
        out.extend(guard.iter().cloned());
    }
    // Convert final response into two SSE chunks (role + content)
    let id = &api_response.id;
    let model = &api_response.model;
    let choice = api_response.choices.first().unwrap();
    out.push(crate::utils::stream::make_chat_chunk(id, model, None, None)); // role-only (delta empty)
    out.push(crate::utils::stream::make_chat_chunk(id, model, Some(choice.message.content.clone()), Some("stop")));
    out.push("data: [DONE]\n\n".to_string());
    Ok(out)
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
) -> Response {
    // Mirror TypeScript behavior: stream SSE chunks instead of plain JSON.
    if body.messages.is_empty() {
        let mut res = Response::new(Body::from(
            format_sse_event(&json!({
                "error": {
                    "message": "Invalid request: messages are required",
                    "type": "invalid_request_error",
                }
            })),
        ));
        *res.status_mut() = StatusCode::BAD_REQUEST;
        let headers = res.headers_mut();
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("text/event-stream"),
        );
        headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));
        return res;
    }

    // Emit an initial "starting" status so VS Code always gets a valid SSE stream
    let mut chunks = vec![];
    make_status_sse("Starting request...".to_string(), Some("init"), None, |sse| {
        chunks.push(sse);
    });

    let (result, duration_ms) = timer::measure(|| handle_chat_completion(body)).await;

    match result {
        Ok(mut more_chunks) => {
            logger::info(
                "chat.completion.success",
                Some(json!({ "latencyMs": duration_ms })),
            );
            chunks.append(&mut more_chunks);
        }
        Err(err) => {
            logger::error(
                "chat.completion.failed",
                Some(json!({ "error": err.to_string() })),
            );
            // Emit a fallback chat response so VS Code gets a valid OpenAI shape
            let fallback_id = format!("chatcmpl-{}", Uuid::new_v4());
            chunks.push(crate::utils::stream::make_chat_chunk(&fallback_id, "fallback", None, None)); // role-only
            chunks.push(crate::utils::stream::make_chat_chunk(&fallback_id, "fallback", Some("I’m sorry, I couldn’t process that request.".to_string()), Some("stop")));
        }
    }

    // Always terminate with [DONE] so VS Code knows the stream ended
    chunks.push("data: [DONE]\n\n".to_string());

    let mut body_str = String::new();
    for chunk in chunks {
        body_str.push_str(&chunk);
    }

    let mut res = Response::new(Body::from(body_str));
    let headers = res.headers_mut();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/event-stream"),
    );
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    res
}