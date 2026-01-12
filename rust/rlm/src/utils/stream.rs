// here

use serde::Serialize;

/// Simple representation of a server-sent event.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct SseEvent<T: Serialize> {
    pub data: T,
}

/// Format an SSE data line ("data: ...\n\n").
pub fn format_sse_event<T: Serialize>(data: &T) -> String {
    let payload = serde_json::to_string(data).unwrap_or_else(|_| "null".to_string());
    format!("data: {}\n\n", payload)
}