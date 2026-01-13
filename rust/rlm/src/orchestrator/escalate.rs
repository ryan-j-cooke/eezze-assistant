use crate::types::config::ModelConfig;

pub struct EscalationState {
    pub current_model: ModelConfig,
    pub attempts: u32,
}

#[derive(Debug, Clone, Default)]
pub struct EscalationPolicy {
    pub max_attempts: u32,
    pub ladder: Vec<ModelConfig>,
    pub max_model: String,
    pub fallback_models: Vec<String>,
}

pub fn can_escalate(state: &EscalationState, policy: &EscalationPolicy) -> bool {
    if state.attempts >= policy.max_attempts {
        return false;
    }

    index_of_model(&state.current_model, &policy.ladder) < policy.ladder.len() - 1
}

pub fn escalate_model(
    state: &EscalationState,
    policy: &EscalationPolicy,
) -> Option<ModelConfig> {
    if !can_escalate(state, policy) {
        return None;
    }

    let current_index = index_of_model(&state.current_model, &policy.ladder);
    policy.ladder.get(current_index + 1).cloned()
}

fn index_of_model(model: &ModelConfig, ladder: &[ModelConfig]) -> usize {
    ladder
        .iter()
        .position(|m| m.name == model.name)
        .unwrap_or_else(|| {
            panic!("Model {} not found in escalation ladder", model.name);
        })
}
