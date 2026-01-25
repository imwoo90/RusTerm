pub mod backend;
pub mod opfs;

// Re-export commonly used items
pub use backend::StorageBackend;
pub use opfs::{get_opfs_root, init_opfs_session, new_session, LogStorage};
