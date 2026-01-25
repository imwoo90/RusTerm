use crate::state::LineEnding;
use crate::worker::chunk_handler::ChunkHandler;
use crate::worker::error::LogError;
use crate::worker::export::LogExporter;
use crate::worker::formatter::{
    DefaultFormatter, HexFormatter, LogFormatter, LogFormatterStrategy,
};
use crate::worker::index::{ByteOffset, LineIndex, LineRange, LogIndex};
use crate::worker::line_writer::LineWriter;
use crate::worker::search::LogSearcher;
use crate::worker::storage::{LogStorage, StorageBackend};

use wasm_bindgen::prelude::*;
use web_sys::FileSystemSyncAccessHandle;

use crate::config::{MAX_LINE_BYTES, READ_BUFFER_SIZE};

#[wasm_bindgen]
pub struct LogProcessor {
    storage: LogStorage,
    index: LogIndex,
    formatter: LogFormatter,
    chunk_handler: ChunkHandler,
}

#[wasm_bindgen]
impl LogProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<LogProcessor, JsValue> {
        LogProcessor::new_internal().map_err(JsValue::from)
    }

    fn new_internal() -> Result<Self, LogError> {
        Ok(LogProcessor {
            storage: LogStorage::new()?,
            index: LogIndex::new(),
            formatter: LogFormatter::new(LineEnding::NL),
            chunk_handler: ChunkHandler::new(),
        })
    }

    // --- Public API ---
    pub fn get_line_count(&self) -> u32 {
        self.index.get_total_count() as u32
    }

    pub fn set_line_ending(&mut self, mode: &str) {
        self.formatter.line_ending_mode = match mode {
            "None" => LineEnding::None,
            "NL" => LineEnding::NL,
            "CR" => LineEnding::CR,
            "NLCR" => LineEnding::NLCR,
            _ => LineEnding::NL,
        };
    }

    pub fn set_sync_handle(&mut self, handle: FileSystemSyncAccessHandle) -> Result<(), JsValue> {
        self.set_sync_handle_internal(handle).map_err(JsValue::from)
    }

    fn set_sync_handle_internal(
        &mut self,
        handle: FileSystemSyncAccessHandle,
    ) -> Result<(), LogError> {
        self.storage.backend.handle = Some(handle);
        let size = self.storage.backend.get_file_size()?;
        if size.0 > 0 {
            self.index.reset_base();
            let (mut off, mut buf) = (ByteOffset(0), vec![0u8; READ_BUFFER_SIZE]);
            while off.0 < size.0 {
                let len = (size.0 - off.0).min(buf.len() as u64) as usize;
                self.storage.backend.read_at(off, &mut buf[..len])?;
                for (i, &b) in buf[..len].iter().enumerate() {
                    if b == 10 {
                        self.index.push_line(off + (i as u64 + 1));
                    }
                }
                off = off + (len as u64);
            }
        }
        Ok(())
    }

    pub fn append_chunk(&mut self, chunk: &[u8], is_hex: bool) -> Result<u32, JsValue> {
        let formatter: Box<dyn LogFormatterStrategy> = if is_hex {
            Box::new(HexFormatter {
                line_ending: self.formatter.line_ending_mode,
                max_bytes: MAX_LINE_BYTES,
            })
        } else {
            Box::new(DefaultFormatter {
                line_ending: self.formatter.line_ending_mode,
                max_bytes: MAX_LINE_BYTES,
            })
        };

        let text = if is_hex {
            formatter.format_chunk(chunk)
        } else {
            self.decode_with_streaming_internal(chunk)?
        };

        let timestamp = self.formatter.get_timestamp();
        let is_filtering = self.index.is_filtering;
        let active_filter = self.index.active_filter.clone();

        let (batch, offsets, filtered) = self.chunk_handler.prepare_batch_with_formatter(
            &text,
            &*formatter,
            &timestamp,
            is_filtering,
            |text: &str| is_filtering && active_filter.as_ref().is_some_and(|f| f.matches(text)),
        );

        if !batch.is_empty() {
            LineWriter::write_and_update(
                &mut self.storage,
                &mut self.index,
                &batch,
                offsets,
                filtered,
            )
            .map_err(JsValue::from)?;
        }
        Ok(self.get_line_count())
    }

    pub fn append_log(&mut self, text: String) -> Result<u32, JsValue> {
        let log = format!("[TX] {} {}\n", self.formatter.get_timestamp(), text);
        let len = ByteOffset(log.len() as u64);
        let filtered = if self.index.is_filtering
            && self
                .index
                .active_filter
                .as_ref()
                .is_some_and(|f| f.matches(&log))
        {
            vec![LineRange {
                start: ByteOffset(0),
                end: len,
            }]
        } else {
            vec![]
        };
        LineWriter::write_and_update(
            &mut self.storage,
            &mut self.index,
            &log,
            vec![len],
            filtered,
        )
        .map_err(JsValue::from)?;
        Ok(self.get_line_count())
    }

    pub fn request_window(&self, start: usize, count: usize) -> Result<JsValue, JsValue> {
        self.request_window_internal(start, count)
            .map_err(JsValue::from)
    }

    fn request_window_internal(&self, start: usize, count: usize) -> Result<JsValue, LogError> {
        let total = self.get_line_count() as usize;
        let (s, e) = (start.min(total), (start + count).min(total));
        let mut lines = Vec::with_capacity(e - s);
        for i in s..e {
            if let Some(range) = self.index.get_line_range(LineIndex(i)) {
                let mut buf = vec![0u8; (range.end.0 - range.start.0) as usize];
                self.storage.backend.read_at(range.start, &mut buf)?;
                let text = self
                    .storage
                    .decoder
                    .decode_with_u8_array(&buf)
                    .map_err(LogError::from)?
                    .trim_end_matches('\n')
                    .to_string();
                lines.push((i, text));
            }
        }
        serde_wasm_bindgen::to_value(&lines).map_err(|e| LogError::Encoding(e.to_string()))
    }

    pub fn search_logs(
        &mut self,
        query: String,
        case: bool,
        regex: bool,
        invert: bool,
    ) -> Result<u32, JsValue> {
        LogSearcher::search(
            &mut self.storage,
            &mut self.index,
            query,
            case,
            regex,
            invert,
        )
        .map_err(JsValue::from)
    }

    pub fn clear(&mut self) -> Result<(), JsValue> {
        self.clear_internal().map_err(JsValue::from)
    }

    fn clear_internal(&mut self) -> Result<(), LogError> {
        self.storage.backend.truncate(0)?;
        self.storage.backend.flush()?;
        self.index.reset_base();
        self.chunk_handler.clear();
        Ok(())
    }

    pub fn export_logs(&self, ts: bool) -> Result<js_sys::Object, JsValue> {
        self.export_logs_internal(ts).map_err(JsValue::from)
    }

    fn export_logs_internal(&self, ts: bool) -> Result<js_sys::Object, LogError> {
        let size = self.storage.backend.get_file_size()?;
        let handle = self
            .storage
            .backend
            .handle
            .as_ref()
            .cloned()
            .ok_or_else(|| LogError::Storage("OPFS handle missing for export".into()))?;

        LogExporter::export_logs(
            handle,
            self.storage.decoder.clone(),
            self.storage.encoder.clone(),
            self.formatter.line_ending_mode,
            size,
            ts,
        )
    }

    fn decode_with_streaming_internal(&self, chunk: &[u8]) -> Result<String, JsValue> {
        let opts = web_sys::TextDecodeOptions::new();
        opts.set_stream(true);
        self.storage
            .decoder
            .decode_with_u8_array_and_options(chunk, &opts)
    }
}
