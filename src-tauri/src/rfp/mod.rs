//! RFP analysis and compliance tracking
//!
//! This module provides RFP-specific features:
//! - Document analysis and requirement extraction
//! - Compliance matrix generation
//! - Critical data highlighting
//! - Gap analysis

pub mod analyzer;
pub mod compliance;

pub use analyzer::*;
pub use compliance::*;

/// Initialize the RFP module
pub fn init_rfp() {
    log::info!("RFP module initialized");
}
