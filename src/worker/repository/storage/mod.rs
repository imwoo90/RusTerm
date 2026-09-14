//! Storage backends sub-module catalog (mod.rs = index.md).
//!
//! Defines storage abstractions and implementations for browser Origin Private File System (OPFS)
//! sync access handles.

pub mod backend;
pub mod opfs;

// Re-export commonly used items
pub use backend::StorageBackend;
pub use opfs::{get_opfs_root, init_opfs_session, new_session, LogStorage};

#[cfg(test)]
pub mod tests;
