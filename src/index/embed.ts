import crypto from 'crypto';
import { EXPERT_EMBEDDING_DEFAULT } from '../../eezze.config';

export interface EmbedResult {
    vector: number[];
    dimensions: number;
    model: string;
}

export interface OllamaEmbeddingResponse {
    embedding: number[];
    model?: string;
}

/**
 * Generate an embedding for the given text.
 * This is used ONLY for retrieval / verification,
 * never for direct reasoning.
 */
export async function handleEmbeddings(
    text: string,
    options?: {
        model?: string;
        normalize?: boolean;
    }
): Promise<EmbedResult> {
    const model = options?.model ?? EXPERT_EMBEDDING_DEFAULT;

    const response = await fetch('http://localhost:11434/api/embeddings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ model, prompt: text }),
    });

    if (!response.ok) {
        throw new Error(`Embedding request failed (${response.status})`);
    }

    // Type-assert to known response shape
    const data = (await response.json()) as OllamaEmbeddingResponse;

    if (!data.embedding || !Array.isArray(data.embedding)) {
        throw new Error('Invalid embedding response from Ollama');
    }

    let vector: number[] = data.embedding;

    if (options?.normalize !== false) {
        vector = normalizeVector(vector);
    }

    return {
        vector,
        dimensions: vector.length,
        model,
    };
}

/**
 * Normalize a vector to unit length.
 * Improves cosine similarity stability.
 */
function normalizeVector(vector: number[]): number[] {
    const norm = Math.sqrt(
        vector.reduce((sum, value) => sum + value * value, 0)
    );

    if (norm === 0) {
        return vector;
    }

    return vector.map((value) => value / norm);
}

/**
 * Create a stable hash for embedding cache keys.
 */
export function embeddingKey(text: string, model: string): string {
    return crypto
        .createHash('sha256')
        .update(`${model}:${text}`)
        .digest('hex');
}
