use super::types::WorkerMsg;
use dioxus::prelude::*;

/// Hook to request a window of log data from Worker
pub fn use_data_request(
    start_index: Signal<usize>,
    window_size: usize,
    total_lines: Signal<usize>,
    worker: Signal<Option<web_sys::Worker>>,
) {
    use_effect(move || {
        let start = start_index();
        total_lines();
        if let Some(w) = worker.read().as_ref() {
            if let Ok(msg) = serde_json::to_string(&WorkerMsg::RequestWindow {
                start_line: start,
                count: window_size,
            }) {
                let _ = w.post_message(&msg.into());
            }
        }
    });
}
