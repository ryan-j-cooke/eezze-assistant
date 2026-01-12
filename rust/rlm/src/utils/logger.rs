
use serde::Serialize;
use serde_json::Value;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<Value>,
}

fn now_iso_like() -> String {
    // Simple ISO-like timestamp based on UNIX epoch seconds; precise formatting
    // is less important than monotonicity for logs.
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

fn log_internal(level: &str, message: &str, meta: Option<Value>) {
    let entry = LogEntry {
        timestamp: now_iso_like(),
        level: level.to_string(),
        message: message.to_string(),
        meta,
    };

    match serde_json::to_string(&entry) {
        Ok(payload) => {
            if level == "error" {
                let _ = writeln!(io::stderr(), "{}", payload);
            } else {
                let _ = writeln!(io::stdout(), "{}", payload);
            }
        }
        Err(_) => {
            // Fallback to a very simple log line if serialization fails.
            let _ = writeln!(io::stderr(), "[logger-error] {}: {}", level, message);
        }
    }
}

fn meta_to_value<M: Serialize>(meta: Option<M>) -> Option<Value> {
    meta.and_then(|m| serde_json::to_value(m).ok())
}

pub fn debug<M: Serialize>(message: &str, meta: Option<M>) {
    log_internal("debug", message, meta_to_value(meta));
}

pub fn info<M: Serialize>(message: &str, meta: Option<M>) {
    log_internal("info", message, meta_to_value(meta));
}

pub fn warn<M: Serialize>(message: &str, meta: Option<M>) {
    log_internal("warn", message, meta_to_value(meta));
}

pub fn error<M: Serialize>(message: &str, meta: Option<M>) {
    log_internal("error", message, meta_to_value(meta));
}

