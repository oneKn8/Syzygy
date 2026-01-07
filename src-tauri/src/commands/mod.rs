//! Tauri IPC command handlers
//!
//! This module contains all the command handlers that are exposed to the frontend
//! via Tauri's IPC mechanism.

pub mod file_ops;
pub mod project;
pub mod templates;

pub use file_ops::*;
pub use project::*;
pub use templates::*;
