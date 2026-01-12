use crate::llm::types::{LLMChatRequest, LLMProvider};
use crate::llm::verifier::{verify_with_llm, VerificationRequest};
use crate::orchestrator::confidence::{combine_confidence, is_confidence_acceptable, should_escalate};
use crate::orchestrator::escalate::{escalate_model, EscalationPolicy, EscalationState};
use crate::types::chat::{ChatMessage, ChatRole};
use crate::types::config::ModelConfig;
use crate::utils::logger;

pub struct LoopOptions<'a, P: LLMProvider + ?Sized> {
    pub provider: &'a P,
    pub initial_model: ModelConfig,
    pub verifier_model: ModelConfig,
    pub escalation_policy: EscalationPolicy,
    pub max_retries: Option<u32>,
    pub min_confidence: Option<f64>,
}

pub struct LoopResult {
    pub content: String,
    pub model: String,
    pub confidence: f64,
    pub attempts: u32,
}

pub async fn run_orchestrator<P: LLMProvider + ?Sized>(
    prompt: &str,
    context: &[String],
    options: LoopOptions<'_, P>,
) -> anyhow::Result<LoopResult> {
    let mut state = EscalationState {
        current_model: options.initial_model.clone(),
        attempts: 0,
    };

    let max_retries = options.max_retries.unwrap_or(2);
    let min_confidence = options.min_confidence.unwrap_or(0.75);

    logger::debug(
        "orchestrator.start",
        Some(serde_json::json!({
            "promptLength": prompt.len(),
            "contextItems": context.len(),
            "initialModel": options.initial_model.name,
            "maxRetries": max_retries,
            "minConfidence": min_confidence,
        })),
    );

    let mut last_response = String::new();
    let mut last_confidence = 0.0_f64;

    loop {
        state.attempts += 1;

        logger::debug(
            "orchestrator.iteration",
            Some(serde_json::json!({
                "attempt": state.attempts,
                "model": state.current_model.name,
            })),
        );

        let messages = build_messages(prompt, context);

        let completion = options
            .provider
            .chat(LLMChatRequest {
                model: state.current_model.clone(),
                messages: messages.clone(),
                stream: false,
            })
            .await?;

        logger::debug(
            "orchestrator.model_completion",
            Some(serde_json::json!({
                "attempt": state.attempts,
                "model": state.current_model.name,
                "contentPreview": completion.content.chars().take(160).collect::<String>(),
            })),
        );

        last_response = completion.content;

        logger::debug(
            "orchestrator.verifier_call",
            Some(serde_json::json!({
                "attempt": state.attempts,
                "verifierModel": options.verifier_model.name,
            })),
        );

        let verdict = verify_with_llm(
            options.provider,
            options.verifier_model.clone(),
            VerificationRequest {
                prompt: prompt.to_string(),
                response: last_response.clone(),
                context: context.to_vec(),
            },
        )
        .await?;

        last_confidence = combine_confidence(crate::orchestrator::confidence::ConfidenceInputs {
            model_confidence: None,
            verifier_confidence: Some(verdict.confidence),
            embedding_score: None,
        });

        logger::debug(
            "orchestrator.verifier_result",
            Some(serde_json::json!({
                "attempt": state.attempts,
                "approved": verdict.approved,
                "verifierConfidence": verdict.confidence,
                "combinedConfidence": last_confidence,
            })),
        );

        if verdict.approved && is_confidence_acceptable(last_confidence, min_confidence) {
            logger::info(
                "orchestrator.accepted",
                Some(serde_json::json!({
                    "model": state.current_model.name,
                    "attempts": state.attempts,
                    "confidence": last_confidence,
                })),
            );

            return Ok(LoopResult {
                content: last_response,
                model: state.current_model.name.clone(),
                confidence: last_confidence,
                attempts: state.attempts,
            });
        }

        if should_escalate(last_confidence, 0.5) {
            if let Some(next) = escalate_model(&state, &options.escalation_policy) {
                logger::info(
                    "orchestrator.escalate",
                    Some(serde_json::json!({
                        "fromModel": state.current_model.name,
                        "toModel": next.name,
                        "attempts": state.attempts,
                        "confidence": last_confidence,
                    })),
                );
                state.current_model = next;
                continue;
            }
        }

        if state.attempts >= max_retries {
            logger::warn(
                "orchestrator.max_retries_reached",
                Some(serde_json::json!({
                    "attempts": state.attempts,
                    "lastConfidence": last_confidence,
                    "lastModel": state.current_model.name,
                })),
            );
            break;
        }
    }

    logger::info(
        "orchestrator.completed_without_accept",
        Some(serde_json::json!({
            "model": state.current_model.name,
            "attempts": state.attempts,
            "confidence": last_confidence,
        })),
    );

    Ok(LoopResult {
        content: last_response,
        model: state.current_model.name,
        confidence: last_confidence,
        attempts: state.attempts,
    })
}

fn build_messages(prompt: &str, context: &[String]) -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: ChatRole::System,
            content:
                "Answer the user prompt accurately and concisely using the provided context.".to_string(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: build_user_prompt(prompt, context),
        },
    ]
}

fn build_user_prompt(prompt: &str, context: &[String]) -> String {
    if context.is_empty() {
        return prompt.to_string();
    }

    let context_block = context
        .iter()
        .enumerate()
        .map(|(i, c)| format!("[{}] {}", i + 1, c))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "CONTEXT:\n{}\n\nQUESTION:\n{}",
        context_block, prompt
    )
}
