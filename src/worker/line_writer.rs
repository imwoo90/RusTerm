use crate::worker::error::LogError;
use crate::worker::index::{ByteOffset, LineRange, LogIndex};
use crate::worker::storage::{LogStorage, StorageBackend};

/// Handles writing lines to storage and updating the index
pub struct LineWriter;

impl LineWriter {
    pub fn new() -> Self {
        Self
    }

    /// Writes text to storage and updates the index with offsets and filtered ranges
    pub fn write_and_update(
        storage: &mut LogStorage,
        index: &mut LogIndex,
        text: &str,
        offsets: Vec<ByteOffset>,
        filtered: Vec<LineRange>,
    ) -> Result<(), LogError> {
        let start = storage.backend.get_file_size()?;
        storage
            .backend
            .write_at(start, storage.encoder.encode_with_input(text).as_ref())?;

        for off in offsets {
            index.push_line(start + off.0);
        }

        for mut r in filtered {
            r.start = start + r.start.0;
            r.end = start + r.end.0;
            index.push_filtered(r);
        }

        Ok(())
    }
}

impl Default for LineWriter {
    fn default() -> Self {
        Self::new()
    }
}
