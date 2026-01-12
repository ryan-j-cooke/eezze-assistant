import { ChatMessage } from '../types/chat';
import { PLAN_PROMPT } from '../prompts/plan';
import { LLMProvider } from '../llm/types';
import { ModelConfig } from '../types/config';

/**
 * Generate a high-level execution plan using a small planning model.
 */
export async function generatePlan(
    provider: LLMProvider,
    model: ModelConfig,
    userPrompt: string
): Promise<ChatMessage[]> {
    const messages: ChatMessage[] = [
        {
            role: 'system',
            content: PLAN_PROMPT
        },
        {
            role: 'user',
            content: userPrompt
        }
    ];

    const response = await provider.chat({
        model,
        messages
    });

    return [
        ...messages,
        {
            role: 'assistant',
            content: response.content
        }
    ];
}
