import { ChatMessage } from "../types/chat";
import { ModelConfig } from "../types/config";
import { logger } from "../utils/logger";

export interface OllamaChatRequest {
    model: string;
    messages: ChatMessage[];
    temperature?: number;
    max_tokens?: number;
    stream?: boolean;
}

export interface OllamaChatResponse {
    message: {
        role: 'assistant';
        content: string;
    };
    done: boolean;
}

/**
 * Send a chat request to a locally running Ollama instance.
 */
export async function ollamaChat(
    config: ModelConfig,
    messages: ChatMessage[],
    options?: {
        stream?: boolean;
    }
): Promise<string> {
    const request: OllamaChatRequest = {
        model: config.name,
        messages,
        temperature: config.temperature,
        max_tokens: config.maxTokens,
        stream: options?.stream ?? false,
    };

    logger.debug('ollama.chat.request', {
        model: config.name,
        messages: messages.length,
        temperature: config.temperature,
        maxTokens: config.maxTokens,
        stream: request.stream,
    });

    const started = Date.now();

    const response = await fetch(
        'http://localhost:11434/api/chat',
        {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(request),
        }
    );

    const latencyMs = Date.now() - started;

    if (!response.ok) {
        logger.error('ollama.chat.http_error', {
            status: response.status,
            model: config.name,
            latencyMs,
        });

        throw new Error(
            `Ollama chat request failed (${response.status})`
        );
    }

    if (request.stream) {
        logger.error('ollama.chat.streaming_not_implemented', {
            model: config.name,
        });
        throw new Error('Streaming not implemented yet');
    }

    const data = (await response.json()) as OllamaChatResponse;

    if (!data?.message?.content) {
        logger.error('ollama.chat.invalid_response', {
            model: config.name,
            latencyMs,
        });
        throw new Error('Invalid response from Ollama');
    }

    logger.info('ollama.chat.success', {
        model: config.name,
        latencyMs,
    });

    return data.message.content;
}
