use async_trait::async_trait;

use crate::types::chat::ChatMessage;
use crate::types::config::ModelConfig;

#[derive(Debug, Clone)]
pub struct LLMChatRequest {
    pub model: ModelConfig,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

#[derive(Debug, Clone)]
pub struct LLMChatResponse {
    pub content: String,
    pub model: String,
    pub finish_reason: Option<String>,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;

    async fn chat(&self, request: LLMChatRequest) -> anyhow::Result<LLMChatResponse>;
}
