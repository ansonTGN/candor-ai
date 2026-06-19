# Candor AI Documentation

## Overview

Candor AI is a production-grade Rust-based agentic operating system with voice, memory, PDA capabilities, and a pluggable skill ecosystem. It implements a 7-phase cognitive loop (Perceive → Reason → Plan → Act → Reflect → Learn → Evolve).

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

## Key Concepts

- **7-Phase Cognitive Loop**: Perceive → Reason → Plan → Act → Reflect → Learn → Evolve
- **Sandboxed Execution**: Cross-platform process isolation, WASM execution, policy enforcement
- **Hook System**: Hooks at every pipeline stage for observability and customization
- **PDA System**: Persistent identity, memory, and session management via `~/.candor/`
- **Voice Stack**: Whisper.cpp (STT) + Piper TTS (speech synthesis)
- **Tool System**: Pluggable tool registry with sandboxed execution
- **MCP Integration**: Model Context Protocol for external tool calls

## Getting Started

```bash
# Build from source
cargo build --release

# Run CLI
cargo run -- chat

# Run all tests
cargo test --workspace
```

## Repository Structure

```
candor-ai/
├── bin/candor-ai/          # CLI entry point
├── crates/
│   ├── candor-core/        # Core types, traits, primitives
│   ├── candor-graph/       # Knowledge graph, state machine
│   ├── candor-sandbox/     # Secure execution sandbox
│   ├── candor-cognitive/   # 7-phase cognitive loop
│   ├── candor-memory/      # Long-term memory persistence
│   ├── candor-sentinel/    # Security monitoring, guardrails
│   ├── candor-orchestrator/ # Agent orchestration
│   ├── candor-tools/       # Tool definitions
│   ├── candor-mcp/         # MCP protocol integration
│   ├── candor-telemetry/   # Observability & metrics
│   └── candor-daemon/      # (planned)
└── desktop/                # Tauri desktop app
```

## Prerequisites

- Rust (edition 2024)
- Cargo workspace
- Tauri CLI (for desktop build)

## Quality Gates

```bash
cargo check --workspace    # Type check
cargo clippy -- -D warnings  # Lint
cargo test --workspace     # Run tests (320+)
cargo fmt --check          # Format check
```
