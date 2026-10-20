//! # Utility Subsystem (index.md)
//!
//! ## Overview
//! General helper utilities covering terminal ANSI parsing, device identification, hex/text format conversions, command history, and Web Serial API interop bindings.
//!
//! ## Submodules
//! - [`ansi_decoder`] (`ansi_decoder.rs`): Fast ANSI terminal color and styling escape code parser.
//! - [`device_id`] (`device_id/mod.rs`): USB VID/PID hardware chip identification and user-defined port aliases.
//! - [`format`](mod@format) (`format.rs`): Bidirectional text-to-hex formatting, hex parsing, and worker message senders.
//! - [`history`] (`history.rs`): FIFO command history ring buffer with deduplication and persistence.
//! - [`macros`] (`macros.rs`): User-configurable macro buttons with keybindings and hex/ascii transmission.
//! - [`serial_api`] (`serial_api.rs`): Web Serial API browser JavaScript interop wrappers and mock stream simulation.
//! - [`terminal_bindings`] (`terminal_bindings.rs`): xterm.js DOM bindings and browser file download triggers.
//!
//! ## Search Tags
//! #utils, #ansi-decoder, #device-id, #hex-format, #history, #macros, #serial-api

pub mod ansi_decoder;
pub mod device_id;
pub mod format;
pub mod history;
pub mod macros;
pub mod serial_api;
pub mod terminal_bindings;

pub use ansi_decoder::decode_ansi_text;
pub use device_id::{get_port_info, set_stored_alias, SerialDeviceInfo};
pub use format::{
    convert_hex_to_text, convert_text_to_hex, format_hex_input, parse_hex_string,
    send_chunk_to_worker, send_worker_msg,
};
pub use history::CommandHistory;
pub use macros::MacroStorage;
pub use serial_api as serial;
pub use serial_api::generate_simulated_chunk;
pub use terminal_bindings::{save_stream_to_disk, save_terminal_history};
