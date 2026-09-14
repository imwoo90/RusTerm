# RusTerm Agent-Native Guide & Standards (AGENTS.md)

This file defines the coding guidelines, architectural constraints, and compilation standards for RusTerm. The codebase follows an **Agent-Native** design based on the **"Code as Documentation"** philosophy, optimized for high-efficiency collaboration between human developers and AI Agents.

---

## 1. Core Philosophy: Code as Documentation (LLM-Wiki)

To prevent documentation drift and maximize signal-to-noise ratio in LLM context windows, the Rust source code itself serves as the single source of truth for implementation, architecture, and documentation.

1. **`mod.rs` = `index.md`**:
   * Leveraging Rust's module tree, every directory's `mod.rs` acts as the `index.md` (table of contents & architecture catalog) for that module.
   * Use module-level doc comments (`//!`) at the top of `mod.rs` to detail high-level architectural responsibilities, data flows, and design decisions in Markdown.
2. **Compiler-Verified Documentation**:
   * Never write documentation that cannot be compiled. Structs, traits, and public functions must use standard doc comments (`///`) with executable code examples (`doctests`).
   * When running `cargo test`, the compiler executes all doctests, guaranteeing documentation never goes stale.
3. **Compile-Checked Intra-Doc Links**:
   * Reference other types, modules, or functions using Rust's native intra-doc link syntax (e.g., `[`[`LogChunk`](crate::worker::LogChunk)`]`).
   * Rustdoc validates every link at build time (`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`), eliminating broken links and hallucinations.

---

## 2. LLM-Agent Constraints (Enforced at Compile-Time via `build.rs` & `build_linter.rs`)

To keep files compact, modular, and optimized for LLM context windows, strict architectural limits are enforced during `cargo check`, `cargo build`, and `cargo test`:

1. **Rule 1: File-Level Living Wiki Header (Min 100 Characters)**:
   * Every non-test production `.rs` file must begin with a file-level doc comment (`//!`) of **at least 100 characters** describing its purpose, responsibilities, and architecture.
2. **Rule 2: Production Logical Code Limit (Max 10,000 Characters)**:
   * The total character count of active production code lines (excluding comments, doc comments, empty lines, and `#[cfg(test)]` blocks) must be **under 10,000 characters** (approx. 200–300 lines of SLOC).
   * Exceeding this limit indicates bloated responsibility; split into cohesive submodules.
   * *Strict Enforcement*: Limits cannot be bypassed via code attributes (`#[allow(...)]`). To adjust ceilings globally, edit `.agent-lint.toml`.
3. **Rule 2b: Inline Unit Test Limit in `src/` (Max 5,000 Characters)**:
   * Inline unit tests (`#[cfg(test)]`) inside a `src/` file must not exceed **5,000 characters**.
   * When tests exceed 5,000 characters:
     - **Integration tests** (public API): move to the root `tests/` directory (e.g., `tests/<module>_test.rs`).
     - **Unit tests** (requiring access to private/crate items): extract into a dedicated submodule file (e.g., `src/<module>/tests.rs` or `src/<module>_tests.rs` with `#[cfg(test)] mod tests;`).
     This preserves encapsulation while keeping production files compact for LLM context windows.
4. **Rule 3: File Documentation Limit (Max 4,000 Characters)**:
   * The total character count of documentation comments (`//`, `///`, `//!`, `/* */`) must be **under 4,000 characters** (approx. 50–80 lines).
   * This forces descriptions to remain concise and high-signal, preventing LLM context bloat.
5. **Rule 4: Function Physical Size Limit (Max 2,000 Characters)**:
   * A single function (production or test, including signature, body, comments, and braces) must be **under 2,000 characters** (approx. 40–50 physical lines).
   * Ensures every function fits cleanly on a single screen or within a single context window turn. Oversized test functions must be refactored into smaller test cases or helper assertions.
   * *Strict Enforcement*: Functions cannot bypass limits via code attributes (`#[allow(...)]`). Decompose into smaller helper functions.
6. **Anti-Code-Golfing Principle**:
   * Never compress variable names (e.g. `transaction_context` -> `tc`), eliminate idiomatic whitespace/newlines, or abuse macros to artificially circumvent character limits. Limits exist to force clean architectural decomposition into cohesive submodules and helper functions.

---

## 3. Development Workflow (TDD & Quality Verification)

When contributing or adding features, always adhere to the verification cycle:

1. **Write Failing Tests First**: Write unit tests or doctests describing expected behaviors.
2. **Verify Static & Compiler Compliance**:
   ```bash
   # 1. Check compiler linter rules (Agent-Native constraints)
   cargo check

   # 2. Run unit tests and doctests
   cargo test

   # 3. Verify doc link integrity and ensure zero warnings
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
   ```
3. **Verify Runtime UX & Browser Behavior (E2E Integration Test)**:
   ```bash
   # Execute full 25-checkpoint Playwright E2E interactive test suite
   npm run test:e2e
   ```

---

## 4. AI Agent Navigation Guide

For AI Agents interacting with this repository:

1. **Entry Points**: Start with `AGENTS.md` (this file) and `README.md` to grasp project guidelines and compilation rules.
2. **Module Indexing**: Treat `mod.rs` in any directory as the module's architecture map. Read the `//!` header to understand submodules and dependencies before diving into child files.
3. **Graph Traversal via Rustdoc Links**: Follow compile-checked intra-doc links (`[Type]`) to traverse dependencies deterministically.
4. **Pre-commit Verification**: Never consider a task complete without passing `cargo check`, `cargo test`, `cargo doc`, and `npm run test:e2e`.

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
