export interface ConfidenceInputs {
    modelConfidence?: number;
    verifierConfidence?: number;
    embeddingScore?: number;
}

/**
 * Combine multiple confidence signals into a single score.
 * All inputs are expected to be in the range 0.0 – 1.0.
 */
export function combineConfidence(
    inputs: ConfidenceInputs
): number {
    const weights = {
        model: 0.3,
        verifier: 0.5,
        embedding: 0.2,
    };

    let score = 0;
    let totalWeight = 0;

    if (typeof inputs.modelConfidence === 'number') {
        score += inputs.modelConfidence * weights.model;
        totalWeight += weights.model;
    }

    if (typeof inputs.verifierConfidence === 'number') {
        score += inputs.verifierConfidence * weights.verifier;
        totalWeight += weights.verifier;
    }

    if (typeof inputs.embeddingScore === 'number') {
        score += inputs.embeddingScore * weights.embedding;
        totalWeight += weights.embedding;
    }

    if (totalWeight === 0) {
        return 0;
    }

    return clamp(score / totalWeight, 0, 1);
}

/**
 * Decide whether confidence is sufficient to return an answer.
 */
export function isConfidenceAcceptable(
    confidence: number,
    threshold: number = 0.75
): boolean {
    return confidence >= threshold;
}

/**
 * Decide whether escalation is required.
 */
export function shouldEscalate(
    confidence: number,
    minConfidence: number = 0.5
): boolean {
    return confidence < minConfidence;
}

/**
 * Clamp values safely.
 */
function clamp(
    value: number,
    min: number,
    max: number
): number {
    return Math.max(min, Math.min(max, value));
}
