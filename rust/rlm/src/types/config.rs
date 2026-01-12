#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub server: ServerConfig,
    pub models: ModelConfigMap,
    pub orchestration: OrchestrationConfig,
    pub index: IndexConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default)]
    pub host: Option<String>,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    #[serde(rename = "provider")]
    pub provider: ModelProvider,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default, rename = "maxTokens")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    Ollama,
    Openai,
    Local,
}

pub type ModelConfigMap = HashMap<String, ModelConfig>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    pub max_attempts: u32,
    pub confidence_threshold: f64,
    pub allow_escalation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub dimensions: u32,
    pub similarity_threshold: f64,
}

