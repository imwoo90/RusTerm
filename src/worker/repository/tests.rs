#![allow(clippy::unwrap_used, clippy::expect_used, clippy::module_inception)]
//! Unit and integration tests for log repository storage, byte offset indexing, and active filters.

#[cfg(test)]
mod tests {
    use super::super::index::{ActiveFilterBuilder, ByteOffset, LineIndex, LineRange, LogIndex};
    use super::super::storage::{OpfsBackend, StorageBackend};
    use super::super::LogRepository;

    #[test]
    fn test_repository_initial_state() {
        let repo = LogRepository::new().expect("Repo init failed");
        assert_eq!(repo.get_line_count(), 0);
        assert!(!repo.is_filtering());
    }

    #[test]
    fn test_repository_append_and_read_lines() {
        let mut repo = LogRepository::new().expect("Repo init failed");
        let text = "First line\nSecond line\n";
        let offsets = vec![ByteOffset(11), ByteOffset(23)];

        repo.append_lines(text, offsets, Vec::new())
            .expect("Append failed");
        assert_eq!(repo.get_line_count(), 2);

        let r0 = repo.get_line_range(LineIndex(0)).expect("Missing line 0");
        let line0_bytes = repo.read_line(r0).expect("Read line 0 failed");
        assert_eq!(String::from_utf8_lossy(&line0_bytes), "First line\n");

        let r1 = repo.get_line_range(LineIndex(1)).expect("Missing line 1");
        let line1_bytes = repo.read_line(r1).expect("Read line 1 failed");
        assert_eq!(String::from_utf8_lossy(&line1_bytes), "Second line\n");
    }

    #[test]
    fn test_repository_clear_resets_storage_and_index() {
        let mut repo = LogRepository::new().expect("Repo init failed");
        let text = "Log to clear\n";
        let offsets = vec![ByteOffset(13)];

        repo.append_lines(text, offsets, Vec::new()).unwrap();
        assert_eq!(repo.get_line_count(), 1);

        repo.clear().expect("Clear failed");
        assert_eq!(repo.get_line_count(), 0);
        assert_eq!(
            repo.storage.backend.get_file_size().unwrap().0,
            0,
            "Storage must be truncated to 0"
        );
    }

    #[test]
    fn test_repository_filter_matching() {
        let mut repo = LogRepository::new().expect("Repo init failed");
        assert!(!repo.matches_active_filter("Error: Sample"));

        let filter = ActiveFilterBuilder::new("Error".to_string())
            .case_sensitive(true)
            .build()
            .unwrap();

        repo.index.is_filtering = true;
        repo.index.active_filter = Some(filter);

        assert!(repo.is_filtering());
        assert!(repo.matches_active_filter("Error: Test Failed"));
        assert!(!repo.matches_active_filter("Info: Test Passed"));
    }

    #[test]
    fn test_log_index_push_and_range_lookup() {
        let mut index = LogIndex::new();
        assert_eq!(index.get_total_count(), 0);

        index.push_line(ByteOffset(10));
        index.push_line(ByteOffset(25));
        index.push_line(ByteOffset(40));

        assert_eq!(index.get_total_count(), 3);

        let r0 = index.get_line_range(LineIndex(0)).expect("Missing line 0");
        assert_eq!(r0.start.0, 0);
        assert_eq!(r0.end.0, 10);

        let r1 = index.get_line_range(LineIndex(1)).expect("Missing line 1");
        assert_eq!(r1.start.0, 10);
        assert_eq!(r1.end.0, 25);

        let r2 = index.get_line_range(LineIndex(2)).expect("Missing line 2");
        assert_eq!(r2.start.0, 25);
        assert_eq!(r2.end.0, 40);

        assert!(index.get_line_range(LineIndex(3)).is_none());
    }

    #[test]
    fn test_log_index_reset_base() {
        let mut index = LogIndex::new();
        index.push_line(ByteOffset(15));
        index.push_line(ByteOffset(30));
        assert_eq!(index.get_total_count(), 2);

        index.reset_base();
        assert_eq!(index.get_total_count(), 0);
        assert_eq!(index.line_offsets, vec![ByteOffset(0)]);
        assert!(index.get_line_range(LineIndex(0)).is_none());
    }

    #[test]
    fn test_log_index_filtered_ranges() {
        let mut index = LogIndex::new();
        index.push_line(ByteOffset(20));
        index.push_line(ByteOffset(40));
        index.push_line(ByteOffset(60));

        index.is_filtering = true;
        index.push_filtered(LineRange {
            start: ByteOffset(20),
            end: ByteOffset(40),
        });

        assert_eq!(index.get_total_count(), 1);
        let r = index
            .get_line_range(LineIndex(0))
            .expect("Filtered line missing");
        assert_eq!(r.start.0, 20);
        assert_eq!(r.end.0, 40);

        index.clear_filter();
        assert_eq!(index.get_total_count(), 3);
    }

    #[test]
    fn test_active_filter_case_sensitive() {
        let filter = ActiveFilterBuilder::new("Sensor".to_string())
            .case_sensitive(true)
            .build()
            .expect("Filter build failed");

        assert!(filter.matches("Info: Sensor reading"));
        assert!(!filter.matches("Info: sensor reading"));
        assert!(!filter.matches("Info: SENSOR reading"));
    }

    #[test]
    fn test_active_filter_case_insensitive() {
        let filter = ActiveFilterBuilder::new("Sensor".to_string())
            .case_sensitive(false)
            .build()
            .expect("Filter build failed");

        assert!(filter.matches("Info: Sensor reading"));
        assert!(filter.matches("Info: sensor reading"));
        assert!(filter.matches("Info: SENSOR reading"));
        assert!(!filter.matches("Info: Voltage fluctuation"));
    }

    #[test]
    fn test_active_filter_regex_valid() {
        let filter =
            ActiveFilterBuilder::new(r"Error:\s+System\s+overheat\s+at\s+\d+\.\d+".to_string())
                .regex(true)
                .build()
                .expect("Regex build failed");

        assert!(filter.matches("Error: System overheat at 82.5°C"));
        assert!(!filter.matches("Warning: System overheat at 82.5°C"));
        assert!(!filter.matches("Error: System overheat at unknown"));
    }

    #[test]
    fn test_active_filter_regex_invalid_returns_err() {
        let result = ActiveFilterBuilder::new("[".to_string())
            .regex(true)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_active_filter_inverted() {
        let filter = ActiveFilterBuilder::new("Error".to_string())
            .invert(true)
            .build()
            .expect("Filter build failed");

        assert!(filter.matches("Info: Everything OK"));
        assert!(filter.matches("Warning: Voltage unstable"));
        assert!(!filter.matches("Error: Critical failure"));
    }

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
        storage
            .write_at(ByteOffset(0), b"Small")
            .expect("Write failed");

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
        assert_eq!(size.0, 12);

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
        storage
            .read_at(ByteOffset(0), &mut buf)
            .expect("Read failed");
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
