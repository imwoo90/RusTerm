use crate::worker::error::LogError;
use crate::worker::repository::index::{ActiveFilterBuilder, LineRange};
use crate::worker::repository::storage::StorageBackend;
use crate::worker::state::WorkerState;
use gloo_timers::future::TimeoutFuture;
use std::cell::RefCell;
use std::rc::Rc;

pub struct LogSearcher;

const SEARCH_BATCH_SIZE: usize = 5000;

impl LogSearcher {
    pub async fn search_async(
        state_rc: Rc<RefCell<WorkerState>>,
        query: String,
        match_case: bool,
        use_regex: bool,
        invert: bool,
    ) -> Result<(), LogError> {
        let (total_lines, search_id) = {
            let mut state = state_rc.borrow_mut();
            if query.trim().is_empty() {
                state.proc.repository.index.clear_filter();
                return Ok(());
            }

            state.proc.repository.index.active_filter = Some(
                ActiveFilterBuilder::new(query)
                    .case_sensitive(match_case)
                    .regex(use_regex)
                    .invert(invert)
                    .build()
                    .map_err(LogError::Regex)?,
            );
            state.proc.repository.index.is_filtering = true;
            state.proc.repository.index.filtered_lines.clear();

            state.current_search_id += 1;
            (
                state.proc.repository.index.line_count,
                state.current_search_id,
            )
        };

        let mut idx = 0;
        let mut buf = vec![0u8; 512 * 1024];

        while idx < total_lines {
            // Check for cancellation
            if state_rc.borrow().current_search_id != search_id {
                return Ok(());
            }

            let batch_end = (idx + SEARCH_BATCH_SIZE).min(total_lines);

            {
                let mut state = state_rc.borrow_mut();
                let repo = &mut state.proc.repository;

                // Ensure index consistency
                if batch_end > repo.index.line_count {
                    // Lines might have been cleared or changed
                    break;
                }

                let (s_off, e_off) = {
                    let off = &repo.index.line_offsets;
                    (off[idx], off[batch_end])
                };
                let size = (e_off.0 - s_off.0) as usize;
                if buf.len() < size {
                    buf.resize(size, 0);
                }

                repo.storage.backend.read_at(s_off, &mut buf[..size])?;

                let text = repo
                    .storage
                    .decoder
                    .decode_with_u8_array(&buf[..size])
                    .map_err(LogError::Js)?;

                let filter = repo.index.active_filter.as_ref().unwrap().clone();

                for (j, line) in text.trim_end_matches('\n').split('\n').enumerate() {
                    if filter.matches(line) {
                        let off_ptr = &repo.index.line_offsets;
                        if idx + j + 1 < off_ptr.len() {
                            let range = LineRange {
                                start: off_ptr[idx + j],
                                end: off_ptr[idx + j + 1],
                            };
                            repo.index.push_filtered(range);
                        }
                    }
                }
            }

            idx = batch_end;
            // Yield to allow other messages and status updates
            TimeoutFuture::new(0).await;
        }
        Ok(())
    }
}
