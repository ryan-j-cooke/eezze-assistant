use crate::llm::types::{LLMChatRequest, LLMProvider};
use crate::llm::verifier::{verify_with_llm, VerificationRequest};
use crate::orchestrator::confidence::{
    combine_confidence,
    is_confidence_acceptable,
    should_escalate,
    ConfidenceInputs,
};
use crate::orchestrator::escalate::{escalate_model, EscalationPolicy, EscalationState};
use crate::orchestrator::plan::generate_plan;
use crate::orchestrator::revise::{revise_response, RevisionRequest};
use crate::index::retrieve::review_response;
use crate::types::chat::{ChatMessage, ChatRole};
use crate::types::config::ModelConfig;
use crate::utils::logger;
use crate::api::models::make_status_sse;

pub struct LoopOptions<'a, P: LLMProvider + ?Sized> {
    pub provider: &'a P,
    pub initial_model: ModelConfig,
    pub verifier_model: ModelConfig,
    pub escalation_policy: EscalationPolicy,
    pub max_retries: Option<u32>,
    pub min_confidence: Option<f64>,
    pub on_status: Option<&'a Box<dyn Fn(String) + Send + Sync>>,
}

pub struct LoopResult {
    pub content: String,
    pub model: String,
    pub confidence: f64,
    pub attempts: u32,
}

pub struct RecursiveOptions<'a, P: LLMProvider + ?Sized> {
    pub provider: &'a P,
    pub planning_model: ModelConfig,
    pub initial_model: ModelConfig,
    pub verifier_model: ModelConfig,
    pub revision_model: ModelConfig,
    pub escalation_policy: EscalationPolicy,
    pub max_retries: Option<u32>,
    pub min_confidence: Option<f64>,
    pub on_status: Option<Box<dyn Fn(String) + Send + Sync>>,
}

#[allow(unused_assignments)]
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

    if let Some(ref on_status) = options.on_status {
        on_status(make_status_sse("Starting reasoning loop".to_string(), Some("orchestrator"), None));
    }

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

        if let Some(ref on_status) = options.on_status {
            on_status(make_status_sse(
                format!("Attempt {} with model {}", state.attempts, state.current_model.name),
                Some("orchestrator"),
                None,
            ));
        }

        logger::debug(
            "orchestrator.iteration",
            Some(serde_json::json!({
                "attempt": state.attempts,
                "model": state.current_model.name,
                "provider": options.provider.name(),
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
                "completionModel": completion.model,
                "finishReason": completion.finish_reason,
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

pub async fn run_recursive_session<P: LLMProvider + ?Sized>(
    prompt: &str,
    context: &[String],
    options: RecursiveOptions<'_, P>,
) -> anyhow::Result<LoopResult> {
    let planning_model = options.planning_model.clone();
    let revision_model = options.revision_model.clone();
    let min_confidence = options.min_confidence.unwrap_or(0.75);

    // 1) Planning phase
    logger::debug(
        "orchestrator.plan.start",
        Some(serde_json::json!({
            "model": planning_model.name,
        })),
    );

    if let Some(ref on_status) = options.on_status {
        on_status(make_status_sse("Generating plan...".to_string(), Some("planning"), None));
    }

    let plan_messages = generate_plan(options.provider, planning_model.clone(), prompt).await?;
    let plan_message = plan_messages
        .last()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let mut enriched_context: Vec<String> = context.to_vec();
    enriched_context.push(format!("PLAN:\n{}", plan_message));

    logger::debug(
        "orchestrator.plan.result",
        Some(serde_json::json!({
            "model": planning_model.name,
            "planPreview": plan_message.chars().take(200).collect::<String>(),
        })),
    );

    // 2) Core orchestrator loop using the enriched context
    let loop_result = run_orchestrator(
        prompt,
        &enriched_context,
        LoopOptions {
            provider: options.provider,
            initial_model: options.initial_model.clone(),
            verifier_model: options.verifier_model.clone(),
            escalation_policy: options.escalation_policy,
            max_retries: options.max_retries,
            min_confidence: options.min_confidence,
            on_status: options.on_status.as_ref(),
        },
    )
    .await?;

    // 3) Final verification of the chosen answer (LLM verifier + embeddings)
    if let Some(ref on_status) = options.on_status {
        on_status(make_status_sse("Verifying answer...".to_string(), Some("verification"), None));
    }

    let final_verdict = verify_with_llm(
        options.provider,
        options.verifier_model.clone(),
        VerificationRequest {
            prompt: prompt.to_string(),
            response: loop_result.content.clone(),
            context: enriched_context.clone(),
        },
    )
    .await?;

    let embed_review = review_response(prompt, &loop_result.content, &enriched_context).await?;

    let final_confidence = combine_confidence(ConfidenceInputs {
        model_confidence: None,
        verifier_confidence: Some(final_verdict.confidence),
        embedding_score: Some(embed_review.confidence),
    });

    logger::debug(
        "orchestrator.final_verifier_result",
        Some(serde_json::json!({
            "approvedVerifier": final_verdict.approved,
            "verifierConfidence": final_verdict.confidence,
            "approvedEmbeddings": embed_review.approved,
            "embeddingConfidence": embed_review.confidence,
            "embeddingNotes": embed_review.notes,
            "combinedConfidence": final_confidence,
        })),
    );

    if final_verdict.approved
        && embed_review.approved
        && is_confidence_acceptable(final_confidence, min_confidence)
    {
        return Ok(LoopResult {
            content: loop_result.content,
            model: loop_result.model,
            confidence: final_confidence,
            attempts: loop_result.attempts,
        });
    }

    // 4) Revision phase if the final answer is still not acceptable
    logger::info(
        "orchestrator.revise.start",
        Some(serde_json::json!({
            "model": revision_model.name,
        })),
    );

    if let Some(ref on_status) = options.on_status {
        on_status(make_status_sse("Revising answer...".to_string(), Some("revision"), None));
    }

    let revision = revise_response(
        options.provider,
        revision_model.clone(),
        &RevisionRequest {
            original_prompt: prompt.to_string(),
            previous_response: loop_result.content,
            context: enriched_context,
            reviewer_notes: None,
        },
    )
    .await?;

    logger::info(
        "orchestrator.revise.result",
        Some(serde_json::json!({
            "model": revision.model,
        })),
    );

    Ok(LoopResult {
        content: revision.content,
        model: revision.model,
        confidence: final_confidence,
        attempts: loop_result.attempts,
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
