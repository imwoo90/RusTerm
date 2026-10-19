//! # Monitor Hooks Catalog (index.md)
//!
//! ## Overview
//! Contains specialized reactive hooks for virtual scrolling calculation, data window requests, and resize observers sustaining 60fps rendering over large log histories.
//!
//! ## Submodules
//! - [`data_request`] (`data_request.rs`): Asynchronous chunk request hook fetching log ranges from background worker.
//! - [`effects`] (`effects.rs`): Reactive DOM side-effect handlers for autoscrolling and viewport resizing.
//! - [`virtual_scroll`] (`virtual_scroll.rs`): Calculation engine tracking visible window start/end indices during scrolling.
//!
//! ## Search Tags
//! #monitor-hooks, #virtual-scroll, #data-fetching, #observers

pub mod data_request;
pub mod effects;
pub mod virtual_scroll;
