//! # Line Index Subsystem (index.md)
//!
//! ## Overview
//! Manages byte offset tables, active query filters, and filtered line projections
//! enabling instant O(1) random-access and virtual scroll lookups across persisted logs.
//!
//! ## Submodules
//! - [`log_index`]: Core LogIndex structure maintaining line start/end byte offsets and filtered line mapping.
//! - [`filter`]: Active regex and case-sensitive/inverted search filter compilers.
//! - [`types`]: Strongly-typed primitives (`ByteOffset`, `LineIndex`, `LineRange`).
//!
//! ## Search Tags
//! #line-index, #byte-offset, #filter, #virtual-scroll, #fast-lookup

pub mod filter;
pub mod log_index;
pub mod types;

// Re-export commonly used items
pub use filter::ActiveFilterBuilder;
pub use log_index::LogIndex;
pub use types::{ByteOffset, LineIndex, LineRange};

#[cfg(test)]
pub mod tests;
