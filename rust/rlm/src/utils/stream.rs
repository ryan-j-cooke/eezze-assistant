// here

use serde::Serialize;

/// Simple representation of a server-sent event.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct SseEvent<T: Serialize> {
    pub data: T,
}

/// OpenAI-compatible streaming chunk for chat completions.
#[derive(Debug, Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: &'static str,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionChoice {
    pub index: usize,
    pub delta: ChatCompletionDelta,
    pub finish_reason: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
}

/// Format an SSE data line ("data: ...\n\n").
pub fn format_sse_event<T: Serialize>(data: &T) -> String {
    let payload = serde_json::to_string(data).unwrap_or_else(|_| "null".to_string());
    format!("data: {}\n\n", payload)
}

pub fn make_chat_chunk(id: &str, model: &str, content: Option<String>, finish_reason: Option<&'static str>) -> String {
    let chunk = ChatCompletionChunk {
        id: id.to_string(),
        object: "chat.completion.chunk",
        created: chrono::Utc::now().timestamp(),
        model: model.to_string(),
        choices: vec![ChatCompletionChoice {
            index: 0,
            delta: ChatCompletionDelta {
                role: None,
                content,
                thinking: None,
            },
            finish_reason,
        }],
    };
    format_sse_event(&chunk)
}

pub fn make_thinking_chunk(id: &str, model: &str, thinking: String) -> String {
    let chunk = ChatCompletionChunk {
        id: id.to_string(),
        object: "chat.completion.chunk",
        created: chrono::Utc::now().timestamp(),
        model: model.to_string(),
        choices: vec![ChatCompletionChoice {
            index: 0,
            delta: ChatCompletionDelta {
                role: None,
                content: None,
                thinking: Some(thinking),
            },
            finish_reason: None,
        }],
    };
    format_sse_event(&chunk)
}