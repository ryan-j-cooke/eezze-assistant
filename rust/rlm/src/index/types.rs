#[derive(Debug, Clone)]
pub struct EmbedResult {
    pub vector: Vec<f32>,
    pub dimensions: usize,
    pub model: String,
}
