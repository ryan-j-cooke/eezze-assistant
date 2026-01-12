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
import { generatePlan } from './plan';
import { reviseResponse } from './revise';
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

export interface RecursiveOptions extends LoopOptions {
    planningModel?: ModelConfig;
    revisionModel?: ModelConfig;
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
 * Higher-level recursive session that adds an explicit planning and revision phase
 * around the core orchestrator loop.
 */
export async function runRecursiveSession(
    prompt: string,
    context: string[],
    options: RecursiveOptions
): Promise<LoopResult> {
    const planningModel = options.planningModel ?? options.initialModel;
    const revisionModel = options.revisionModel ?? options.initialModel;
    const minConfidence = options.minConfidence ?? 0.75;

    // 1) Planning phase
    logger.debug('orchestrator.plan.start', {
        model: planningModel.name,
    });

    const planMessages = await generatePlan(
        options.provider,
        planningModel,
        prompt
    );

    const planMessage = planMessages[planMessages.length - 1];
    const enrichedContext: string[] = [
        ...context,
        `PLAN:\n${planMessage.content}`,
    ];

    logger.debug('orchestrator.plan.result', {
        model: planningModel.name,
        planPreview: planMessage.content.slice(0, 200),
    });

    // 2) Core orchestrator loop using the enriched context
    const loopResult = await runOrchestrator(
        prompt,
        enrichedContext,
        options
    );

    // 3) Final verification of the chosen answer
    const finalVerdict = await verifyWithLLM(
        options.provider,
        options.verifierModel,
        {
            prompt,
            response: loopResult.content,
            context: enrichedContext,
        }
    );

    const finalConfidence = combineConfidence({
        verifierConfidence: finalVerdict.confidence,
    });

    logger.debug('orchestrator.final_verifier_result', {
        approved: finalVerdict.approved,
        verifierConfidence: finalVerdict.confidence,
        combinedConfidence: finalConfidence,
    });

    if (
        finalVerdict.approved &&
        isConfidenceAcceptable(finalConfidence, minConfidence)
    ) {
        return {
            ...loopResult,
            confidence: finalConfidence,
        };
    }

    // 4) Revision phase if the final answer is still not acceptable
    logger.info('orchestrator.revise.start', {
        model: revisionModel.name,
    });

    const revision = await reviseResponse(
        options.provider,
        revisionModel,
        {
            originalPrompt: prompt,
            previousResponse: loopResult.content,
            context: enrichedContext,
            reviewerNotes: undefined,
        }
    );

    logger.info('orchestrator.revise.result', {
        model: revision.model,
    });

    return {
        content: revision.content,
        model: revision.model,
        confidence: finalConfidence,
        attempts: loopResult.attempts,
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
