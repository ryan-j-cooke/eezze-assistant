import { ChatMessage } from '../types/chat';
import { ModelConfig } from '../types/config';

/**
 * Generic LLM chat request
 */
export interface LLMChatRequest {
    model: ModelConfig;
    messages: ChatMessage[];
    stream?: boolean;
}

/**
 * Generic LLM chat response
 */
export interface LLMChatResponse {
    content: string;
    model: string;
    finishReason?: 'stop' | 'length' | 'error';
}

/**
 * LLM transport interface
 */
export interface LLMProvider {
    name: string;

    chat(
        request: LLMChatRequest
    ): Promise<LLMChatResponse>;
}
