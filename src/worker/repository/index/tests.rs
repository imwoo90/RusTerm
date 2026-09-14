//! Unit tests for line indexing tables, byte offset mappings, and active filters.
//!
//! Validates monotonic line offsets, filtered range projections, case-sensitivity,
//! regex compilation, inverted query evaluations, and index resets.

#[cfg(test)]
mod tests {
    use super::super::filter::ActiveFilterBuilder;
    use super::super::log_index::LogIndex;
    use super::super::types::{ByteOffset, LineIndex, LineRange};

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
        let r = index.get_line_range(LineIndex(0)).expect("Filtered line missing");
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
        let filter = ActiveFilterBuilder::new(r"Error:\s+System\s+overheat\s+at\s+\d+\.\d+".to_string())
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
}
