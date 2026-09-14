//! # Monitor Hooks Catalog (index.md)
//!
//! ## Overview
//! Contains specialized reactive hooks for virtual scrolling calculation, data window requests, and resize observers sustaining 60fps rendering over large log histories.
//!
//! ## Submodules
//! - [`data_request`]: Asynchronous chunk request hook fetching log ranges from background worker.
//! - [`effects`]: Reactive DOM side-effect handlers for autoscrolling and viewport resizing.
//! - [`virtual_scroll`]: Calculation engine tracking visible window start/end indices during scrolling.
//!
//! ## Search Tags
//! #monitor-hooks, #virtual-scroll, #data-fetching, #observers

pub mod data_request;
pub mod effects;
pub mod virtual_scroll;
