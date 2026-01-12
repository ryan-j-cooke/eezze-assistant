import { ModelConfig } from '../types/config';

export type Role = 'system' | 'user' | 'assistant' | 'tool';

export interface ChatMessage {
    role: Role;
    content: string;
}

export interface ChatRequest {
    model: ModelConfig;
    messages: ChatMessage[];
    stream?: boolean;
}

export interface ChatResponse {
    id: string;
    created: number;
    model: string;
    choices: ChatChoice[];
}

export interface ChatChoice {
    index: number;
    message: ChatMessage;
    finish_reason: 'stop' | 'length' | 'tool_call' | 'review_failed';
}

export interface CompletionUsage {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
}

/**
 * Reviewer / verifier types
 */
export interface ReviewResult {
    approved: boolean;
    confidence: number;
    notes?: string;
}

/**
 * Embedding store types
 */
export interface EmbeddingRecord {
    id: string;
    text: string;
    vector: number[];
    model: string;
}

export interface RetrievedContext {
    id: string;
    text: string;
    score: number;
}

/**
 * Orchestrator control flow
 */
export interface OrchestratorOptions {
    maxRetries?: number;
    minConfidence?: number;
    escalateModel?: ModelConfig;
}

/**
 * Internal error surface
 */
export interface OrchestratorError {
    code: string;
    message: string;
    recoverable: boolean;
}
