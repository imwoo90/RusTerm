use crate::state::LineEnding;
use chrono::Timelike;
use regex::Regex;
use std::borrow::Cow;
use std::fmt::Write;
use wasm_bindgen::prelude::*;
use wasm_streams::ReadableStream;
use web_sys::{FileSystemReadWriteOptions, FileSystemSyncAccessHandle, TextDecoder, TextEncoder};

const READ_BUFFER_SIZE: usize = 64 * 1024;
const SEARCH_BATCH_SIZE: usize = 5000;
const LEFTOVER_FLUSH_LIMIT: usize = 4096;
const EXPORT_CHUNK_SIZE: u64 = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Clone)]
pub struct ActiveFilter {
    pub query: String,
    pub match_case: bool,
    pub regex: Option<Regex>,
    pub invert: bool,
}

impl ActiveFilter {
    pub fn matches(&self, text: &str) -> bool {
        let matched = if let Some(re) = &self.regex {
            re.is_match(text)
        } else if self.match_case {
            text.contains(&self.query)
        } else {
            text.to_lowercase().contains(&self.query.to_lowercase())
        };
        if self.invert {
            !matched
        } else {
            matched
        }
    }
}

#[wasm_bindgen]
pub struct LogProcessor {
    sync_handle: Option<FileSystemSyncAccessHandle>,
    line_offsets: Vec<u64>,
    line_count: usize,
    filtered_lines: Vec<LineRange>,
    is_filtering: bool,
    active_filter: Option<ActiveFilter>,
    leftover_chunk: String,
    encoder: TextEncoder,
    decoder: TextDecoder,
    line_ending_mode: LineEnding,
}

#[wasm_bindgen]
impl LogProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<LogProcessor, JsValue> {
        Ok(LogProcessor {
            sync_handle: None,
            line_offsets: vec![0],
            line_count: 0,
            filtered_lines: Vec::new(),
            is_filtering: false,
            active_filter: None,
            leftover_chunk: String::new(),
            encoder: TextEncoder::new()?,
            decoder: TextDecoder::new()?,
            line_ending_mode: LineEnding::NL,
        })
    }

    // --- OPFS Helpers ---
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No handle")?;
        let opts = FileSystemReadWriteOptions::new();
        opts.set_at(offset as f64);
        handle
            .read_with_u8_array_and_options(buf, &opts)
            .map(|n| n as usize)
    }

    fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No handle")?;
        let opts = FileSystemReadWriteOptions::new();
        opts.set_at(offset as f64);
        handle
            .write_with_u8_array_and_options(data, &opts)
            .map(|n| n as usize)
    }

    fn get_file_size(&self) -> Result<u64, JsValue> {
        self.sync_handle
            .as_ref()
            .ok_or("No handle")?
            .get_size()
            .map(|s| s as u64)
    }

    // --- Public API ---
    pub fn get_line_count(&self) -> u32 {
        (if self.is_filtering {
            self.filtered_lines.len()
        } else {
            self.line_count
        }) as u32
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

    pub fn set_sync_handle(&mut self, handle: FileSystemSyncAccessHandle) -> Result<(), JsValue> {
        self.sync_handle = Some(handle);
        let size = self.get_file_size()?;
        if size > 0 {
            self.line_offsets = vec![0];
            self.line_count = 0;
            let (mut off, mut buf) = (0u64, vec![0u8; READ_BUFFER_SIZE]);
            while off < size {
                let len = (size - off).min(buf.len() as u64) as usize;
                self.read_at(off, &mut buf[..len])?;
                for (i, &b) in buf[..len].iter().enumerate() {
                    if b == 10 {
                        self.line_offsets.push(off + i as u64 + 1);
                        self.line_count += 1;
                    }
                }
                off += len as u64;
            }
        }
        Ok(())
    }

    pub fn append_chunk(&mut self, chunk: &[u8], is_hex: bool) -> Result<u32, JsValue> {
        let text = if is_hex {
            self.format_hex_chunk(chunk)
        } else {
            self.decode_with_streaming(chunk)?
        };

        let (batch, offsets, filtered) = self.prepare_batch(&text);
        if !batch.is_empty() {
            self.write_and_update(&batch, offsets, filtered)?;
        }
        Ok(self.get_line_count())
    }

    pub fn append_log(&mut self, text: String) -> Result<u32, JsValue> {
        let log = format!("[TX] {} {}\n", self.get_timestamp(), text);
        let len = log.len() as u64;
        let filtered = if self.is_filtering
            && self
                .active_filter
                .as_ref()
                .map_or(false, |f| f.matches(&log))
        {
            vec![LineRange { start: 0, end: len }]
        } else {
            vec![]
        };
        self.write_and_update(&log, vec![len], filtered)?;
        Ok(self.get_line_count())
    }

    pub fn request_window(&self, start: usize, count: usize) -> Result<JsValue, JsValue> {
        let total = self.get_line_count() as usize;
        let (s, e) = (start.min(total), (start + count).min(total));
        let mut lines = Vec::with_capacity(e - s);
        for i in s..e {
            let (s_off, e_off) = self.get_offsets_for_line(i);
            let mut buf = vec![0u8; (e_off - s_off) as usize];
            self.read_at(s_off, &mut buf)?;
            lines.push(
                self.decoder
                    .decode_with_u8_array(&buf)?
                    .trim_end_matches('\n')
                    .to_string(),
            );
        }
        Ok(serde_wasm_bindgen::to_value(&lines)?)
    }

    pub fn search_logs(
        &mut self,
        query: String,
        case: bool,
        regex: bool,
        invert: bool,
    ) -> Result<u32, JsValue> {
        if query.trim().is_empty() {
            return self.reset_filter();
        }

        let re = if regex {
            Some(
                regex::RegexBuilder::new(&query)
                    .case_insensitive(!case)
                    .build()
                    .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };
        self.active_filter = Some(ActiveFilter {
            query,
            match_case: case,
            regex: re,
            invert,
        });
        self.is_filtering = true;
        self.filtered_lines.clear();

        let (mut buf, mut idx) = (vec![0u8; 512 * 1024], 0);
        while idx < self.line_count {
            let batch_end = (idx + SEARCH_BATCH_SIZE).min(self.line_count);
            let (s_off, e_off) = (self.line_offsets[idx], self.line_offsets[batch_end]);
            let size = (e_off - s_off) as usize;
            if buf.len() < size {
                buf.resize(size, 0);
            }
            self.read_at(s_off, &mut buf[..size])?;

            let text = self.decoder.decode_with_u8_array(&buf[..size])?;
            let filter = self.active_filter.as_ref().unwrap();
            for (j, line) in text.trim_end_matches('\n').split('\n').enumerate() {
                if filter.matches(line) {
                    self.filtered_lines.push(LineRange {
                        start: self.line_offsets[idx + j],
                        end: self.line_offsets[idx + j + 1],
                    });
                }
            }
            idx = batch_end;
        }
        Ok(self.filtered_lines.len() as u32)
    }

    pub fn clear(&mut self) -> Result<(), JsValue> {
        let h = self.sync_handle.as_ref().ok_or("No handle")?;
        h.truncate_with_f64(0.0)?;
        h.flush()?;
        self.line_offsets = vec![0];
        self.line_count = 0;
        self.filtered_lines.clear();
        Ok(())
    }

    pub fn export_logs(&self, ts: bool) -> Result<js_sys::Object, JsValue> {
        let size = self.get_file_size()?;
        let (dec, enc, mode, h_clone) = (
            self.decoder.clone(),
            self.encoder.clone(),
            self.line_ending_mode,
            self.sync_handle.as_ref().cloned().unwrap(),
        );

        let stream = futures_util::stream::unfold(0u64, move |off| {
            let (h, d, e) = (h_clone.clone(), dec.clone(), enc.clone());
            async move {
                if off >= size {
                    return None;
                }
                let len = (size - off).min(EXPORT_CHUNK_SIZE) as usize;
                let mut buf = vec![0u8; len];
                let opts = FileSystemReadWriteOptions::new();
                opts.set_at(off as f64);
                if h.read_with_u8_array_and_options(&mut buf, &opts).is_err() {
                    return None;
                }

                let res = if ts {
                    JsValue::from(js_sys::Uint8Array::from(&buf[..]))
                } else {
                    let text = d.decode_with_u8_array(&buf).unwrap_or_default();
                    let sep = match mode {
                        LineEnding::CR => "\r",
                        LineEnding::NLCR => "\r\n",
                        _ => "\n",
                    };
                    let out = text
                        .split(sep)
                        .map(|l| if l.len() > 15 { &l[15..] } else { l })
                        .collect::<Vec<_>>()
                        .join(sep);
                    JsValue::from(e.encode_with_input(&out))
                };
                Some((Ok(res), off + len as u64))
            }
        });
        Ok(ReadableStream::from_stream(stream).into_raw().into())
    }

    // --- Private Log Logic ---
    fn write_and_update(
        &mut self,
        text: &str,
        offsets: Vec<u64>,
        filtered: Vec<LineRange>,
    ) -> Result<(), JsValue> {
        let start = self.get_file_size()?;
        self.write_at(start, self.encoder.encode_with_input(text).as_ref())?;
        for off in offsets {
            self.line_offsets.push(start + off);
            self.line_count += 1;
        }
        for mut r in filtered {
            r.start += start;
            r.end += start;
            self.filtered_lines.push(r);
        }
        Ok(())
    }

    fn prepare_batch(&mut self, chunk: &str) -> (String, Vec<u64>, Vec<LineRange>) {
        if !self.leftover_chunk.is_empty() && self.leftover_chunk.len() > LEFTOVER_FLUSH_LIMIT {
            self.leftover_chunk.push('\n');
        }
        let full = if self.leftover_chunk.is_empty() {
            Cow::Borrowed(chunk)
        } else {
            Cow::Owned(format!("{}{}", self.leftover_chunk, chunk))
        };

        let mut iter = match self.line_ending_mode {
            LineEnding::None | LineEnding::NL => full.split("\n"),
            LineEnding::CR => full.split("\r"),
            LineEnding::NLCR => full.split("\r\n"),
        }
        .peekable();

        let (mut batch, mut offsets, mut filtered, mut rel) = (
            String::with_capacity(chunk.len() * 2),
            Vec::new(),
            Vec::new(),
            0u64,
        );
        let ts = self.get_timestamp();

        while let Some(line) = iter.next() {
            if iter.peek().is_none() {
                self.leftover_chunk = line.to_string();
                break;
            }
            let clean = self.clean_line_ending(line);
            let start_len = batch.len();
            let _ = write!(batch, "{} {}\n", ts, clean);
            let len = (batch.len() - start_len) as u64;
            if self.is_filtering
                && self
                    .active_filter
                    .as_ref()
                    .map_or(false, |f| f.matches(&batch[start_len..]))
            {
                filtered.push(LineRange {
                    start: rel,
                    end: rel + len,
                });
            }
            rel += len;
            offsets.push(rel);
        }
        (batch, offsets, filtered)
    }

    fn get_timestamp(&self) -> String {
        let now = chrono::Utc::now();
        format!(
            "[{:02}:{:02}:{:02}.{:03}]",
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_subsec_millis()
        )
    }

    fn clean_line_ending<'a>(&self, line: &'a str) -> &'a str {
        let mut clean = line;
        if self.line_ending_mode == LineEnding::NL && clean.ends_with('\r') {
            clean = &clean[..clean.len() - 1];
        }
        if self.line_ending_mode == LineEnding::CR && clean.starts_with('\n') {
            clean = &clean[1..];
        }
        clean
    }

    fn format_hex_chunk(&self, chunk: &[u8]) -> String {
        chunk.iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{:02X} ", b);
            acc
        }) + "\n"
    }

    fn decode_with_streaming(&self, chunk: &[u8]) -> Result<String, JsValue> {
        let opts = web_sys::TextDecodeOptions::new();
        opts.set_stream(true);
        self.decoder.decode_with_u8_array_and_options(chunk, &opts)
    }

    fn get_offsets_for_line(&self, i: usize) -> (u64, u64) {
        if self.is_filtering {
            let r = self.filtered_lines[i];
            (r.start, r.end)
        } else {
            (self.line_offsets[i], self.line_offsets[i + 1])
        }
    }

    fn reset_filter(&mut self) -> Result<u32, JsValue> {
        self.is_filtering = false;
        self.active_filter = None;
        self.filtered_lines.clear();
        Ok(self.line_count as u32)
    }
}
