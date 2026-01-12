use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::index::types::EmbedResult;
use crate::utils::logger;

#[derive(Debug, Serialize)]
struct OllamaEmbeddingRequest<'a> {
    model: &'a str,
    prompt: &'a str,
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
    let request = OllamaEmbeddingRequest { model, prompt: text };

    let response = client
        .post("http://localhost:11434/api/embeddings")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Embedding request failed ({})",
            response.status().as_u16()
        );
    }

    let data: OllamaEmbeddingResponse = response.json().await?;

    if data.embedding.is_empty() {
        anyhow::bail!("Invalid embedding response from Ollama");
    }

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
