use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::index::types::EmbedResult;
use crate::utils::logger;

#[derive(Debug, Serialize)]
struct OllamaEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
    model: Option<String>,
}

pub async fn handle_embeddings(
    client: &Client,
    text: &str,
    model: &str,
    normalize: bool,
) -> anyhow::Result<EmbedResult> {
    let truncated = truncate_for_embedding(text);

    logger::debug(
        "embeddings.request",
        Some(serde_json::json!({
            "model": model,
            "originalLen": text.len(),
            "truncatedLen": truncated.len(),
            "normalized": normalize,
        })),
    );

    let request = OllamaEmbeddingRequest {
        model,
        input: truncated,
    };

    let response = client
        .post("http://localhost:11434/api/embeddings")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body_text = response.text().await.unwrap_or_else(|_| "<body read error>".to_string());

        logger::error(
            "embeddings.http_error",
            Some(serde_json::json!({
                "status": status,
                "model": model,
                "body": body_text,
                "originalLen": text.len(),
                "truncatedLen": truncated.len(),
            })),
        );

        anyhow::bail!("Embedding request failed ({})", status);
    }

    let data: OllamaEmbeddingResponse = response.json().await?;

    let mut vector = data.embedding;

    if normalize {
        vector = normalize_vector(&vector);
    }

    let result = EmbedResult {
        dimensions: vector.len(),
        model: data.model.unwrap_or_else(|| model.to_string()),
        vector,
    };

    logger::debug(
        "embeddings.generated",
        Some(serde_json::json!({
            "dimensions": result.dimensions,
            "model": result.model,
        })),
    );

    Ok(result)
}

fn normalize_vector(vector: &[f32]) -> Vec<f32> {
    let norm = vector
        .iter()
        .fold(0.0_f32, |acc, v| acc + v * v)
        .sqrt();

    if norm == 0.0 {
        return vector.to_vec();
    }

    vector.iter().map(|v| v / norm).collect()
}

fn truncate_for_embedding(text: &str) -> &str {
    const MAX_CHARS: usize = 8000;

    if text.len() <= MAX_CHARS {
        return text;
    }

    let mut end = MAX_CHARS;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }

    &text[..end]
}
