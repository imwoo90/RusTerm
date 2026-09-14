//! Monitor hooks sub-module catalog (mod.rs = index.md).
//!
//! Contains specialized reactive hooks for virtual scrolling calculation, data window requests,
//! and resize observers sustaining 60fps rendering over large log histories.

pub mod data_request;
pub mod effects;
pub mod virtual_scroll;
