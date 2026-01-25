use crate::worker::error::LogError;
use crate::worker::index::ByteOffset;

/// Trait for storage backend operations
pub trait StorageBackend {
    fn read_at(&self, offset: ByteOffset, buf: &mut [u8]) -> Result<usize, LogError>;
    fn write_at(&self, offset: ByteOffset, data: &[u8]) -> Result<usize, LogError>;
    fn get_file_size(&self) -> Result<ByteOffset, LogError>;
    fn truncate(&self, size: u64) -> Result<(), LogError>;
    fn flush(&self) -> Result<(), LogError>;
}
