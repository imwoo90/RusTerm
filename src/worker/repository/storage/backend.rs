//! # Storage Backend Abstraction
//!
//! ## Overview
//! Declares the storage backend trait specifying required read, write, truncate, and flush operations across different persistent log storage implementations.
//!
//! ## Search Tags
//! #storage-backend, #trait-abstraction, #disk-io, #read-write, #flush

use crate::worker::error::LogError;
use crate::worker::repository::index::ByteOffset;

/// Trait for storage backend operations
pub trait StorageBackend {
    fn read_at(&self, offset: ByteOffset, buf: &mut [u8]) -> Result<usize, LogError>;
    fn write_at(&self, offset: ByteOffset, data: &[u8]) -> Result<usize, LogError>;
    fn get_file_size(&self) -> Result<ByteOffset, LogError>;
    fn truncate(&self, size: u64) -> Result<(), LogError>;
    fn flush(&self) -> Result<(), LogError>;
}
