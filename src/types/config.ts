import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    EXPERT_FAST_MODEL,
    EXPERT_REVIEWER_MODEL,
} from '../../eezze.config';

/**
 * Global runtime configuration.
 * Loaded at startup and treated as immutable.
 */
export interface RuntimeConfig {
    server: ServerConfig;
    models: ModelConfigMap;
    orchestration: OrchestrationConfig;
    index: IndexConfig;
}

/**
 * HTTP server configuration.
 */
export interface ServerConfig {
    host?: string;
    port: number;
}

/**
 * Model definitions available to the system.
 */
export interface ModelConfig {
    name: string;
    provider: 'ollama' | 'openai' | 'local';
    temperature?: number;
    maxTokens?: number;
}

/**
 * Named model registry.
 * Example keys: 'fast', 'reviewer', 'escalation'
 */
export type ModelConfigMap = Record<string, ModelConfig>;

/**
 * Orchestration policy configuration.
 */
export interface OrchestrationConfig {
    maxAttempts: number;
    confidenceThreshold: number;
    allowEscalation: boolean;
}

/**
 * Vector index configuration.
 */
export interface IndexConfig {
    dimensions: number;
    similarityThreshold: number;
}

export function loadConfig(): RuntimeConfig {
    // Option 1: load from a JSON file (fallback to default)
    const configPath = path.resolve(
        path.dirname(fileURLToPath(import.meta.url)),
        '../../config.json'
    );
    if (fs.existsSync(configPath)) {
        const raw = fs.readFileSync(configPath, 'utf-8');
        return JSON.parse(raw) as RuntimeConfig;
    }

    // Option 2: fallback defaults
    return {
        server: {
            host: '127.0.0.1',
            port: 4000
        },
        models: {
            fast: { name: EXPERT_FAST_MODEL, provider: 'ollama' },
            reviewer: { name: EXPERT_REVIEWER_MODEL, provider: 'ollama' }
        },
        orchestration: {
            maxAttempts: 3,
            confidenceThreshold: 0.75,
            allowEscalation: true
        } as OrchestrationConfig,
        index: {
            dimensions: 768,
            similarityThreshold: 0.8
        } as IndexConfig
    };
}
