//! Unit and integration tests for combined log repository storage and indexing.
//!
//! Validates synchronous append routines, atomic line range reads, filter matching,
//! and complete repository clearing across memory storage backends.

#[cfg(test)]
mod tests {
    use super::super::index::filter::ActiveFilterBuilder;
    use super::super::index::types::{ByteOffset, LineIndex};
    use super::super::storage::backend::StorageBackend;
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
}
