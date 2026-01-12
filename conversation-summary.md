# Conversation Summary – Local Recursive LLM Orchestrator

## High-Level Goal

Design and prototype a **local-first Recursive Language Model (RLM) orchestration system** optimized for:

- Ultra-small LLMs (speed, determinism)
- Reviewer-based verification
- Recursive revision loops
- OpenAI-compatible local API
- VS Code / Cursor / Continue integration
- Clean migration path from TypeScript → Rust

This system is **not a chatbot**, but a reasoning pipeline.

---

## Key Topics Covered

### 1. Recursive Language Models (RLMs)
- Clarified distinction between standard LLMs and RLM-style systems
- RLM = orchestration + verification + recursion, not just model size
- Discussed ultra-small models (sub‑1B, even ~7M experimental research models)
- Determined Ollama can be used as a backend, but RLM behavior is achieved in orchestration logic

---

### 2. Architecture Decisions

**Core Principles**
- Small models generate
- Smaller models verify
- Larger models escalate only when necessary
- Deterministic prompts, no chain-of-thought leakage
- Binary approval gates

**Pipeline**
```
Plan → Generate → Verify → Revise → Escalate → Confidence → Return
```

---

### 3. Technology Stack

**Prototype**
- TypeScript
- Node.js (raw `http`, no framework)
- Ollama as local model runner
- OpenAI-compatible REST API

**Future**
- Rust binary (Axum / Tokio)
- Identical architecture, minimal impedance mismatch

---

### 4. Project Structure (Final)

```
src/
├── api/                # OpenAI-compatible handlers
├── index/              # Embeddings + retrieval
├── llm/                # Ollama + verifier adapters
├── orchestrator/       # Core RLM logic
├── prompts/            # Deterministic system prompts
├── server/             # HTTP + routing
├── types/              # Stable contracts
├── utils/              # Infra utilities
└── main.ts             # Entry point
```

---

### 5. Orchestrator Design

**Key Concepts**
- Attempts are immutable records
- Verification is binary (APPROVED / REJECTED)
- Confidence is separate from approval
- Escalation is policy-driven
- Revision is surgical, not regenerative

**Important Files**
- `orchestrator/loop.ts`
- `orchestrator/escalate.ts`
- `orchestrator/confidence.ts`
- `orchestrator/revise.ts`
- `orchestrator/types.ts`

---

### 6. Prompt Philosophy

**Plan Prompt**
- Produces execution strategy, not answers

**Verify Prompt**
- Strict gatekeeper
- No fixing, no reasoning, no prose

**Revise Prompt**
- Fixes rejected output
- Explicitly addresses reviewer feedback

These prompts enable **true recursive behavior** even with very small models.

---

### 7. API Compatibility

Implemented OpenAI-style endpoints:

- `POST /v1/chat/completions`
- `POST /v1/embeddings`
- `GET /health`

This allows seamless integration with:
- VS Code AI features
- Cursor
- Continue
- Any OpenAI-compatible client

---

### 8. Infrastructure Utilities

- `logger.ts` → structured JSON logging
- `stream.ts` → SSE streaming with `[DONE]` sentinel
- `hash.ts` → deterministic SHA‑256 keys
- `timer.ts` → latency measurement

All utilities are:
- Dependency-free
- Deterministic
- Rust-portable

---

### 9. Documentation for AI Agents

Created:
- `overview-intent.md` – file-by-file intent map
- This document (`conversation.md`) – architectural and design rationale

These are intended to:
- Ground future AI agents
- Prevent architectural drift
- Speed up refactors and ports

---

## Final Outcome

By the end of the conversation, the project achieved:

- A complete RLM orchestration skeleton
- OpenAI-compatible local server
- Deterministic prompts and verification
- Ultra-small-model-first philosophy
- Clean migration path to Rust
- AI-readable documentation for continuity

This system is **infrastructure**, not a demo.

---

## Status

🟢 Architecture complete  
🟢 Type system complete  
🟢 Prompt system complete  
🟢 API surface complete  
🟡 Final orchestration glue (loop wiring) optional  
🟡 Rust port optional
