#![allow(dead_code)]

use crate::types::chat::ChatMessage;
use crate::types::config::ModelConfig;

#[derive(Debug, Clone)]
pub struct OrchestratorAttempt {
    pub attempt: u32,
    pub model: String,
    pub response: String,
    pub confidence: Option<f64>,
    pub approved: Option<bool>,
    pub reviewer_notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OrchestratorContext {
    pub prompt: String,
    pub messages: Vec<ChatMessage>,
    pub retrieved_context: Vec<String>,
    pub attempts: Vec<OrchestratorAttempt>,
}

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub max_attempts: u32,
    pub confidence_threshold: f64,
    pub initial_model: ModelConfig,
    pub escalation_model: Option<ModelConfig>,
}

#[derive(Debug, Clone)]
pub struct OrchestratorResult {
    pub content: String,
    pub model: String,
    pub confidence: f64,
    pub attempts: Vec<OrchestratorAttempt>,
}
