//! Unit tests for storage backends and in-memory fallback persistence.
//!
//! Validates binary chunk persistence, partial reads, offsets, file sizing,
//! boundary conditions, and buffer truncation routines.

#[cfg(test)]
mod tests {
    use super::super::backend::StorageBackend;
    use super::super::opfs::OpfsBackend;
    use crate::worker::repository::index::ByteOffset;

    fn create_test_backend() -> OpfsBackend {
        OpfsBackend {
            handle: None,
            fallback: std::sync::RwLock::new(Vec::new()),
        }
    }

    #[test]
    fn test_storage_initial_size_zero() {
        let storage = create_test_backend();
        let size = storage.get_file_size().expect("Get size failed");
        assert_eq!(size.0, 0);
    }

    #[test]
    fn test_storage_write_and_read_roundtrip() {
        let storage = create_test_backend();
        let data = b"Hello, RusTerm In-Memory Storage!";
        let written = storage.write_at(ByteOffset(0), data).expect("Write failed");
        assert_eq!(written, data.len());

        let size = storage.get_file_size().expect("Size check failed");
        assert_eq!(size.0, data.len() as u64);

        let mut read_buf = vec![0u8; data.len()];
        let read_bytes = storage
            .read_at(ByteOffset(0), &mut read_buf)
            .expect("Read failed");
        assert_eq!(read_bytes, data.len());
        assert_eq!(&read_buf[..], data);
    }

    #[test]
    fn test_storage_partial_offset_read() {
        let storage = create_test_backend();
        let data = b"0123456789ABCDEF";
        storage.write_at(ByteOffset(0), data).expect("Write failed");

        let mut slice_buf = [0u8; 4];
        let bytes = storage
            .read_at(ByteOffset(10), &mut slice_buf)
            .expect("Slice read failed");
        assert_eq!(bytes, 4);
        assert_eq!(&slice_buf, b"ABCD");
    }

    #[test]
    fn test_storage_read_past_eof() {
        let storage = create_test_backend();
        storage.write_at(ByteOffset(0), b"Small").expect("Write failed");

        let mut buf = [0u8; 10];
        let read = storage
            .read_at(ByteOffset(100), &mut buf)
            .expect("Out of bounds read");
        assert_eq!(read, 0);
    }

    #[test]
    fn test_storage_write_with_gap_expansion() {
        let storage = create_test_backend();
        let written = storage
            .write_at(ByteOffset(5), b"GapData")
            .expect("Sparse write failed");
        assert_eq!(written, 7);

        let size = storage.get_file_size().expect("Size failed");
        assert_eq!(size.0, 12); // 5 zeroes + 7 bytes

        let mut full_buf = vec![0u8; 12];
        storage
            .read_at(ByteOffset(0), &mut full_buf)
            .expect("Full read failed");
        assert_eq!(&full_buf[..5], &[0, 0, 0, 0, 0]);
        assert_eq!(&full_buf[5..], b"GapData");
    }

    #[test]
    fn test_storage_truncate_and_clear() {
        let storage = create_test_backend();
        storage
            .write_at(ByteOffset(0), b"LongDataToTruncate")
            .expect("Write failed");

        storage.truncate(4).expect("Truncate failed");
        assert_eq!(storage.get_file_size().unwrap().0, 4);

        let mut buf = vec![0u8; 4];
        storage.read_at(ByteOffset(0), &mut buf).expect("Read failed");
        assert_eq!(&buf, b"Long");

        storage.truncate(0).expect("Truncate to zero failed");
        assert_eq!(storage.get_file_size().unwrap().0, 0);
    }

    #[test]
    fn test_storage_flush_success() {
        let storage = create_test_backend();
        assert!(storage.flush().is_ok());
    }
}
