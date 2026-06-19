# Candor AI — Agent Context

## Overview

**Candor AI** is a production-grade Rust-based agentic operating system with voice, memory, PDA capabilities, and a pluggable skill ecosystem. Implements a 7-phase cognitive loop (Perceive → Reason → Plan → Act → Reflect → Learn → Evolve).

## Tech Stack

- **Language:** Rust (edition 2024)
- **Build System:** Cargo workspace with 11 crates
- **Architecture:** Multi-crate modular (core, graph, sandbox, cognitive, memory, sentinel, orchestrator, tools, mcp, telemetry)
- **Desktop:** Desktop application support
- **Testing:** Rust test framework

## Repository Structure

```
crates/
├── candor-core/          # Core types, traits, primitives
├── candor-graph/         # Knowledge graph implementation
├── candor-sandbox/       # Secure code execution sandbox
├── candor-cognitive/     # Cognitive loop (7-phase reasoning)
├── candor-memory/        # Long-term memory persistence
├── candor-sentinel/      # Security monitoring & guardrails
├── candor-orchestrator/  # Agent orchestration & task routing
├── candor-tools/         # Tool definitions & execution
├── candor-mcp/           # MCP protocol integration
└── candor-telemetry/     # Observability & metrics
bin/candor-ai/            # CLI binary entry point
tests/                    # Integration tests
```

## Key Commands

- `cargo build` — Build all crates
- `cargo test` — Run all tests
- `cargo clippy` — Lint checks
- `cargo fmt` — Format code
- `make install` — Build and install CLI binary

## Architecture

- **CLI Binary** (`bin/candor-ai`): Entry point with subcommands (task, chat, voice, pda, health)
- **7-Phase Loop**: Perceive → Reason → Plan → Act → Reflect → Learn → Evolve
- **PDA System**: Persistent identity, memory, and session management via `~/.candor/`
- **Voice Stack**: Whisper.cpp (STT) + Piper TTS (speech synthesis)
- **Tool System**: Pluggable tool registry with sandboxed execution
- **MCP Integration**: Model Context Protocol for external tool calls

## Quality Gates

- `cargo test --workspace`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`
