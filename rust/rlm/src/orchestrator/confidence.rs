#[derive(Debug, Clone, Default)]
pub struct ConfidenceInputs {
    pub model_confidence: Option<f64>,
    pub verifier_confidence: Option<f64>,
    pub embedding_score: Option<f64>,
}

pub fn combine_confidence(inputs: ConfidenceInputs) -> f64 {
    let weights = (0.3_f64, 0.5_f64, 0.2_f64);

    let mut score = 0.0;
    let mut total_weight = 0.0;

    if let Some(c) = inputs.model_confidence {
        score += c * weights.0;
        total_weight += weights.0;
    }

    if let Some(c) = inputs.verifier_confidence {
        score += c * weights.1;
        total_weight += weights.1;
    }

    if let Some(c) = inputs.embedding_score {
        score += c * weights.2;
        total_weight += weights.2;
    }

    if total_weight == 0.0 {
        return 0.0;
    }

    clamp(score / total_weight, 0.0, 1.0)
}

pub fn is_confidence_acceptable(confidence: f64, threshold: f64) -> bool {
    confidence >= threshold
}

pub fn should_escalate(confidence: f64, min_confidence: f64) -> bool {
    confidence < min_confidence
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
