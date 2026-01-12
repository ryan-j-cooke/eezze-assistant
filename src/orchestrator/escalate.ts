import { ModelConfig } from '../types/config';

export interface EscalationState {
    currentModel: ModelConfig;
    attempts: number;
}

export interface EscalationPolicy {
    maxAttempts: number;
    ladder: ModelConfig[];
}

/**
 * Decide whether escalation is allowed.
 */
export function canEscalate(
    state: EscalationState,
    policy: EscalationPolicy
): boolean {
    if (state.attempts >= policy.maxAttempts) {
        return false;
    }

    return (
        indexOfModel(state.currentModel, policy.ladder) <
        policy.ladder.length - 1
    );
}

/**
 * Return the next model in the escalation ladder.
 */
export function escalateModel(
    state: EscalationState,
    policy: EscalationPolicy
): ModelConfig | null {
    if (!canEscalate(state, policy)) {
        return null;
    }

    const currentIndex = indexOfModel(
        state.currentModel,
        policy.ladder
    );

    return policy.ladder[currentIndex + 1];
}

/**
 * Find a model in the ladder by name.
 */
function indexOfModel(
    model: ModelConfig,
    ladder: ModelConfig[]
): number {
    const index = ladder.findIndex(
        (m) => m.name === model.name
    );

    if (index === -1) {
        throw new Error(
            `Model ${model.name} not found in escalation ladder`
        );
    }

    return index;
}
