//! Database connection and management

use directories::ProjectDirs;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Failed to connect to database: {0}")]
    ConnectionError(String),
    #[error("Database query error: {0}")]
    QueryError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Record not found: {0}")]
    NotFound(String),
    #[error("Database path error: {0}")]
    PathError(String),
}

impl From<surrealdb::Error> for DatabaseError {
    fn from(err: surrealdb::Error) -> Self {
        DatabaseError::QueryError(err.to_string())
    }
}

pub struct Database {
    pub db: Surreal<Db>,
}

impl Database {
    /// Create a new database connection
    pub async fn new() -> Result<Self, DatabaseError> {
        let db_path = Self::get_database_path()?;

        // Ensure the parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DatabaseError::PathError(format!("Failed to create database directory: {}", e))
            })?;
        }

        log::info!("Opening database at: {:?}", db_path);

        let db = Surreal::new::<RocksDb>(db_path)
            .await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        // Select namespace and database
        db.use_ns("syzygy")
            .use_db("main")
            .await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        let database = Self { db };

        // Initialize schema
        database.init_schema().await?;

        Ok(database)
    }

    /// Get the database path
    fn get_database_path() -> Result<PathBuf, DatabaseError> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "syzygy", "Syzygy") {
            Ok(proj_dirs.data_dir().join("database"))
        } else {
            // Fallback to home directory
            let home = std::env::var("HOME")
                .map_err(|_| DatabaseError::PathError("HOME not set".to_string()))?;
            Ok(PathBuf::from(home).join(".syzygy").join("database"))
        }
    }

    /// Initialize database schema
    async fn init_schema(&self) -> Result<(), DatabaseError> {
        // Content blocks table
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS content_block SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS title ON content_block TYPE string;
                DEFINE FIELD IF NOT EXISTS content ON content_block TYPE string;
                DEFINE FIELD IF NOT EXISTS category ON content_block TYPE string;
                DEFINE FIELD IF NOT EXISTS tags ON content_block TYPE array<string>;
                DEFINE FIELD IF NOT EXISTS format ON content_block TYPE string DEFAULT 'typst';
                DEFINE FIELD IF NOT EXISTS created_at ON content_block TYPE datetime DEFAULT time::now();
                DEFINE FIELD IF NOT EXISTS updated_at ON content_block TYPE datetime DEFAULT time::now();
                DEFINE FIELD IF NOT EXISTS usage_count ON content_block TYPE int DEFAULT 0;
                DEFINE FIELD IF NOT EXISTS embedding ON content_block TYPE option<array<float>>;
                DEFINE INDEX IF NOT EXISTS content_block_title ON content_block FIELDS title;
                DEFINE INDEX IF NOT EXISTS content_block_category ON content_block FIELDS category;
                DEFINE INDEX IF NOT EXISTS content_block_tags ON content_block FIELDS tags;
            "#,
            )
            .await?;

        // Projects metadata table
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS project SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS name ON project TYPE string;
                DEFINE FIELD IF NOT EXISTS path ON project TYPE string;
                DEFINE FIELD IF NOT EXISTS description ON project TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS template ON project TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS created_at ON project TYPE datetime DEFAULT time::now();
                DEFINE FIELD IF NOT EXISTS last_opened ON project TYPE datetime DEFAULT time::now();
                DEFINE FIELD IF NOT EXISTS settings ON project TYPE option<object>;
                DEFINE INDEX IF NOT EXISTS project_path ON project FIELDS path UNIQUE;
            "#,
            )
            .await?;

        // User settings table
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS settings SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS key ON settings TYPE string;
                DEFINE FIELD IF NOT EXISTS value ON settings TYPE any;
                DEFINE FIELD IF NOT EXISTS updated_at ON settings TYPE datetime DEFAULT time::now();
                DEFINE INDEX IF NOT EXISTS settings_key ON settings FIELDS key UNIQUE;
            "#,
            )
            .await?;

        // Content categories table
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS category SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS name ON category TYPE string;
                DEFINE FIELD IF NOT EXISTS description ON category TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS color ON category TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS icon ON category TYPE option<string>;
                DEFINE INDEX IF NOT EXISTS category_name ON category FIELDS name UNIQUE;
            "#,
            )
            .await?;

        // Recent files table
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS recent_file SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS path ON recent_file TYPE string;
                DEFINE FIELD IF NOT EXISTS project_path ON recent_file TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS opened_at ON recent_file TYPE datetime DEFAULT time::now();
                DEFINE INDEX IF NOT EXISTS recent_file_path ON recent_file FIELDS path UNIQUE;
            "#,
            )
            .await?;

        // Pipeline templates table
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS pipeline_template SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS name ON pipeline_template TYPE string;
                DEFINE FIELD IF NOT EXISTS description ON pipeline_template TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS nodes ON pipeline_template TYPE array;
                DEFINE FIELD IF NOT EXISTS edges ON pipeline_template TYPE array;
                DEFINE FIELD IF NOT EXISTS created_at ON pipeline_template TYPE datetime DEFAULT time::now();
                DEFINE FIELD IF NOT EXISTS updated_at ON pipeline_template TYPE datetime DEFAULT time::now();
                DEFINE INDEX IF NOT EXISTS pipeline_template_name ON pipeline_template FIELDS name UNIQUE;
            "#,
            )
            .await?;

        log::info!("Database schema initialized");
        Ok(())
    }

    /// Run a raw query
    pub async fn query(&self, query: &str) -> Result<surrealdb::Response, DatabaseError> {
        Ok(self.db.query(query).await?)
    }
}
