use crate::llm::types::{LLMChatRequest, LLMProvider};
use crate::types::chat::ChatMessage;
use crate::types::config::ModelConfig;

pub struct VerificationRequest {
    pub prompt: String,
    pub response: String,
    pub context: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub approved: bool,
    pub confidence: f64,
    pub notes: Option<String>,
}

pub async fn verify_with_llm<P: LLMProvider + ?Sized>(
    provider: &P,
    model: ModelConfig,
    request: VerificationRequest,
) -> anyhow::Result<VerificationResult> {
    let messages = build_verifier_messages(&request);

    let result = provider
        .chat(LLMChatRequest {
            model,
            messages,
            stream: false,
        })
        .await?;

    Ok(parse_verifier_output(&result.content))
}

fn build_verifier_messages(request: &VerificationRequest) -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: crate::types::chat::ChatRole::System,
            content: "You are a strict verifier. Your job is to approve or reject answers. You must be conservative and reject if unsure.".to_string(),
        },
        ChatMessage {
            role: crate::types::chat::ChatRole::User,
            content: format!(
                "USER PROMPT:\n{}\n\nMODEL RESPONSE:\n{}\n\nREFERENCE CONTEXT:\n{}\n\nTASK:\n1. Is the response correct?\n2. Is it fully supported by the reference context?\n3. Does it avoid speculation or fabrication?\n\nRespond ONLY with valid JSON in the following format:\n{{\n  \"approved\": boolean,\n  \"confidence\": number,\n  \"notes\": string\n}}",
                request.prompt,
                request.response,
                request
                    .context
                    .iter()
                    .enumerate()
                    .map(|(i, c)| format!("[{}] {}", i + 1, c))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        },
    ]
}

fn parse_verifier_output(text: &str) -> VerificationResult {
    let (start, end) = match (text.find('{'), text.rfind('}')) {
        (Some(s), Some(e)) if s <= e => (s, e),
        _ => {
            return VerificationResult {
                approved: false,
                confidence: 0.0,
                notes: Some("Verifier output could not be parsed".to_string()),
            }
        }
    };

    let slice = &text[start..=end];
    match serde_json::from_str::<serde_json::Value>(slice) {
        Ok(parsed) => {
            let approved = parsed
                .get("approved")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let confidence = parsed
                .get("confidence")
                .and_then(|v| v.as_f64())
                .map(|c| clamp(c, 0.0, 1.0))
                .unwrap_or(0.0);
            let notes = parsed
                .get("notes")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            VerificationResult {
                approved,
                confidence,
                notes,
            }
        }
        Err(_) => VerificationResult {
            approved: false,
            confidence: 0.0,
            notes: Some("Verifier output could not be parsed".to_string()),
        },
    }
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
