use crate::state::LineEnding;
use chrono::Timelike;
use regex::Regex;
use wasm_bindgen::prelude::*;
use wasm_streams::ReadableStream;
use web_sys::{FileSystemSyncAccessHandle, TextDecoder, TextEncoder};

#[wasm_bindgen]
pub struct LogProcessor {
    sync_handle: Option<FileSystemSyncAccessHandle>,
    line_offsets: Vec<u64>,
    line_count: usize,

    // Filter state
    is_filtering: bool,
    filtered_lines: Vec<LineRange>,
    active_filter: Option<ActiveFilter>,

    // Decoding State
    leftover_chunk: String,
    line_ending_mode: LineEnding,

    // Encoding helpers
    encoder: TextEncoder,
    decoder: TextDecoder,
}

#[derive(Clone, Copy)]
struct LineRange {
    start: u64,
    end: u64,
}

#[derive(Clone)]
struct ActiveFilter {
    query: String,
    lower_query: String,
    match_case: bool,
    regex: Option<Regex>,
    invert: bool,
}

#[wasm_bindgen]
impl LogProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<LogProcessor, JsValue> {
        Ok(LogProcessor {
            sync_handle: None,
            line_offsets: vec![0],
            line_count: 0,
            is_filtering: false,
            filtered_lines: Vec::new(),
            active_filter: None,
            leftover_chunk: String::new(),
            line_ending_mode: LineEnding::NL,
            encoder: TextEncoder::new()?,
            decoder: TextDecoder::new()?,
        })
    }

    pub fn set_sync_handle(&mut self, handle: FileSystemSyncAccessHandle) {
        web_sys::console::log_1(&"Rust: set_sync_handle called".into());
        self.sync_handle = Some(handle);
        web_sys::console::log_1(&"Rust: set_sync_handle finished".into());
    }

    pub fn get_line_count(&self) -> u32 {
        self.get_current_total() as u32
    }

    pub fn set_line_ending(&mut self, mode: &str) {
        self.line_ending_mode = match mode {
            "None" => LineEnding::None,
            "NL" => LineEnding::NL,
            "CR" => LineEnding::CR,
            "NLCR" => LineEnding::NLCR,
            _ => LineEnding::NL,
        };
    }

    pub fn append_chunk(&mut self, chunk: &[u8], is_hex: bool) -> Result<u32, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;

        let str_part = if is_hex {
            let hex = chunk
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{}\n", hex)
        } else {
            let opts = web_sys::TextDecodeOptions::new();
            opts.set_stream(true);
            self.decoder
                .decode_with_u8_array_and_options(chunk, &opts)?
        };

        let full_text = format!("{}{}", self.leftover_chunk, str_part);

        let mut lines: Vec<&str> = match self.line_ending_mode {
            LineEnding::None => full_text.split('\n').collect(),
            LineEnding::CR => full_text.split('\r').collect(),
            LineEnding::NLCR => full_text.split("\r\n").collect(),
            LineEnding::NL => full_text.split('\n').collect(),
        };

        if let Some(last) = lines.pop() {
            self.leftover_chunk = last.to_string();
        } else {
            self.leftover_chunk.clear();
        }

        if lines.is_empty() {
            return Ok(self.get_current_total() as u32);
        }

        let now = chrono::Utc::now();
        let time_str = format!(
            "[{:02}:{:02}:{:02}.{:03}] ",
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_subsec_millis()
        );

        let mut batch_buffer = String::new();
        for line in &lines {
            let mut clean_line = *line;
            if self.line_ending_mode == LineEnding::NL && clean_line.ends_with('\r') {
                clean_line = &clean_line[..clean_line.len() - 1];
            }
            if self.line_ending_mode == LineEnding::CR && clean_line.starts_with('\n') {
                clean_line = &clean_line[1..];
            }
            batch_buffer.push_str(&time_str);
            batch_buffer.push_str(clean_line);
            batch_buffer.push('\n');
        }

        let write_buffer = self.encoder.encode_with_input(&batch_buffer);
        let pos = handle.get_size()? as u64;
        let opts = web_sys::FileSystemReadWriteOptions::new();
        opts.set_at(pos as f64);
        handle.write_with_u8_array_and_options(write_buffer.as_ref(), &opts)?;

        // Update offsets
        let current_offset = pos;
        let buf: &[u8] = write_buffer.as_ref();
        for i in 0..buf.len() {
            if buf[i] == 10 {
                // \n
                let line_end = current_offset + i as u64 + 1;
                self.line_offsets.push(line_end);
                self.line_count += 1;
            }
        }

        // Realtime filtering
        if self.is_filtering {
            if let Some(filter) = &self.active_filter {
                let mut relative_byte_offset = 0;
                for line in &lines {
                    let mut clean_line = *line;
                    if self.line_ending_mode == LineEnding::NL && clean_line.ends_with('\r') {
                        clean_line = &clean_line[..clean_line.len() - 1];
                    }
                    if self.line_ending_mode == LineEnding::CR && clean_line.starts_with('\n') {
                        clean_line = &clean_line[1..];
                    }
                    let final_line_str = format!("{}{}\n", time_str, clean_line);
                    let line_bytes = self.encoder.encode_with_input(&final_line_str);
                    let line_byte_len = line_bytes.len() as u64;

                    let start_pos = pos + relative_byte_offset;
                    let end_pos = start_pos + line_byte_len;

                    let mut matched;
                    if let Some(re) = &filter.regex {
                        matched = re.is_match(&final_line_str);
                    } else {
                        matched = if filter.match_case {
                            final_line_str.contains(&filter.query)
                        } else {
                            final_line_str.to_lowercase().contains(&filter.lower_query)
                        };
                    }
                    if filter.invert {
                        matched = !matched;
                    }

                    if matched {
                        self.filtered_lines.push(LineRange {
                            start: start_pos,
                            end: end_pos,
                        });
                    }
                    relative_byte_offset += line_byte_len;
                }
            }
        }

        Ok(self.get_current_total() as u32)
    }

    pub fn request_window(&self, start_line: usize, count: usize) -> Result<JsValue, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;
        let total = self.get_current_total();
        let start = start_line.min(total);
        let end = (start + count).min(total);
        let effective_count = end - start;

        if effective_count == 0 {
            return Ok(serde_wasm_bindgen::to_value(&Vec::<String>::new())?);
        }

        let mut lines = Vec::with_capacity(effective_count);
        if self.is_filtering {
            for i in start..end {
                let meta = &self.filtered_lines[i];
                let size = (meta.end - meta.start) as usize;
                let mut buf = vec![0u8; size];
                let opts = web_sys::FileSystemReadWriteOptions::new();
                opts.set_at(meta.start as f64);
                handle.read_with_u8_array_and_options(&mut buf, &opts)?;
                let text = self.decoder.decode_with_u8_array(&buf)?;
                lines.push(if text.ends_with('\n') {
                    text[..text.len() - 1].to_string()
                } else {
                    text
                });
            }
        } else {
            let start_offset = self.line_offsets[start];
            let end_offset = self.line_offsets[end];
            let size = (end_offset - start_offset) as usize;
            let mut read_buffer = vec![0u8; size];
            let opts = web_sys::FileSystemReadWriteOptions::new();
            opts.set_at(start_offset as f64);
            handle.read_with_u8_array_and_options(&mut read_buffer, &opts)?;
            let text = self.decoder.decode_with_u8_array(&read_buffer)?;
            let split = if text.ends_with('\n') {
                &text[..text.len() - 1]
            } else {
                &text
            };
            for l in split.split('\n') {
                lines.push(l.to_string());
            }
        }

        Ok(serde_wasm_bindgen::to_value(&lines)?)
    }

    pub fn search_logs(
        &mut self,
        query: String,
        match_case: bool,
        use_regex: bool,
        invert: bool,
    ) -> Result<u32, JsValue> {
        if query.trim().is_empty() {
            self.is_filtering = false;
            self.active_filter = None;
            self.filtered_lines.clear();
            return Ok(self.line_count as u32);
        }

        let regex = if use_regex {
            Some(Regex::new(&query).map_err(|e| format!("Invalid regex: {}", e))?)
        } else {
            None
        };

        self.active_filter = Some(ActiveFilter {
            lower_query: if match_case {
                query.clone()
            } else {
                query.to_lowercase()
            },
            query,
            match_case,
            regex,
            invert,
        });

        self.is_filtering = true;
        self.filtered_lines.clear();

        // Perform full search
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;
        let chunk_size = 5000;
        let mut i = 0;
        while i < self.line_count {
            let batch_end = (i + chunk_size).min(self.line_count);
            let batch_start_offset = self.line_offsets[i];
            let batch_end_offset = self.line_offsets[batch_end];
            let size = (batch_end_offset - batch_start_offset) as usize;

            let mut buf = vec![0u8; size];
            let opts = web_sys::FileSystemReadWriteOptions::new();
            opts.set_at(batch_start_offset as f64);
            handle.read_with_u8_array_and_options(&mut buf, &opts)?;
            let batch_text = self.decoder.decode_with_u8_array(&buf)?;
            let split = if batch_text.ends_with('\n') {
                &batch_text[..batch_text.len() - 1]
            } else {
                &batch_text
            };
            let batch_lines: Vec<&str> = split.split('\n').collect();

            let filter = self.active_filter.as_ref().unwrap();
            for (j, line) in batch_lines.iter().enumerate() {
                let mut matched;
                if let Some(re) = &filter.regex {
                    matched = re.is_match(line);
                } else {
                    matched = if filter.match_case {
                        line.contains(&filter.query)
                    } else {
                        line.to_lowercase().contains(&filter.lower_query)
                    };
                }
                if filter.invert {
                    matched = !matched;
                }

                if matched {
                    let glob_idx = i + j;
                    self.filtered_lines.push(LineRange {
                        start: self.line_offsets[glob_idx],
                        end: self.line_offsets[glob_idx + 1],
                    });
                }
            }
            i = batch_end;
        }

        Ok(self.filtered_lines.len() as u32)
    }

    pub fn clear(&mut self) -> Result<(), JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;
        handle.truncate_with_f64(0.0)?;
        handle.flush()?;
        self.line_count = 0;
        self.line_offsets = vec![0];
        self.is_filtering = false;
        self.active_filter = None;
        self.filtered_lines.clear();
        self.leftover_chunk.clear();
        Ok(())
    }

    pub fn get_sync_handle(&self) -> Result<FileSystemSyncAccessHandle, JsValue> {
        self.sync_handle
            .clone()
            .ok_or_else(|| JsValue::from_str("No sync handle"))
    }

    pub fn export_logs(&self, include_timestamps: bool) -> Result<js_sys::Object, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?.clone();
        let file_size = handle.get_size()? as u64;
        let decoder = self.decoder.clone();
        let encoder = self.encoder.clone();
        let line_ending = self.line_ending_mode;

        // Create a Rust stream that reads the file in chunks
        let stream = futures_util::stream::unfold(0u64, move |offset| {
            let handle = handle.clone();
            let decoder = decoder.clone();
            let encoder = encoder.clone();

            async move {
                if offset >= file_size {
                    return None;
                }

                let chunk_size = 64 * 1024; // 64KB
                let read_len = (file_size - offset).min(chunk_size as u64) as usize;
                let mut buf = vec![0u8; read_len];

                let opts = web_sys::FileSystemReadWriteOptions::new();
                opts.set_at(offset as f64);

                if handle
                    .read_with_u8_array_and_options(&mut buf, &opts)
                    .is_err()
                {
                    return None;
                }

                if include_timestamps {
                    // Just return the raw chunk
                    let js_buf = js_sys::Uint8Array::from(&buf[..]);
                    Some((Ok(JsValue::from(js_buf)), offset + read_len as u64))
                } else {
                    // Strip timestamps [HH:MM:SS.mmm] (15 chars)
                    let text = decoder.decode_with_u8_array(&buf).unwrap_or_default();
                    let separator = match line_ending {
                        crate::state::LineEnding::CR => "\r",
                        crate::state::LineEnding::NLCR => "\r\n",
                        _ => "\n",
                    };

                    let mut result = String::with_capacity(text.len());
                    for line in text.split(separator) {
                        if line.len() > 15 {
                            result.push_str(&line[15..]);
                            result.push_str(separator);
                        } else if !line.is_empty() {
                            // If it's too short but not empty, just keep it (unlikely with timestamps)
                            result.push_str(line);
                            result.push_str(separator);
                        }
                    }

                    let encoded = encoder.encode_with_input(&result);
                    Some((Ok(JsValue::from(encoded)), offset + read_len as u64))
                }
            }
        });

        // Convert Rust Stream to JS ReadableStream
        let readable = ReadableStream::from_stream(stream);
        Ok(readable.into_raw().into())
    }

    fn get_current_total(&self) -> usize {
        if self.is_filtering {
            self.filtered_lines.len()
        } else {
            self.line_count
        }
    }
}
