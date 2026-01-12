import { LLMProvider } from '../llm/types';
import { ChatMessage } from '../types/chat';
import { ModelConfig } from '../types/config';

export interface VerificationRequest {
    prompt: string;
    response: string;
    context: string[];
}

export interface VerificationResult {
    approved: boolean;
    confidence: number;
    notes?: string;
}

/**
 * Run a verification pass using a small, strict LLM.
 * This verifier is expected to be deterministic.
 */
export async function verifyWithLLM(
    provider: LLMProvider,
    model: ModelConfig,
    request: VerificationRequest
): Promise<VerificationResult> {
    const messages = buildVerifierMessages(request);

    const result = await provider.chat({
        model,
        messages,
        stream: false,
    });

    return parseVerifierOutput(result.content);
}

/**
 * Construct a strict verification prompt.
 */
function buildVerifierMessages(request: VerificationRequest): ChatMessage[] {
    return [
        {
            role: 'system',
            content:
                'You are a strict verifier. Your job is to approve or reject answers. ' +
                'You must be conservative and reject if unsure.',
        },
        {
            role: 'user',
            content: `
USER PROMPT:
${request.prompt}

MODEL RESPONSE:
${request.response}

REFERENCE CONTEXT:
${request.context.map((c, i) => `[${i + 1}] ${c}`).join('\n')}

TASK:
1. Is the response correct?
2. Is it fully supported by the reference context?
3. Does it avoid speculation or fabrication?

Respond ONLY with valid JSON in the following format:
{
  "approved": boolean,
  "confidence": number,
  "notes": string
}
            `.trim(),
        },
    ];
}

/**
 * Parse verifier output safely.
 */
function parseVerifierOutput(text: string): VerificationResult {
    try {
        const start = text.indexOf('{');
        const end = text.lastIndexOf('}');

        if (start === -1 || end === -1) {
            throw new Error('No JSON block found');
        }

        const parsed = JSON.parse(text.slice(start, end + 1));

        return {
            approved: Boolean(parsed.approved),
            confidence:
                typeof parsed.confidence === 'number'
                    ? clamp(parsed.confidence, 0, 1)
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
            notes: 'Verifier output could not be parsed',
        };
    }
}

/**
 * Clamp confidence values.
 */
function clamp(
    value: number,
    min: number,
    max: number
): number {
    return Math.max(min, Math.min(max, value));
}
