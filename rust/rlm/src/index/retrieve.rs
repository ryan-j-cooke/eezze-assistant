// Embedding-based review helpers used by the orchestrator's higher-level recursive session.
use crate::index::embed::handle_embeddings;
use crate::orchestrator::confidence::{combine_confidence, ConfidenceInputs};

use reqwest::Client;

pub struct ReviewResult {
    pub approved: bool,
    pub confidence: f64,
    pub notes: Option<String>,
}
/// Embedding-based semantic verification.
pub async fn review_response(
    prompt: &str,
    response: &str,
    context: &[String],
) -> anyhow::Result<ReviewResult> {
    let client = Client::new();

    let response_embedding = handle_embeddings(&client, response, "nomic-embed-text", true).await?;

    let mut max_score = 0.0_f32;

    // Include the original prompt as another semantic reference point.
    let mut enriched_context: Vec<String> = context.to_vec();
    enriched_context.push(prompt.to_string());

    for chunk in &enriched_context {
        let context_embedding = handle_embeddings(&client, chunk, "nomic-embed-text", true).await?;

        let score = cosine_similarity(&response_embedding.vector, &context_embedding.vector);
        if score > max_score {
            max_score = score;
        }
    }

    let confidence = combine_confidence(ConfidenceInputs {
        model_confidence: None,
        verifier_confidence: None,
        embedding_score: Some(max_score as f64),
    });

    Ok(ReviewResult {
        approved: max_score >= 0.75,
        confidence,
        notes: None,
    })
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;

    let len = a.len().min(b.len());
    for i in 0..len {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a.sqrt() * norm_b.sqrt())
}