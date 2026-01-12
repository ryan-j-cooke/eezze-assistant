# overview-intent.md

This document provides a **high-level architectural and intent
overview** of the project.\
It is designed for **humans and AI agents** to quickly understand *what
each file does* and *why it exists*.

This system implements a **local-first recursive LLM orchestration
architecture** with verification, confidence scoring, and controlled
escalation.

------------------------------------------------------------------------

## Root

### tsconfig.json

TypeScript compiler configuration.\
Targets Node.js ESM, strict typing, and clean separation between `src/`
and `dist/`.

------------------------------------------------------------------------

## src/

### main.ts

Application entry point.\
Bootstraps configuration, initializes core components, and starts the
server or runtime loop.

------------------------------------------------------------------------

## src/api/

### chat.ts

High-level chat handler.\
Acts as a bridge between incoming API requests and the orchestrator
loop.

### index.ts

API module entry point.\
Exports and wires API routes together.

### models.ts

Defines API-facing model configuration and selection logic.

------------------------------------------------------------------------

## src/index/ (Retrieval & Memory)

### embed.ts

Embedding generation interface.\
Responsible for converting text into vectors using a local embedding
model.

### retrieve.ts

Retrieval logic.\
Fetches relevant context from the embedding store for grounding and
verification.

### store.ts

Persistent embedding store (SQLite-backed).\
Handles insert, query, and delete operations for embeddings.

### types.ts

Shared types for indexing, retrieval, and embedding records.

------------------------------------------------------------------------

## src/llm/ (LLM Transport Layer)

### ollama.ts

LLM transport implementation for Ollama.\
Handles HTTP communication with locally running models.

### types.ts

Abstract LLM interfaces and contracts.\
Defines provider-agnostic request/response shapes.

### verifier.ts

LLM-based verifier.\
Uses a small, deterministic model to judge correctness, grounding, and
hallucinations.

------------------------------------------------------------------------

## src/orchestrator/ (Reasoning Control)

### confidence.ts

Confidence aggregation logic.\
Combines verifier, embedding, and model confidence signals.

### escalate.ts

Model escalation policy.\
Determines when and how to move to a larger model.

### loop.ts

Core recursive reasoning loop.\
Runs generate → verify → score → retry/escalate → terminate.

### plan.ts

Planning logic.\
Breaks complex tasks into structured reasoning steps.

### revise.ts

Revision logic.\
Improves or corrects previous model outputs based on verifier feedback.

### types.ts

Shared orchestrator state and control-flow types.

------------------------------------------------------------------------

## src/prompts/

### plan.ts

Prompt templates for planning phase.

### revise.ts

Prompt templates for revision and correction.

### verify.ts

Prompt templates for verification and judgment.

------------------------------------------------------------------------

## src/server/

### http.ts

Low-level HTTP server setup.\
Responsible for request handling, streaming, and lifecycle management.

### routes.ts

Defines OpenAI-compatible API routes\
(e.g. `/v1/chat/completions`, `/v1/embeddings`).

------------------------------------------------------------------------

## src/types/

### chat.ts

Internal chat message and role definitions.

### config.ts

Global configuration types.

### openai.ts

OpenAI-compatible request and response schemas.

------------------------------------------------------------------------

## src/utils/

### hash.ts

Deterministic hashing utilities (IDs, cache keys).

### logger.ts

Structured logging utilities.

### stream.ts

Streaming helpers for token-by-token responses.

### timer.ts

Timing and performance measurement utilities.

------------------------------------------------------------------------

## Design Intent Summary

-   **Small models generate**
-   **Smaller models verify**
-   **Embeddings ground truth**
-   **Confidence gates output**
-   **Escalation is explicit and bounded**

This project prioritizes: - Determinism over cleverness - Verification
over confidence - Cost-aware local execution - Clear reasoning
boundaries

The system is designed to be portable to **Rust**, **WASM**, and
**single-binary deployment** in the future.
