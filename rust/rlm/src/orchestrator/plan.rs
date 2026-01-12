use crate::llm::types::{LLMChatRequest, LLMProvider};
use crate::prompts::plan::PLAN_PROMPT;
use crate::types::chat::{ChatMessage, ChatRole};
use crate::types::config::ModelConfig;

/// Generate a high-level execution plan using a small planning model.
pub async fn generate_plan<P: LLMProvider + ?Sized>(
    provider: &P,
    model: ModelConfig,
    user_prompt: &str,
) -> anyhow::Result<Vec<ChatMessage>> {
    let mut messages: Vec<ChatMessage> = vec![
        ChatMessage {
            role: ChatRole::System,
            content: PLAN_PROMPT.to_string(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: user_prompt.to_string(),
        },
    ];

    let response = provider
        .chat(LLMChatRequest {
            model: model.clone(),
            messages: messages.clone(),
            stream: false,
        })
        .await?;

    messages.push(ChatMessage {
        role: ChatRole::Assistant,
        content: response.content,
    });

    Ok(messages)
}

// here