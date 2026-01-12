# 📝 Project TODOs

## 1️⃣ RAG / Retrieval
- [ ] Implement **vector storage & retrieval** in `index/store.ts`
  - Store embeddings with IDs / metadata
  - Add a `search(queryVector: number[], topK: number)` function
- [ ] Implement **cosine similarity search** or simple vector comparison
- [ ] Create `index/retrieve.ts` (or extend `store.ts`) to return **top-K relevant documents** for a query
- [ ] Integrate retrieved context into `runOrchestrator` via `context: string[]`

## 2️⃣ Orchestrator / Loop
- [ ] Ensure `runOrchestrator` fully consumes context from RAG
- [ ] Verify **confidence / verifier integration** works with retrieved context
- [ ] Test **escalation logic** with different model tiers
- [ ] Optional: implement **streaming support** for outputs

## 3️⃣ API / HTTP Layer
- [ ] Ensure `api/chat.ts` exports `handleChatCompletion` matching `runOrchestrator` API
- [ ] Remove any **Express route code** to avoid conflicts
- [ ] Ensure `/v1/embeddings` route calls `handleEmbeddings`
- [ ] Add proper **error handling / input validation**

## 4️⃣ Configuration
- [ ] Implement `loadConfig()` to load runtime config
- [ ] Optionally support environment variable overrides for deployment
- [ ] Validate model configs, ports, and RAG options

## 5️⃣ Models / Ollama
- [ ] Pull missing small-to-mid models:
  - `qwen2.5:0.5b`
  - `tinyllama:1.1b`
  - `qwen2.5:3b`
  - `llama3.2:3b`
  - `phi3.5:latest`
- [ ] Ensure existing models are mapped correctly to **planning / generation / escalation / verification** roles

## 6️⃣ Utilities
- [ ] Confirm `embed.ts` type-safe (normalize + key hashing)
- [ ] Verify `logger.ts` works across all modules
- [ ] Test `timer.ts` and `stream.ts` if you plan streaming responses

## 7️⃣ Optional / Future
- [ ] Persist RAG index to disk or a lightweight DB
- [ ] Implement **document ingestion pipeline** (for multi-doc RAG)
- [ ] Add **unit tests** for embeddings, retrieval, orchestrator
- [ ] Optimize orchestrator for **low-latency loops** using ultra-small models first
