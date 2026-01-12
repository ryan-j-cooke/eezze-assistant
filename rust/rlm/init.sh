#!/usr/bin/env bash
set -e

# Root files
touch package.json
touch tsconfig.json
touch README.md
touch .gitignore
touch .env.example

# Source root
mkdir -p src

# API layer (OpenAI-compatible surface)
mkdir -p src/api
touch src/api/chat.rs
touch src/api/models.rs
touch src/api/index.rs

# HTTP server
mkdir -p src/server
touch src/server/http.rs
touch src/server/routes.rs

# Orchestrator (recursive reasoning core)
mkdir -p src/orchestrator
touch src/orchestrator/loop.rs
touch src/orchestrator/plan.rs
touch src/orchestrator/revise.rs
touch src/orchestrator/escalate.rs
touch src/orchestrator/confidence.rs
touch src/orchestrator/types.rs

# LLM adapters
mkdir -p src/llm
touch src/llm/ollama.rs
touch src/llm/verifier.rs
touch src/llm/types.rs

# Index / memory layer
mkdir -p src/index
touch src/index/retrieve.rs
touch src/index/embed.rs
touch src/index/store.rs
touch src/index/types.rs

# Prompt templates
mkdir -p src/prompts
touch src/prompts/plan.rs
touch src/prompts/revise.rs
touch src/prompts/verify.rs

# Utilities
mkdir -p src/utils
touch src/utils/hash.rs
touch src/utils/logger.rs
touch src/utils/stream.rs
touch src/utils/timer.rs

# Shared types
mkdir -p src/types
touch src/types/openai.rs
touch src/types/chat.rs
touch src/types/config.rs

echo "✅ TypeScript project structure initialized"
