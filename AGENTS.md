# RusTerm Agent Guide & Standards (AGENTS.md)

This file defines the project-specific coding guidelines, architectural overview, and verification standards for RusTerm.

---

## 1. Architectural Baseline: Agent-Native Core
This project strictly adheres to the compiler-enforced **Agent-Native** architecture.

👉 **Mandatory Baseline Reading**: Consult [AGENT_NATIVE.md](file://AGENT_NATIVE.md) for the authoritative specification on:
- **Core Philosophy**: Living LLM-Wiki (`mod.rs` = `index.md`), compile-checked intra-doc links, and living doctests.
- **Compiler Hard Limits**: Rules 1-4 enforced via `build.rs` and `build_linter.rs` (file headers, logical code limits, inline test limits, function physical limits).
- **Anti-Code-Golfing Principle**: Decompose into cohesive submodules; never compress variable names or eliminate idiomatic whitespace.
- **Safety Lints**: `missing_docs = "deny"`, `clippy::unwrap_used = "deny"`, `clippy::expect_used = "deny"` in `Cargo.toml`.

---

## 2. System Architecture & Module Domains
RusTerm is a high-performance serial terminal and log analysis web application built with Rust and Dioxus 0.7. Its root crate index (`src/main.rs`) organizes the codebase into seven top-level domains:

1. **`src/components/` (`src/components/mod.rs`)**:
   - Reactive Dioxus 0.7 UI component tree including top-level layout (`rust_term.rs`), connection panel (`connection/mod.rs`), virtual-scrolled log monitor (`monitor/mod.rs`), interactive xterm.js bridge (`terminal/mod.rs`), and reusable UI controls (`ui/mod.rs`).
2. **`src/hooks/` (`src/hooks/mod.rs`)**:
   - Reactive state hooks bridging browser Web Serial hardware streams (`serial.rs`) and background Web Worker message channels (`worker.rs`).
3. **`src/worker/` (`src/worker/mod.rs`)**:
   - Dedicated background Web Worker handling high-throughput serial ingestion (`chunk_handler.rs`, `processor.rs`), command dispatch (`commands.rs`, `dispatcher.rs`), regex/substring search (`search.rs`), and Origin Private File System (OPFS) log storage (`repository/mod.rs` -> `index.rs`, `storage.rs`).
4. **`src/utils/` (`src/utils/mod.rs`)**:
   - Cohesive helper utilities for USB/Bluetooth device identification and alias persistence (`device_id/mod.rs`), hex/text formatting (`format.rs`), ANSI escape decoding (`ansi_decoder.rs`), and Web Serial API wrappers (`serial_api.rs`).
5. **`src/state.rs`**, **`src/types.rs`**, & **`src/config.rs`**:
   - Global reactive context signals (`AppState`), shared IPC message envelopes (`WorkerCommand`, `WorkerResponse`), and system-wide buffering/UI constants.

---

## 3. Verification Protocol
Always verify code changes before completing any task:

```bash
# 1. Check compiler linter rules (Agent-Native constraints)
cargo check

# 2. Run unit tests and integration tests
cargo test --all-targets

# 3. Verify Clippy lints and clean idioms
cargo clippy --all-targets

# 4. Verify doc link integrity and eliminate broken references
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# 5. Verify browser runtime behavior via Playwright E2E interactive suite
npm run test:e2e
```

---

## 4. AI Agent Navigation Guide
1. **Entry Points**: Start with `AGENTS.md` (this file), `AGENT_NATIVE.md`, and `src/main.rs` to grasp the project architecture and top-level module catalog.
2. **Module Indexing (`mod.rs` = `index.md`)**: Treat `src/main.rs` and every `mod.rs` as a directory's Living LLM-Wiki index. Read its `//!` header (`## Overview` and `## Submodules`)—which explicitly specifies whether each child is a leaf file (`foo.rs`) or directory index (`foo/mod.rs`)—before opening child files.
3. **Graph Traversal via Rustdoc Links**: Follow compile-checked intra-doc links (`[Type]`) to traverse dependencies deterministically without brute-force filesystem searches.
4. **Pre-commit Verification**: Never consider a task complete without passing all 5 steps in the Verification Protocol.

---

# Appendix: Dioxus 0.7 Reference Guide

You are working with [0.7 Dioxus](https://dioxuslabs.com/learn/0.7). Dioxus 0.7 changes every API in Dioxus. `cx`, `Scope`, and `use_state` are gone.

### UI with RSX
```rust
rsx! {
    div {
        class: "container",
        color: "red",
        width: if condition { "100%" },
        "Hello, Dioxus!"
    }
    for i in 0..5 {
        div { "{i}" }
    }
    if condition {
        div { "Condition is true!" }
    }
}
```

### Components
* Components are functions annotated with `#[component]`.
* Names must start with a capital letter or contain an underscore.
* Props must be owned (`String`, `Vec<T>`), and implement `PartialEq` and `Clone`. Wrap in `ReadOnlySignal` for reactive Copy props.

### State & Signals
* `use_signal` creates local component state.
* Call `my_signal()` to read / clone.
* Use `*my_signal.write() = ...` or `my_signal.with_mut(...)` to mutate.
* `use_memo` recalculates when read signals change.
* `use_context_provider` and `use_context` for subtree dependency injection.

### Async & Web
* In WASM client code, avoid server-only macros. Use standard Dioxus hooks, `spawn(...)`, and Web Worker messaging.
