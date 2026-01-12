#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct StoredEmbedding {
    pub id: String,
    pub text: String,
    pub vector: Vec<f32>,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub id: String,
    pub text: String,
    pub score: f32,
}

/// A simple in-memory embedding store.
#[derive(Default, Debug)]
pub struct EmbeddingStore {
    items: Vec<StoredEmbedding>,
}

impl EmbeddingStore {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Insert or update an embedding.
    pub fn upsert(&mut self, embedding: StoredEmbedding) {
        if let Some(existing) = self.items.iter_mut().find(|e| e.id == embedding.id) {
            *existing = embedding;
        } else {
            self.items.push(embedding);
        }
    }

    /// Retrieve top-K most similar embeddings.
    pub fn query(
        &self,
        query_vector: &[f32],
        limit: Option<usize>,
        min_score: Option<f32>,
        model: Option<&str>,
    ) -> Vec<QueryResult> {
        let limit = limit.unwrap_or(5);
        let min_score = min_score.unwrap_or(0.0);

        let iter = self.items.iter().filter(|item| match model {
            Some(m) => item.model == m,
            None => true,
        });

        let mut results: Vec<QueryResult> = iter
            .filter_map(|item| {
                let score = cosine_similarity(query_vector, &item.vector);
                if score >= min_score {
                    Some(QueryResult {
                        id: item.id.clone(),
                        text: item.text.clone(),
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }

    pub fn delete(&mut self, id: &str) {
        self.items.retain(|item| item.id != id);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
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