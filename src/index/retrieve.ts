import { ModelConfig } from '../types/config';
import { EXPERT_REVIEWER_SMALL } from '../../eezze.config';
import { handleEmbeddings } from './embed';
import { runOrchestrator } from '../orchestrator/loop';
import { ollamaChat } from '../llm/ollama';
import { LLMProvider } from '../llm/types';
import { ChatMessage } from './types';

export interface ReviewResult {
    approved: boolean;
    confidence: number;
    notes?: string;
}

// Minimal chat wrapper for a single request
export async function chat({
    model,
    messages,
}: {
    model: ModelConfig;
    messages: ChatMessage[];
}) {
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

    const result = await runOrchestrator(
        messages[messages.length - 1].content,
        messages.map((m) => m.content),
        {
            provider,
            initialModel: model,
            verifierModel: model,
            escalationPolicy: {
                maxAttempts: 1,
                ladder: [model],
            },
            maxRetries: 1,
            minConfidence: 0.75,
        }
    );

    return result.content;
}

/**
 * Review a model response using:
 * 1. A small verification model
 * 2. Embedding similarity against source context
 * 3. A structured yes/no gate
 */
export async function reviewResponse(
    prompt: string,
    response: string,
    context: string[],
    reviewerModel?: ModelConfig
): Promise<ReviewResult> {
    const model: ModelConfig =
        reviewerModel ?? {
            name: EXPERT_REVIEWER_SMALL,
            provider: 'ollama',
            temperature: 0,
            maxTokens: 256,
        };

    const reviewPrompt = buildReviewPrompt(prompt, response, context);

    const review = await chat({
        model,
        messages: [
            {
                role: 'system',
                content:
                    'You are a strict verifier. Approve only if the answer is grounded, correct, and does not hallucinate.',
            },
            {
                role: 'user',
                content: reviewPrompt,
            },
        ],
    });

    const parsed = parseReview(review);

    if (!parsed.approved) {
        return parsed;
    }

    const embeddingScore = await verifyWithEmbeddings(
        response,
        context
    );

    if (embeddingScore < 0.75) {
        return {
            approved: false,
            confidence: embeddingScore,
            notes: 'Low semantic similarity to provided context',
        };
    }

    return {
        approved: true,
        confidence: Math.min(
            1,
            (parsed.confidence + embeddingScore) / 2
        ),
    };
}

/**
 * Build a minimal, deterministic review prompt.
 */
function buildReviewPrompt(
    prompt: string,
    response: string,
    context: string[]
): string {
    return `
USER PROMPT:
${prompt}

MODEL RESPONSE:
${response}

REFERENCE CONTEXT:
${context.map((c, i) => `[${i + 1}] ${c}`).join('\n')}

TASK:
1. Is the response factually correct?
2. Is it fully supported by the reference context?
3. Does it avoid speculation?

Answer ONLY in JSON:
{
  "approved": boolean,
  "confidence": number (0.0 - 1.0),
  "notes": string
}
`.trim();
}

/**
 * Parse strict JSON review output.
 */
function parseReview(text: string): ReviewResult {
    try {
        const jsonStart = text.indexOf('{');
        const jsonEnd = text.lastIndexOf('}');

        if (jsonStart === -1 || jsonEnd === -1) {
            throw new Error('No JSON found');
        }

        const parsed = JSON.parse(
            text.slice(jsonStart, jsonEnd + 1)
        );

        return {
            approved: Boolean(parsed.approved),
            confidence:
                typeof parsed.confidence === 'number'
                    ? parsed.confidence
                    : 0,
            notes:
                typeof parsed.notes === 'string'
                    ? parsed.notes
                    : undefined,
        };
    } catch {
        return {
            approved: false,
            confidence: 0,
            notes: 'Failed to parse reviewer output',
        };
    }
}

/**
 * Embedding-based semantic verification.
 */
async function verifyWithEmbeddings(
    response: string,
    context: string[]
): Promise<number> {
    const responseEmbedding = await handleEmbeddings(response);

    let maxScore = 0;

    for (const chunk of context) {
        const contextEmbedding = await handleEmbeddings(chunk);

        const score = cosineSimilarity(
            responseEmbedding.vector,
            contextEmbedding.vector
        );

        maxScore = Math.max(maxScore, score);
    }

    return maxScore;
}

/**
 * Compute cosine similarity between two vectors.
 */
function cosineSimilarity(a: number[], b: number[]): number {
    let dot = 0;
    let normA = 0;
    let normB = 0;

    for (let i = 0; i < a.length; i++) {
        dot += a[i] * b[i];
        normA += a[i] * a[i];
        normB += b[i] * b[i];
    }

    if (normA === 0 || normB === 0) {
        return 0;
    }

    return dot / (Math.sqrt(normA) * Math.sqrt(normB));
}
