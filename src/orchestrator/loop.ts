import { LLMProvider } from '../llm/types';
import { logger } from '../utils/logger';

import {
    combineConfidence,
    isConfidenceAcceptable,
    shouldEscalate,
} from './confidence';
import {
    EscalationPolicy,
    EscalationState,
    escalateModel,
} from './escalate';
import { verifyWithLLM } from '../llm/verifier';
import { ChatMessage } from '../types/chat';
import { ModelConfig } from '../types/config';

export interface LoopOptions {
    provider: LLMProvider;
    initialModel: ModelConfig;
    verifierModel: ModelConfig;
    escalationPolicy: EscalationPolicy;
    maxRetries?: number;
    minConfidence?: number;
}

export interface LoopResult {
    content: string;
    model: string;
    confidence: number;
    attempts: number;
}

/**
 * Run a recursive reasoning + verification loop.
 */
export async function runOrchestrator(
    prompt: string,
    context: string[],
    options: LoopOptions
): Promise<LoopResult> {
    let state: EscalationState = {
        currentModel: options.initialModel,
        attempts: 0,
    };

    const maxRetries = options.maxRetries ?? 2;
    const minConfidence = options.minConfidence ?? 0.75;

    logger.debug('orchestrator.start', {
        promptLength: prompt.length,
        contextItems: context.length,
        initialModel: options.initialModel.name,
        maxRetries,
        minConfidence,
    });

    let lastResponse = '';
    let lastConfidence = 0;

    while (true) {
        state.attempts++;

        logger.debug('orchestrator.iteration', {
            attempt: state.attempts,
            model: state.currentModel.name,
        });

        const messages: ChatMessage[] = [
            {
                role: 'system',
                content:
                    'Answer the user prompt accurately and concisely using the provided context.',
            },
            {
                role: 'user',
                content: buildUserPrompt(prompt, context),
            },
        ];

        const completion = await options.provider.chat({
            model: state.currentModel,
            messages,
            stream: false,
        });

        logger.debug('orchestrator.model_completion', {
            attempt: state.attempts,
            model: state.currentModel.name,
            contentPreview: completion.content.slice(0, 160),
        });

        lastResponse = completion.content;

        logger.debug('orchestrator.verifier_call', {
            attempt: state.attempts,
            verifierModel: options.verifierModel.name,
        });

        const verdict = await verifyWithLLM(
            options.provider,
            options.verifierModel,
            {
                prompt,
                response: lastResponse,
                context,
            }
        );

        lastConfidence = combineConfidence({
            verifierConfidence: verdict.confidence,
        });

        logger.debug('orchestrator.verifier_result', {
            attempt: state.attempts,
            approved: verdict.approved,
            verifierConfidence: verdict.confidence,
            combinedConfidence: lastConfidence,
        });

        if (
            verdict.approved &&
            isConfidenceAcceptable(
                lastConfidence,
                minConfidence
            )
        ) {
            logger.info('orchestrator.accepted', {
                model: state.currentModel.name,
                attempts: state.attempts,
                confidence: lastConfidence,
            });

            return {
                content: lastResponse,
                model: state.currentModel.name,
                confidence: lastConfidence,
                attempts: state.attempts,
            };
        }

        if (
            shouldEscalate(lastConfidence) &&
            escalateModel(state, options.escalationPolicy)
        ) {
            const next = escalateModel(
                state,
                options.escalationPolicy
            );

            if (!next) {
                break;
            }

            logger.info('orchestrator.escalate', {
                fromModel: state.currentModel.name,
                toModel: next.name,
                attempts: state.attempts,
                confidence: lastConfidence,
            });

            state.currentModel = next;
            continue;
        }

        if (state.attempts >= maxRetries) {
            logger.warn('orchestrator.max_retries_reached', {
                attempts: state.attempts,
                lastConfidence,
                lastModel: state.currentModel.name,
            });
            break;
        }
    }

    logger.info('orchestrator.completed_without_accept', {
        model: state.currentModel.name,
        attempts: state.attempts,
        confidence: lastConfidence,
    });

    return {
        content: lastResponse,
        model: state.currentModel.name,
        confidence: lastConfidence,
        attempts: state.attempts,
    };
}

/**
 * Build the final user prompt with context.
 */
function buildUserPrompt(
    prompt: string,
    context: string[]
): string {
    if (context.length === 0) {
        return prompt;
    }

    return `
CONTEXT:
${context.map((c, i) => `[${i + 1}] ${c}`).join('\n')}

QUESTION:
${prompt}
    `.trim();
}
