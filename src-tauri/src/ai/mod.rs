//! AI integration layer
//!
//! This module handles AI capabilities:
//! - Local LLM via Ollama
//! - Cloud AI via Claude API
//! - Document embeddings for semantic search

pub mod ollama;

pub use ollama::*;

pub fn init_ai() {
    log::info!("AI module initialized");
}
