//! Origin Private File System (OPFS) storage implementation.
//!
//! Leverages `FileSystemSyncAccessHandle` in Web Workers for ultra-low latency, non-blocking
//! binary disk persistence of streaming serial logs.

use crate::worker::error::LogError;
use crate::worker::repository::index::ByteOffset;
use crate::worker::repository::storage::backend::StorageBackend;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// OPFS-based storage backend with in-memory fallback for non-secure contexts
pub struct OpfsBackend {
    pub handle: Option<web_sys::FileSystemSyncAccessHandle>,
    pub fallback: std::sync::RwLock<Vec<u8>>,
}

impl StorageBackend for OpfsBackend {
    fn read_at(&self, offset: ByteOffset, buf: &mut [u8]) -> Result<usize, LogError> {
        if let Some(handle) = self.handle.as_ref() {
            let opts = web_sys::FileSystemReadWriteOptions::new();
            opts.set_at(offset.0 as f64);
            handle
                .read_with_u8_array_and_options(buf, &opts)
                .map(|n| n as usize)
                .map_err(LogError::from)
        } else {
            let guard = self
                .fallback
                .read()
                .map_err(|_| LogError::Storage("Lock error".into()))?;
            let start = offset.0 as usize;
            if start >= guard.len() {
                return Ok(0);
            }
            let to_copy = (guard.len() - start).min(buf.len());
            buf[..to_copy].copy_from_slice(&guard[start..start + to_copy]);
            Ok(to_copy)
        }
    }

    fn write_at(&self, offset: ByteOffset, data: &[u8]) -> Result<usize, LogError> {
        if let Some(handle) = self.handle.as_ref() {
            let opts = web_sys::FileSystemReadWriteOptions::new();
            opts.set_at(offset.0 as f64);
            handle
                .write_with_u8_array_and_options(data, &opts)
                .map(|n| n as usize)
                .map_err(LogError::from)
        } else {
            let mut guard = self
                .fallback
                .write()
                .map_err(|_| LogError::Storage("Lock error".into()))?;
            let start = offset.0 as usize;
            if start + data.len() > guard.len() {
                guard.resize(start + data.len(), 0);
            }
            guard[start..start + data.len()].copy_from_slice(data);
            Ok(data.len())
        }
    }

    fn get_file_size(&self) -> Result<ByteOffset, LogError> {
        if let Some(handle) = self.handle.as_ref() {
            handle
                .get_size()
                .map(|s| ByteOffset(s as u64))
                .map_err(LogError::from)
        } else {
            let guard = self
                .fallback
                .read()
                .map_err(|_| LogError::Storage("Lock error".into()))?;
            Ok(ByteOffset(guard.len() as u64))
        }
    }

    fn truncate(&self, size: u64) -> Result<(), LogError> {
        if let Some(handle) = self.handle.as_ref() {
            handle
                .truncate_with_f64(size as f64)
                .map_err(LogError::from)
        } else {
            let mut guard = self
                .fallback
                .write()
                .map_err(|_| LogError::Storage("Lock error".into()))?;
            guard.truncate(size as usize);
            Ok(())
        }
    }

    fn flush(&self) -> Result<(), LogError> {
        if let Some(handle) = self.handle.as_ref() {
            handle.flush().map_err(LogError::from)
        } else {
            Ok(())
        }
    }
}

/// Log storage wrapper
pub struct LogStorage {
    pub backend: OpfsBackend,
}

impl LogStorage {
    pub fn new() -> Result<Self, LogError> {
        Ok(Self {
            backend: OpfsBackend {
                handle: None,
                fallback: std::sync::RwLock::new(Vec::new()),
            },
        })
    }
}

/// Gets the OPFS root directory handle
pub async fn get_opfs_root() -> Result<web_sys::FileSystemDirectoryHandle, JsValue> {
    let global = js_sys::global();
    let navigator = js_sys::Reflect::get(&global, &"navigator".into())?;
    if navigator.is_undefined() || navigator.is_null() {
        return Err(JsValue::from_str("navigator is unavailable"));
    }
    let storage = js_sys::Reflect::get(&navigator, &"storage".into())?;
    if storage.is_undefined() || storage.is_null() {
        return Err(JsValue::from_str(
            "navigator.storage is undefined (requires secure context)",
        ));
    }
    let storage: web_sys::StorageManager = storage.unchecked_into();
    let root = wasm_bindgen_futures::JsFuture::from(storage.get_directory()).await?;
    Ok(root.into())
}

/// Acquires a lock on a file handle with retries
pub async fn get_lock(
    file_handle: web_sys::FileSystemFileHandle,
) -> Result<web_sys::FileSystemSyncAccessHandle, JsValue> {
    for _ in 0..20 {
        match wasm_bindgen_futures::JsFuture::from(file_handle.create_sync_access_handle()).await {
            Ok(h) => return Ok(h.into()),
            Err(e) => {
                let error_name = js_sys::Reflect::get(&e, &"name".into()).unwrap_or_default();
                if error_name == "NoModificationAllowedError" || error_name == "InvalidStateError" {
                    gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err("Failed to acquire OPFS lock after retries".into())
}

/// Gets all log files from the root directory
async fn get_files(
    root: &web_sys::FileSystemDirectoryHandle,
) -> Result<Vec<(String, web_sys::FileSystemFileHandle)>, JsValue> {
    let mut files = Vec::new();
    let entries_fn = js_sys::Reflect::get(root, &"entries".into())?;
    let iterator = js_sys::Function::from(entries_fn)
        .call0(root)?
        .unchecked_into::<js_sys::AsyncIterator>();

    loop {
        let result = wasm_bindgen_futures::JsFuture::from(iterator.next()?).await?;
        let done = js_sys::Reflect::get(&result, &"done".into())?
            .as_bool()
            .unwrap_or(true);
        if done {
            break;
        }
        let value = js_sys::Reflect::get(&result, &"value".into())?;
        let entry = value.unchecked_into::<js_sys::Array>();
        let name = entry.get(0).as_string().unwrap_or_default();
        if name.starts_with("logs_") && name.ends_with(".txt") {
            let handle = entry
                .get(1)
                .unchecked_into::<web_sys::FileSystemFileHandle>();
            files.push((name, handle));
        }
    }

    files.sort_by(|a, b| {
        let ts_a = a.0[5..a.0.len() - 4].parse::<u64>().unwrap_or(0);
        let ts_b = b.0[5..b.0.len() - 4].parse::<u64>().unwrap_or(0);
        ts_b.cmp(&ts_a)
    });

    Ok(files)
}

/// Creates a new OPFS session
pub async fn new_session(
    root: &web_sys::FileSystemDirectoryHandle,
    cleanup_current: bool,
    current_filename: &mut Option<String>,
) -> Result<web_sys::FileSystemSyncAccessHandle, JsValue> {
    if cleanup_current {
        if let Some(name) = current_filename {
            let _ = wasm_bindgen_futures::JsFuture::from(root.remove_entry(name)).await;
        }
    }

    let filename = format!("logs_{}.txt", chrono::Utc::now().timestamp_millis());
    let opts = web_sys::FileSystemGetFileOptions::new();
    opts.set_create(true);
    let file_handle =
        wasm_bindgen_futures::JsFuture::from(root.get_file_handle_with_options(&filename, &opts))
            .await?;
    let file_handle: web_sys::FileSystemFileHandle = file_handle.into();

    let lock = get_lock(file_handle).await?;
    *current_filename = Some(filename);
    Ok(lock)
}

/// Initializes an OPFS session, reusing existing file if possible
pub async fn init_opfs_session(
    current_filename: &mut Option<String>,
) -> Result<web_sys::FileSystemSyncAccessHandle, JsValue> {
    let root = get_opfs_root().await?;
    let files = get_files(&root).await?;

    if let Some((name, handle)) = files.first().cloned() {
        match get_lock(handle).await {
            Ok(lock) => {
                *current_filename = Some(name);
                // Cleanup others
                for file in files.iter().skip(1) {
                    let _ = wasm_bindgen_futures::JsFuture::from(root.remove_entry(&file.0)).await;
                }
                Ok(lock)
            }
            Err(_) => {
                // If lock fails, start new
                let res = new_session(&root, false, current_filename).await;
                // Cleanup all including the failed one
                for file in &files {
                    let _ = wasm_bindgen_futures::JsFuture::from(root.remove_entry(&file.0)).await;
                }
                res
            }
        }
    } else {
        new_session(&root, false, current_filename).await
    }
}
