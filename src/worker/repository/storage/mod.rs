//! # Storage Subsystem (index.md)
//!
//! ## Overview
//! Storage backend abstractions and implementations interfacing with the browser's
//! Origin Private File System (OPFS) synchronous access handles for zero-copy file persistence.
//!
//! ## Submodules
//! - [`backend`]: Generic `StorageBackend` trait abstracting disk and memory backends.
//! - [`opfs`]: OPFS synchronous access handle implementation and session directory management.
//!
//! ## Search Tags
//! #storage, #opfs, #file-persistence, #sync-access-handle

pub mod backend;
pub mod opfs;

// Re-export commonly used items
pub use backend::StorageBackend;
pub use opfs::{get_opfs_root, init_opfs_session, new_session, LogStorage};

#[cfg(test)]
pub mod tests;
