use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::types::chat::ChatMessage;
use crate::types::config::ModelConfig;
use crate::utils::logger;

#[derive(Debug, Serialize)]
struct OllamaChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "max_tokens")]
    max_tokens: Option<u32>,
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaChatResponseMessage,
    done: bool,
}

pub async fn ollama_chat(
    client: &Client,
    config: &ModelConfig,
    messages: &[ChatMessage],
    stream: bool,
) -> anyhow::Result<String> {
    let request = OllamaChatRequest {
        model: &config.name,
        messages,
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        stream,
    };

    logger::debug(
        "ollama.chat.request",
        Some(serde_json::json!({
            "model": config.name,
            "messages": messages.len(),
            "temperature": config.temperature,
            "maxTokens": config.max_tokens,
            "stream": stream,
        })),
    );

    let started = std::time::Instant::now();

    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&request)
        .send()
        .await?;

    let latency_ms = started.elapsed().as_millis();

    if !response.status().is_success() {
        logger::error(
            "ollama.chat.http_error",
            Some(serde_json::json!({
                "status": response.status().as_u16(),
                "model": config.name,
                "latencyMs": latency_ms,
            })),
        );
        anyhow::bail!(
            "Ollama chat request failed ({})",
            response.status().as_u16()
        );
    }

    if stream {
        logger::error(
            "ollama.chat.streaming_not_implemented",
            Some(serde_json::json!({ "model": config.name })),
        );
        anyhow::bail!("Streaming not implemented yet");
    }

    let data: OllamaChatResponse = response.json().await?;

    if data.message.content.is_empty() {
        logger::error(
            "ollama.chat.invalid_response",
            Some(serde_json::json!({
                "model": config.name,
                "latencyMs": latency_ms,
            })),
        );
        anyhow::bail!("Invalid response from Ollama");
    }

    logger::info(
        "ollama.chat.success",
        Some(serde_json::json!({
            "model": config.name,
            "latencyMs": latency_ms,
        })),
    );

    Ok(data.message.content)
}
