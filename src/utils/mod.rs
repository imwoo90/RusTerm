//! # Utility Subsystem (index.md)
//!
//! ## Overview
//! General helper utilities covering terminal ANSI parsing, device identification, hex/text format conversions, command history, and Web Serial API interop bindings.
//!
//! ## Submodules
//! - [`ansi_decoder`] (`ansi_decoder.rs`): Fast ANSI terminal color and styling escape code parser.
//! - [`device_id`] (`device_id/mod.rs`): USB VID/PID hardware chip identification and user-defined port aliases.
//! - [`file_save`] (`file_save.rs`): Browser file download triggers for exported log files.
//! - [`format`](mod@format) (`format.rs`): Bidirectional text-to-hex formatting, hex parsing, and worker message senders.
//! - [`history`] (`history.rs`): FIFO command history ring buffer with deduplication and persistence.
//! - [`macros`] (`macros.rs`): User-configurable macro buttons with keybindings and hex/ascii transmission.
//! - [`scroll`] (`scroll.rs`): Virtual scrolling window index and viewport offset math.
//! - [`serial_api`] (`serial_api.rs`): Web Serial API browser JavaScript interop wrappers.
//! - [`simulation`] (`simulation.rs`): Mock serial port generator for local development without hardware.
//! - [`terminal_bindings`] (`terminal_bindings.rs`): DOM keyboard and terminal event binding handlers.
//!
//! ## Search Tags
//! #utils, #ansi-decoder, #device-id, #hex-format, #history, #macros, #virtual-scroll

pub mod ansi_decoder;
pub mod device_id;
pub mod file_save;
pub mod format;
pub mod history;
pub mod macros;
pub mod scroll;
pub mod serial_api;
pub mod simulation;
pub mod terminal_bindings;

pub use ansi_decoder::decode_ansi_text;
pub use device_id::{get_port_info, set_stored_alias, SerialDeviceInfo};
pub use format::{
    convert_hex_to_text, convert_text_to_hex, format_hex_input, parse_hex_string,
    send_chunk_to_worker, send_worker_msg,
};
pub use history::CommandHistory;
pub use macros::MacroStorage;
pub use scroll::{calculate_start_index, calculate_window_size};
pub use serial_api as serial;
