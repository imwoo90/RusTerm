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
RusTerm is a high-performance serial terminal and log analysis web application built with Rust and Dioxus 0.7:

1. **`src/worker/`**:
   - Web Worker thread handling serial port data streams, VT100 ANSI parsing, and chunk dispatching.
   - Refer to [`LogChunk`](crate::worker::LogChunk) for streaming packet structures.
2. **`src/worker/repository/`**:
   - High-throughput storage engine backing log history using Origin Private File System (OPFS) and indexed search offsets.
3. **`src/utils/`**:
   - Cohesive helper utilities for device discovery (`device_id`), hex/text conversions (`format`), command history, and ANSI decode tables.
4. **`src/gui/` & `src/views/`**:
   - Reactive UI components powered by Dioxus 0.7 signals and RSX.

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
1. **Entry Points**: Start with `AGENTS.md` (this file) and `README.md` to grasp project architecture and verification requirements.
2. **Module Indexing**: Treat `mod.rs` in any directory as the module's architecture map. Read the `//!` header to understand submodules and dependencies before diving into child files.
3. **Graph Traversal via Rustdoc Links**: Follow compile-checked intra-doc links (`[Type]`) to traverse dependencies deterministically.
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
