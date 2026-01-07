//! Storage layer using SurrealDB
//!
//! This module provides persistent storage for:
//! - Content library (reusable document blocks)
//! - Project metadata
//! - User settings and preferences
//! - Vector embeddings for semantic search

mod database;
mod content_library;
mod settings;

pub use database::*;
pub use content_library::*;
pub use settings::*;

use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::RwLock;

static DB: OnceCell<Arc<RwLock<Database>>> = OnceCell::new();

/// Initialize the storage module
pub fn init_storage() {
    log::info!("Initializing storage module...");

    // Database initialization happens lazily on first access
    // This allows the app to start even if the database fails to initialize
    log::info!("Storage module initialized");
}

/// Get a reference to the database
pub fn get_db() -> Option<Arc<RwLock<Database>>> {
    DB.get().cloned()
}

/// Initialize database connection (called async from setup)
pub async fn init_database() -> Result<(), String> {
    let db = Database::new().await.map_err(|e| e.to_string())?;
    let db = Arc::new(RwLock::new(db));

    DB.set(db).map_err(|_| "Database already initialized".to_string())?;

    log::info!("Database connection established");
    Ok(())
}
