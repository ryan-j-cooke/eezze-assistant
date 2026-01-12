# Streaming LLM Thinking/Planning in VS Code Chat

## The Challenge
VS Code's Chat UI needs **real-time streaming chunks** to display the LLM's thinking, planning, and reasoning process as it happens.

## Current Implementation
Your `/v1/chat/completions` endpoint:
- Waits for the complete response from your orchestrator
- Sends it all at once in 2 SSE chunks (role + content)
- VS Code receives the full answer but can't show intermediate steps

## What You Need
Stream each step as it's generated:

### The Response Format
Each chunk must follow the OpenAI streaming format:
```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion.chunk",
  "created": 1234567890,
  "model": "model-name",
  "choices": [{
    "index": 0,
    "delta": {
      "content": "The thinking/planning text here"
    },
    "finish_reason": null
  }]
}