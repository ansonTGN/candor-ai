# Candor AI — Agent Context

## Overview

**Candor AI** is a production-grade Rust-based agentic operating system with voice, memory, PDA capabilities, and a pluggable skill ecosystem. Implements a 7-phase cognitive loop (Perceive → Reason → Plan → Act → Reflect → Learn → Evolve).

## Tech Stack

- **Language:** Rust (edition 2024)
- **Build System:** Cargo workspace with 11 crates
- **Desktop:** Tauri desktop app support
- **Testing:** cargo test (unit + integration, 320+ tests)
- **Linting:** cargo clippy
- **CI:** GitHub Actions (ci.yml, release.yml)

## Architecture

```
User Input → candor-orchestrator (7-phase ISA pipeline)
                ↓
         candor-graph (state machine + hooks)
                ↓
    ┌───────────┼───────────┐
    ↓           ↓           ↓
 candor-tools candor-mcp  candor-sandbox
 (fs/git/shell) (MCP bridge) (WASM/process)
    ↓           ↓           ↓
 candor-sentinel (rules, slop detection, doctrine)
    ↓
 candor-cognitive (embedding, backends)
    ↓
 candor-memory (schema, store)
    ↓
 candor-telemetry (metrics, logging)
```

## Key Patterns

- **7-Phase Cognitive Loop**: Perceive → Reason → Plan → Act → Reflect → Learn → Evolve
- **Sandboxed Execution**: Cross-platform process isolation, WASM execution, policy enforcement
- **Hook System**: Hooks at every pipeline stage for observability and customization
- **PDA System**: Persistent identity, memory, and session management via `~/.candor/`
- **Voice Stack**: Whisper.cpp (STT) + Piper TTS (speech synthesis)
- **Tool System**: Pluggable tool registry with sandboxed execution
- **MCP Integration**: Model Context Protocol for external tool calls
- **Error Recovery**: Retry policies, error escalation, checkpoint/restore via candor-graph

## Repository Structure

```
candor-ai/
├── bin/candor-ai/          # CLI entry point (task, chat, voice, pda, health)
├── crates/
│   ├── candor-core/        # Core types, traits, primitives, error, protocol
│   ├── candor-graph/       # Knowledge graph, state machine, hooks, recovery
│   ├── candor-sandbox/     # Secure execution sandbox, WASM, policies
│   ├── candor-cognitive/   # 7-phase cognitive loop, LLM backends, embedding
│   ├── candor-memory/      # Long-term memory persistence, schema, store
│   ├── candor-sentinel/    # Security monitoring, slop detection, guardrails
│   ├── candor-orchestrator/ # Agent orchestration, task routing, approval gate
│   ├── candor-tools/       # Tool definitions (fs, git, shell, search, test)
│   ├── candor-mcp/         # MCP protocol integration
│   ├── candor-telemetry/   # Observability & metrics
│   # candor-daemon — planned but not yet implemented
└── desktop/                # Tauri desktop app
```

## Key Commands

- `cargo check --workspace` — Type check all crates
- `cargo test --workspace` — Run all tests (320+)
- `cargo clippy --workspace -- -D warnings` — Lint check
- `cargo fmt --check` — Format check
- `cargo build` — Build all crates

## Quality Gates

- `cargo check --workspace` — 0 errors
- `cargo clippy --workspace -- -D warnings` — 0 warnings
- `cargo test --workspace` — all pass (320+ tests)
- `cargo fmt --check` — passes
