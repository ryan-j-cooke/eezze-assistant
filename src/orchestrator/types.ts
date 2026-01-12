import { ChatMessage } from '../types/chat';
import { ModelConfig } from '../types/config';

/**
 * A single execution attempt within the orchestration loop.
 */
export interface OrchestratorAttempt {
    attempt: number;
    model: string;
    response: string;
    confidence?: number;
    approved?: boolean;
    reviewerNotes?: string;
}

/**
 * Shared context passed between orchestration stages.
 */
export interface OrchestratorContext {
    prompt: string;
    messages: ChatMessage[];
    retrievedContext: string[];
    attempts: OrchestratorAttempt[];
}

/**
 * Configuration governing orchestration behavior.
 */
export interface OrchestratorConfig {
    maxAttempts: number;
    confidenceThreshold: number;
    initialModel: ModelConfig;
    escalationModel?: ModelConfig;
}

/**
 * Result returned from the full orchestration loop.
 */
export interface OrchestratorResult {
    content: string;
    model: string;
    confidence: number;
    attempts: OrchestratorAttempt[];
}
