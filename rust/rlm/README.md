# Recursive LLM (Rust implementation)

This crate is a Rust implementation of the local recursive LLM server that mirrors the existing TypeScript app. It exposes an OpenAI‑compatible HTTP surface (`/v1/chat/completions`, `/v1/embeddings`, `/v1/models`) backed by:

- A recursive orchestrator loop
- An Ollama LLM adapter
- Embedding generation via `nomic-embed-text`
- Optional in‑memory embedding store and retrieval helpers

The goal is that the **file layout and responsibilities match the TypeScript version**, so you can debug behaviour in either language in the same place.

## Running

From this directory:

```bash
cargo run
```

The server will start on port `4000` by default (configurable via `config.json`). It will first check that required Ollama models are available.

Key endpoints:

- `GET /health` – basic health check
- `GET /v1/models` – OpenAI‑style models list
- `POST /v1/embeddings` – embeddings API
- `POST /v1/chat/completions` – OpenAI‑compatible chat completions (SSE stream)

## File structure

### Root

- **`Cargo.toml`** – Rust crate manifest and dependencies (tokio, axum, reqwest, serde, async‑trait, etc.).
- **`src/main.rs`** – Entry point. Mirrors `src/main.ts` in TypeScript.
  - Loads runtime config (or defaults).
  - Checks required Ollama models via `/api/show`.
  - Constructs listener and starts the HTTP server from `server::http::create_server`.
  - Wires graceful shutdown on SIGINT/SIGTERM.

### HTTP server

- **`src/server/mod.rs`** – Module declarations for the HTTP server.
- **`src/server/http.rs`** – Equivalent of `src/server/http.ts`.
  - Builds an Axum `Router`.
  - Registers `/health`.
  - Calls `server::routes::register_api_routes` to attach API routes.
- **`src/server/routes.rs`** – Equivalent of `src/api/index.ts`.
  - `register_api_routes(app)` calls:
    - `api::models::register_model_routes`
    - `api::chat::register_chat_routes`
    - `api::index::register_embedding_routes`

### API layer (`src/api`)

- **`src/api/chat.rs`** – Mirrors `src/api/chat.ts`.
  - `handle_chat_completion` builds a prompt from messages.
  - Constructs `ModelConfig` for initial and verifier models.
  - Uses an `OllamaProvider` (LLM adapter) and `run_orchestrator` to run the recursive loop.
  - Builds an OpenAI‑style `ChatCompletionResponse`.
  - `register_chat_routes` exposes `POST /v1/chat/completions` and streams SSE chunks:
    - First chunk: role only (`chat.completion.chunk`).
    - Second chunk: content with `finish_reason: "stop"`.
    - Final `data: [DONE]` line.

- **`src/api/models.rs`** – Mirrors `src/api/models.ts`.
  - `register_model_routes` exposes `GET /v1/models` and returns a single local model (`qwen2.5:3b`) in OpenAI format.

- **`src/api/index.rs`** – Mirrors the embeddings handler in `src/api/index.ts`.
  - `register_embedding_routes` exposes `POST /v1/embeddings`.
  - Validates `input` (string or string array).
  - Uses `index::embed::handle_embeddings` to call Ollama’s `/api/embeddings` endpoint.
  - Returns an `OpenAIEmbeddingResponse` (`object: "list"`, `data: [...]`, `model`).

### Types (`src/types`)

- **`src/types/chat.rs`** – Mirrors `src/types/chat.ts`.
  - `ChatMessage`, `ChatCompletionRequest`, `ChatCompletionChoice`, `ChatCompletionResponse`.
- **`src/types/openai.rs`** – Mirrors `src/types/openai.ts`.
  - `OpenAIError`, `OpenAIEmbeddingRequest`, `OpenAIEmbedding`, `OpenAIEmbeddingResponse`, and streaming chunk types.
- **`src/types/config.rs`** – Mirrors `src/types/config.ts`.
  - `RuntimeConfig`, `ServerConfig`, `ModelConfig`, `ModelProvider`, `OrchestrationConfig`, `IndexConfig`.
  - Shapes match the JSON used by both Rust and TypeScript.
- **`src/types/mod.rs`** – Re‑exports the type modules.

### LLM adapter (`src/llm`)

- **`src/llm/types.rs`** – Mirrors `src/llm/types.ts`.
  - `LLMChatRequest`, `LLMChatResponse`, and `LLMProvider` trait (`async_trait`).
- **`src/llm/ollama.rs`** – Mirrors `src/llm/ollama.ts`.
  - `ollama_chat` sends JSON to `http://localhost:11434/api/chat`.
  - Logs request/response metadata.
  - Validates response and returns assistant content.
- **`src/llm/verifier.rs`** – Mirrors `src/llm/verifier.ts`.
  - `VerificationRequest`, `VerificationResult`.
  - `verify_with_llm` builds verification messages, calls the provider, and parses a JSON verdict from the model output.

### Orchestrator (`src/orchestrator`)

- **`src/orchestrator/loop.rs`** – Mirrors `src/orchestrator/loop.ts`.
  - `run_orchestrator` runs the recursive reasoning + verification loop.
  - Repeatedly:
    - Calls the main model via `LLMProvider`.
    - Calls the verifier via `verify_with_llm`.
    - Combines confidence and decides whether to accept, retry, or escalate.
- **`src/orchestrator/confidence.rs`** – Mirrors `confidence.ts`.
  - `ConfidenceInputs`, `combine_confidence`, `is_confidence_acceptable`, `should_escalate`.
- **`src/orchestrator/escalate.rs`** – Mirrors `escalate.ts`.
  - `EscalationState`, `EscalationPolicy`, `can_escalate`, `escalate_model`.
- **`src/orchestrator/plan.rs`** – Mirrors `plan.ts`.
  - `generate_plan` uses a planning model and `PLAN_PROMPT` to produce a high‑level execution plan.
- **`src/orchestrator/revise.rs`** – Mirrors `revise.ts`.
  - `RevisionRequest`, `RevisionResult`.
  - `revise_response` builds a revision prompt with context + reviewer notes and asks a model to rewrite the answer.
- **`src/orchestrator/types.rs`** – Mirrors `types.ts`.
  - `OrchestratorAttempt`, `OrchestratorContext`, `OrchestratorConfig`, `OrchestratorResult`.
- **`src/orchestrator/mod.rs`** – Module exports (`r#loop`, `plan`, `revise`, etc.).

### Prompts (`src/prompts`)

- **`src/prompts/plan.rs`** – `PLAN_PROMPT` string (planning instructions).
- **`src/prompts/revise.rs`** – `REVISE_PROMPT` string (revision instructions).
- **`src/prompts/verify.rs`** – `VERIFY_PROMPT` string (verification instructions).
- **`src/prompts/mod.rs`** – Module exports.

### Index / embeddings (`src/index`)

- **`src/index/embed.rs`** – Mirrors `src/index/embed.ts`.
  - `handle_embeddings` calls Ollama’s `/api/embeddings`.
  - Normalizes vectors and returns `EmbedResult { vector, dimensions, model }`.
- **`src/index/types.rs`** – `EmbedResult` definition.
- **`src/index/store.rs`** – Mirrors `src/index/store.ts` but uses an in‑memory store.
  - `EmbeddingStore` with `upsert`, `query`, `delete`, `clear`.
  - Internal `cosine_similarity` implementation.
- **`src/index/retrieve.rs`** – Mirrors key behaviour from `src/index/retrieve.ts`.
  - `chat` helper that wraps a single request through `run_orchestrator`.
  - `review_response` that uses embeddings to compute semantic similarity.
- **`src/index/mod.rs`** – Module exports.

### Utilities (`src/utils`)

- **`src/utils/logger.rs`** – JSON logger similar to TS `utils/logger.ts`.
  - `debug`, `info`, `warn`, `error` log functions.
- **`src/utils/hash.rs`** – Mirrors `utils/hash.ts`.
  - `hash_string`, `hash_strings` implemented with SHA‑256.
- **`src/utils/stream.rs`** – SSE utilities.
  - `format_sse_event(data)` produces `data: ...\n\n` lines for streaming responses.
- **`src/utils/timer.rs`** – Mirrors `utils/timer.ts`.
  - `Timer` and `measure` helper to time async operations.
- **`src/utils/mod.rs`** – Module exports.

---

Because this Rust crate mirrors the TypeScript project structure, you can usually find the Rust equivalent of a TS file by matching its path under `src/` (e.g. `src/api/chat.ts` ↔ `src/api/chat.rs`, `src/orchestrator/loop.ts` ↔ `src/orchestrator/loop.rs`).