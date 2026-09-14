//! Memory line indexing and search index catalog (mod.rs = index.md).
//!
//! Manages byte offset tables, active query filters, and filtered line projections
//! enabling instant O(1) lookup of millions of persisted lines.

pub mod filter;
pub mod log_index;
pub mod types;

// Re-export commonly used items
pub use filter::ActiveFilterBuilder;
pub use log_index::LogIndex;
pub use types::{ByteOffset, LineIndex, LineRange};
