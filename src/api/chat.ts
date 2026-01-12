import { Express, Request, Response } from 'express';
import { randomUUID } from 'crypto';
import { logger } from '../utils/logger';
import { ChatCompletionRequest, ChatCompletionResponse, ChatMessage } from '../types/chat';
import { ModelConfig } from '../types/config';
import { LLMProvider } from '../llm/types';
import { ollamaChat } from '../llm/ollama';
import { EscalationPolicy } from '../orchestrator/escalate';
import { runRecursiveSession } from '../orchestrator/loop';
import { EXPERT_FAST_MODEL, EXPERT_REVIEWER_MODEL } from '../../eezze.config';

export async function handleChatCompletion(body: ChatCompletionRequest): Promise<ChatCompletionResponse> {
    const prompt = buildPromptFromMessages(body.messages);

    const rawModel = body.model;

    // Temporarily disable alias/colon parsing: use the incoming model string
    // directly as the Ollama model name.
    const clientModelId: string = rawModel;
    const ollamaModelName: string = rawModel;

    logger.info('chat.completion.handle.start', {
        model: rawModel,
        clientModelId,
        ollamaModelName,
        messages: body.messages.length,
        temperature: body.temperature,
        maxTokens: body.max_tokens,
    });

    // Main answering model: honour the client-requested model string.
    const initialModel: ModelConfig = {
        name: ollamaModelName,
        provider: 'ollama',
        temperature: body.temperature,
        maxTokens: body.max_tokens,
    };

    // Planning model: smaller/faster model.
    const planningModel: ModelConfig = {
        name: EXPERT_FAST_MODEL,
        provider: 'ollama',
        temperature: body.temperature,
        maxTokens: body.max_tokens,
    };

    // Verifier & revision model: small reviewer model.
    const verifierModel: ModelConfig = {
        name: EXPERT_REVIEWER_MODEL,
        provider: 'ollama',
        temperature: 0,
        maxTokens: 256,
    };

    const revisionModel: ModelConfig = verifierModel;

    const provider: LLMProvider = {
        name: 'ollama',
        async chat(request) {
            const content = await ollamaChat(
                request.model,
                request.messages,
                { stream: request.stream }
            );

            return {
                content,
                model: request.model.name,
                finishReason: 'stop',
            };
        },
    };

    const escalationPolicy: EscalationPolicy = {
        maxAttempts: 2,
        ladder: [initialModel],
    };

    const result = await runRecursiveSession(
        prompt,
        [],
        {
            provider,
            initialModel,
            verifierModel,
            escalationPolicy,
            maxRetries: 2,
            minConfidence: 0.75,
            planningModel,
            revisionModel,
        }
    );

    logger.info('chat.completion.handle.result', {
        requestedModel: rawModel,
        clientModelId,
        ollamaModelName,
        model: result.model,
        attempts: result.attempts,
        confidence: result.confidence,
    });

    const apiResponse: ChatCompletionResponse = {
        id: `chatcmpl-${randomUUID()}`,
        object: 'chat.completion',
        created: Math.floor(Date.now() / 1000),
        // Echo back the model string the client sent so tooling sees the same id it requested.
        model: rawModel,
        choices: [
            {
                index: 0,
                message: {
                    role: 'assistant',
                    content: result.content,
                },
                finish_reason: 'stop',
            },
        ],
    };

    console.log('apiResponse: ', apiResponse);

    return apiResponse;
}

function buildPromptFromMessages(messages: ChatMessage[]): string {
    return messages
        .map((message) => `${message.role.toUpperCase()}: ${message.content}`)
        .join('\n');
}

/**
 * Registers OpenAI-compatible chat completion routes
 */
export function registerChatRoutes(app: Express): void {
    app.post(
        '/v1/chat/completions',
        async (
            request: Request<{}, {}, ChatCompletionRequest>,
            response: Response
        ) => {
            const body = request.body;

            if (!body || !body.messages || body.messages.length === 0) {
                return response.status(400).json({
                    error: {
                        message: 'Invalid request: messages are required',
                        type: 'invalid_request_error',
                    },
                });
            }

            const start = Date.now();

            try {
                const apiResponse = await handleChatCompletion(body);

                logger.info(
                    'chat.completion.success',
                    {
                        latencyMs: Date.now() - start,
                    }
                );

                console.log(
                    'API response for chat.completions:',
                    JSON.stringify(apiResponse)
                );

                // Always send streaming format for Copilot Chat compatibility
                response.setHeader('Content-Type', 'text/event-stream');
                response.setHeader('Cache-Control', 'no-cache');
                response.setHeader('Connection', 'keep-alive');

                const firstChoice = apiResponse.choices[0];

                // First chunk: role only
                const roleChunk = {
                    id: apiResponse.id,
                    object: 'chat.completion.chunk',
                    created: apiResponse.created,
                    model: apiResponse.model,
                    choices: [
                        {
                            index: firstChoice.index,
                            delta: {
                                role: firstChoice.message.role,
                            },
                            finish_reason: null,
                        },
                    ],
                };

                // Second chunk: content with finish_reason
                const contentChunk = {
                    id: apiResponse.id,
                    object: 'chat.completion.chunk',
                    created: apiResponse.created,
                    model: apiResponse.model,
                    choices: [
                        {
                            index: firstChoice.index,
                            delta: {
                                content: firstChoice.message.content,
                            },
                            finish_reason: 'stop',
                        },
                    ],
                };

                response.write(`data: ${JSON.stringify(roleChunk)}\n\n`);
                response.write(`data: ${JSON.stringify(contentChunk)}\n\n`);
                response.write('data: [DONE]\n\n');
                response.end();

                return;

                return response.json(apiResponse);
            }
            catch (err: any) {
                logger.error('chat.completion.failed', err);

                return response.status(500).json({
                    error: {
                        message: err?.message ?? 'Internal server error',
                        type: 'internal_error',
                    },
                });
            }
        }
    );
}
