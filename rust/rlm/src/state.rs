// state here.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ProjectState {
    projects: HashMap<String, String>, // key: workspace root, value: optional metadata
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }

    // Register or update a project by its workspace root path
    pub fn set_project(&mut self, workspace_root: String) {
        self.projects.insert(workspace_root.clone(), workspace_root);
    }

    // Get the registered workspace root (returns the first one for now)
    pub fn get_workspace_root(&self) -> Option<&String> {
        self.projects.keys().next()
    }
}

// Thread-safe global state
lazy_static::lazy_static! {
    pub static ref PROJECT_STATE: Arc<Mutex<ProjectState>> = Arc::new(Mutex::new(ProjectState::new()));
}

/// Helper to set the current project from a request containing <workspace_info>
pub fn set_project_from_request(messages: &[crate::types::chat::ChatMessage]) {
    if let Some(root) = extract_workspace_root(messages) {
        if let Ok(mut state) = PROJECT_STATE.lock() {
            let mut state_ref = state;
            state_ref.set_project(root);
        }
    }
}

/// Extract the workspace root from <workspace_info> block
fn extract_workspace_root(messages: &[crate::types::chat::ChatMessage]) -> Option<String> {
    for msg in messages.iter().rev() {
        if msg.role == crate::types::chat::ChatRole::User {
            let content = &msg.content;
            if let Some(start) = content.find("<workspace_info>") {
                if let Some(end) = content.find("</workspace_info>") {
                    let block = &content[start..end + 16];
                    // Look for a line starting with "- /path/to/root"
                    for line in block.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("- ") && trimmed.contains('/') {
                            let path = trimmed.strip_prefix("- ").unwrap_or(trimmed);
                            return Some(path.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}