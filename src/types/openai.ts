/**
 * Common OpenAI error shape.
 */
export interface OpenAIError {
    message: string;
    type?: string;
    code?: string;
}

/**
 * Embeddings request.
 */
export interface OpenAIEmbeddingRequest {
    model: string;
    input: string | string[];
}

/**
 * Single embedding object.
 */
export interface OpenAIEmbedding {
    object: 'embedding';
    embedding: number[];
    index: number;
}

/**
 * Embeddings response.
 */
export interface OpenAIEmbeddingResponse {
    object: 'list';
    data: OpenAIEmbedding[];
    model: string;
}

/**
 * Streaming chunk format for chat completions.
 */
export interface OpenAIChatCompletionChunk {
    id: string;
    object: 'chat.completion.chunk';
    created: number;
    model: string;
    choices: Array<{
        index: number;
        delta: {
            role?: 'assistant';
            content?: string;
        };
        finish_reason?: 'stop' | 'length';
    }>;
}
