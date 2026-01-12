// here
use crate::index::embed::handle_embeddings;
use crate::llm::ollama::ollama_chat;
use crate::llm::types::{LLMChatRequest, LLMProvider, LLMChatResponse};
use crate::orchestrator::confidence::combine_confidence;
use crate::orchestrator::r#loop::{run_orchestrator, LoopOptions};
use crate::types::config::{ModelConfig, ModelProvider};
use crate::types::chat::{ChatMessage, ChatRole};
use crate::utils::logger;

use reqwest::Client;

pub struct ReviewResult {
    pub approved: bool,
    pub confidence: f64,
    pub notes: Option<String>,
}

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

/// Minimal chat wrapper for a single request.
pub async fn chat(
    model: ModelConfig,
    messages: Vec<ChatMessage>,
) -> anyhow::Result<String> {
    let provider = OllamaProvider {
        client: Client::new(),
    };

    let last_message_content = messages
        .last()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let context: Vec<String> = messages.iter().map(|m| m.content.clone()).collect();

    let result = run_orchestrator(
        &last_message_content,
        &context,
        LoopOptions {
            provider: &provider,
            initial_model: model.clone(),
            verifier_model: model.clone(),
            escalation_policy: crate::orchestrator::escalate::EscalationPolicy {
                max_attempts: 1,
                ladder: vec![model.clone()],
            },
            max_retries: Some(1),
            min_confidence: Some(0.75),
        },
    )
    .await?;

    Ok(result.content)
}

/// Embedding-based semantic verification.
pub async fn review_response(
    prompt: &str,
    response: &str,
    context: &[String],
) -> anyhow::Result<ReviewResult> {
    let client = Client::new();

    let response_embedding = handle_embeddings(&client, response, "nomic-embed-text", true).await?;

    let mut max_score = 0.0_f32;

    for chunk in context {
        let context_embedding = handle_embeddings(&client, chunk, "nomic-embed-text", true).await?;

        let score = cosine_similarity(&response_embedding.vector, &context_embedding.vector);
        if score > max_score {
            max_score = score;
        }
    }

    let confidence = combine_confidence(crate::orchestrator::confidence::ConfidenceInputs {
        model_confidence: None,
        verifier_confidence: Some(max_score as f64),
        embedding_score: None,
    });

    Ok(ReviewResult {
        approved: max_score >= 0.75,
        confidence,
        notes: None,
    })
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;

    let len = a.len().min(b.len());
    for i in 0..len {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a.sqrt() * norm_b.sqrt())
}