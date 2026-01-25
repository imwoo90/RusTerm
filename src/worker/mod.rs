pub mod chunk_handler;
pub mod dispatcher;
pub mod error;
pub mod export;
pub mod filter_engine;
pub mod formatter;
pub mod index;
pub mod lifecycle;
pub mod line_writer;
pub mod processor;
pub mod repository;
pub mod search;
pub mod state;
pub mod storage;
pub mod types;

// Re-export public functions
pub use lifecycle::get_app_script_path;
