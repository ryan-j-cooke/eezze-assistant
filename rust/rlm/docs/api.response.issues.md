## IMPORTANT: Embedding API response contract

This system relies on **Ollama's `/api/embeddings` endpoint** and a thin wrapper on both the TypeScript and Rust sides. When this contract is broken, you will see errors like:

- `Embedding request failed (500)`
- `Invalid embedding response from Ollama`
- `llm embedding error: the input length exceeds the context length`

To keep everything working, **both the request and response must match the expected shapes below.**

### 1. Request shape to Ollama `/api/embeddings`

The Rust server (`src/index/embed.rs`) and the original TypeScript code (`src/index/embed.ts`) both assume the same minimal request body to Ollama:

```jsonc
{
  "model": "nomic-embed-text",   // or another embedding model name
  "input": "<text to embed>"     // single string, not an array
}
```

Notes:

- The field **must be called `input`**, not `prompt`.
- We send a **single string**, not an array. If you need multiple embeddings, call the endpoint multiple times.
- Very long strings can exceed the model's context. The Rust wrapper now truncates overly long text on a character boundary before sending it, to avoid Ollama returning `input length exceeds the context length`.

### 2. Expected response shape from Ollama

Ollama is expected to return a JSON object compatible with this shape (both in TS and Rust):

```jsonc
{
  "embedding": [0.1, 0.2, ...],   // array<number>
  "model": "nomic-embed-text"    // optional string
}
```

The **critical requirement** is that `embedding` exists and is an array (possibly empty). The Rust side **no longer treats an empty array as an error**; it only fails when:

- The HTTP status is non-2xx, or
- The body cannot be parsed as the above structure.

This matches the original TypeScript behavior in `src/index/embed.ts`, which only rejected responses when `response.ok` was false or `embedding` was missing / not an array.

### 3. OpenAI-style `/v1/embeddings` contract (Node API)

Separately from Ollama, the Node API exposes an OpenAI-style endpoint at `/v1/embeddings` (`src/api/index.ts`). It expects:

**Request:**

```jsonc
{
  "input": "text" | ["text1", "text2", ...],
  "model": "<optional embedding model name>"
}
```

**Response:**

```jsonc
{
  "object": "list",
  "data": [
    { "object": "embedding", "embedding": number[], "index": 0 },
    { "object": "embedding", "embedding": number[], "index": 1 }
  ],
  "model": "<resolved embedding model name>"
}
```

Internally, this endpoint fans out to the Ollama `/api/embeddings` endpoint described above via `handleEmbeddings`.

### 4. Things that will break this pipeline

Avoid the following changes, as they will reintroduce hard-to-debug 500s:

- Renaming `input` back to `prompt` in the Ollama request body.
- Changing the Ollama response shape so that `embedding` is not present or not an array.
- Adding strict checks that reject empty `embedding` arrays (TS never did this; Rust previously did and caused `Invalid embedding response` errors).

If you need to change any of this behavior, **update both**:

- Rust: `src/index/embed.rs`
- TypeScript: `src/index/embed.ts` and `src/api/index.ts`

and re-run integration tests that exercise `/v1/chat/completions` and `/v1/embeddings` to ensure the orchestrator + verifier + embedding pipeline still succeeds.