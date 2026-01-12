use crate::llm::types::{LLMChatRequest, LLMProvider};
use crate::types::chat::{ChatMessage, ChatRole};
use crate::types::config::ModelConfig;

pub struct RevisionRequest {
    pub original_prompt: String,
    pub previous_response: String,
    pub context: Vec<String>,
    pub reviewer_notes: Option<String>,
}

pub struct RevisionResult {
    pub content: String,
    pub model: String,
}

/// Revise a previous model response using explicit feedback.
pub async fn revise_response<P: LLMProvider + ?Sized>(
    provider: &P,
    model: ModelConfig,
    request: &RevisionRequest,
) -> anyhow::Result<RevisionResult> {
    let messages = build_revision_messages(request);

    let result = provider
        .chat(LLMChatRequest {
            model: model.clone(),
            messages,
            stream: false,
        })
        .await?;

    Ok(RevisionResult {
        content: result.content,
        model: model.name,
    })
}

fn build_revision_messages(request: &RevisionRequest) -> Vec<ChatMessage> {
    let context_block = request
        .context
        .iter()
        .enumerate()
        .map(|(i, c)| format!("[{}] {}", i + 1, c))
        .collect::<Vec<_>>()
        .join("\n");

    let reviewer_block = request
        .reviewer_notes
        .as_ref()
        .map(|notes| format!("REVIEWER FEEDBACK:\n{}\n", notes))
        .unwrap_or_default();

    let user_content = format!(
        "ORIGINAL QUESTION:\n{}\n\nPREVIOUS (REJECTED) RESPONSE:\n{}\n\n{}REFERENCE CONTEXT:\n{}\n\nTASK:\nRewrite the response so that it is:\n- Factually correct\n- Fully supported by the reference context\n- Clear and concise\n- Free of speculation\n\nReturn ONLY the revised answer text.",
        request.original_prompt,
        request.previous_response,
        reviewer_block,
        context_block,
    );

    vec![
        ChatMessage {
            role: ChatRole::System,
            content:
                "You are revising a previous answer that was rejected. Correct errors, remove unsupported claims, and strictly adhere to the provided context.".to_string(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: user_content,
        },
    ]
}

// here