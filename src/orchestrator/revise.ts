import { LLMProvider } from '../llm/types';
import { ChatMessage } from '../types/chat';
import { ModelConfig } from '../types/config';

export interface RevisionRequest {
    originalPrompt: string;
    previousResponse: string;
    context: string[];
    reviewerNotes?: string;
}

export interface RevisionResult {
    content: string;
    model: string;
}

/**
 * Revise a previous model response using explicit feedback.
 * This function assumes the original attempt was rejected.
 */
export async function reviseResponse(
    provider: LLMProvider,
    model: ModelConfig,
    request: RevisionRequest
): Promise<RevisionResult> {
    const messages = buildRevisionMessages(request);

    const result = await provider.chat({
        model,
        messages,
        stream: false,
    });

    return {
        content: result.content,
        model: model.name,
    };
}

/**
 * Build a focused revision prompt.
 * This prompt is intentionally corrective, not open-ended.
 */
function buildRevisionMessages(request: RevisionRequest): ChatMessage[] {
    return [
        {
            role: 'system',
            content:
                'You are revising a previous answer that was rejected. ' +
                'Correct errors, remove unsupported claims, and strictly adhere to the provided context.',
        },
        {
            role: 'user',
            content: `
ORIGINAL QUESTION:
${request.originalPrompt}

PREVIOUS (REJECTED) RESPONSE:
${request.previousResponse}

${request.reviewerNotes ? `REVIEWER FEEDBACK:\n${request.reviewerNotes}\n` : ''}

REFERENCE CONTEXT:
${request.context.map((c, i) => `[${i + 1}] ${c}`).join('\n')}

TASK:
Rewrite the response so that it is:
- Factually correct
- Fully supported by the reference context
- Clear and concise
- Free of speculation

Return ONLY the revised answer text.
            `.trim(),
        },
    ];
}
