//! Content Library - Reusable document blocks storage

use crate::storage::get_db;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

/// A reusable content block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub title: String,
    pub content: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub usage_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

fn default_format() -> String {
    "typst".to_string()
}

/// Content category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// Input for creating/updating a content block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlockInput {
    pub title: String,
    pub content: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_format")]
    pub format: String,
}

/// Search parameters for content blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSearchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

impl Default for ContentSearchParams {
    fn default() -> Self {
        Self {
            query: None,
            category: None,
            tags: None,
            limit: 50,
            offset: 0,
        }
    }
}

// === Tauri Commands ===

/// Create a new content block
#[tauri::command]
pub async fn create_content_block(input: ContentBlockInput) -> Result<ContentBlock, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let block: Option<ContentBlock> = db
        .db
        .create("content_block")
        .content(ContentBlock {
            id: None,
            title: input.title,
            content: input.content,
            category: input.category,
            tags: input.tags,
            format: input.format,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_count: 0,
            embedding: None,
        })
        .await
        .map_err(|e| e.to_string())?;

    block.ok_or_else(|| "Failed to create content block".to_string())
}

/// Get a content block by ID
#[tauri::command]
pub async fn get_content_block(id: String) -> Result<ContentBlock, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let block: Option<ContentBlock> = db
        .db
        .select(("content_block", id.as_str()))
        .await
        .map_err(|e| e.to_string())?;

    block.ok_or_else(|| "Content block not found".to_string())
}

/// Update a content block
#[tauri::command]
pub async fn update_content_block(id: String, input: ContentBlockInput) -> Result<ContentBlock, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let block: Option<ContentBlock> = db
        .db
        .update(("content_block", id.as_str()))
        .merge(serde_json::json!({
            "title": input.title,
            "content": input.content,
            "category": input.category,
            "tags": input.tags,
            "format": input.format,
            "updated_at": Utc::now(),
        }))
        .await
        .map_err(|e| e.to_string())?;

    block.ok_or_else(|| "Failed to update content block".to_string())
}

/// Delete a content block
#[tauri::command]
pub async fn delete_content_block(id: String) -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let _: Option<ContentBlock> = db
        .db
        .delete(("content_block", id.as_str()))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all content blocks with optional filtering
#[tauri::command]
pub async fn list_content_blocks(params: Option<ContentSearchParams>) -> Result<Vec<ContentBlock>, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;
    let params = params.unwrap_or_default();

    let mut query = String::from("SELECT * FROM content_block");
    let mut conditions = Vec::new();

    if let Some(ref cat) = params.category {
        conditions.push(format!("category = '{}'", cat.replace('\'', "''")));
    }

    if let Some(ref q) = params.query {
        let escaped = q.replace('\'', "''");
        conditions.push(format!(
            "(title CONTAINS '{}' OR content CONTAINS '{}')",
            escaped, escaped
        ));
    }

    if let Some(ref tags) = params.tags {
        if !tags.is_empty() {
            let tag_conditions: Vec<String> = tags
                .iter()
                .map(|t| format!("'{}' IN tags", t.replace('\'', "''")))
                .collect();
            conditions.push(format!("({})", tag_conditions.join(" OR ")));
        }
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY updated_at DESC");
    query.push_str(&format!(" LIMIT {} START {}", params.limit, params.offset));

    let mut response = db.db.query(&query).await.map_err(|e| e.to_string())?;
    let blocks: Vec<ContentBlock> = response.take(0).map_err(|e| e.to_string())?;

    Ok(blocks)
}

/// Increment usage count for a content block
#[tauri::command]
pub async fn increment_block_usage(id: String) -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    db.db
        .query("UPDATE content_block SET usage_count += 1 WHERE id = $id")
        .bind(("id", format!("content_block:{}", id)))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// === Category Commands ===

/// Create a new category
#[tauri::command]
pub async fn create_category(
    name: String,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<Category, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let category: Option<Category> = db
        .db
        .create("category")
        .content(Category {
            id: None,
            name,
            description,
            color,
            icon,
        })
        .await
        .map_err(|e| e.to_string())?;

    category.ok_or_else(|| "Failed to create category".to_string())
}

/// List all categories
#[tauri::command]
pub async fn list_categories() -> Result<Vec<Category>, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let categories: Vec<Category> = db
        .db
        .select("category")
        .await
        .map_err(|e| e.to_string())?;

    Ok(categories)
}

/// Delete a category
#[tauri::command]
pub async fn delete_category(id: String) -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let _: Option<Category> = db
        .db
        .delete(("category", id.as_str()))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get content blocks by category
#[tauri::command]
pub async fn get_blocks_by_category(category: String) -> Result<Vec<ContentBlock>, String> {
    list_content_blocks(Some(ContentSearchParams {
        category: Some(category),
        ..Default::default()
    }))
    .await
}

/// Search content blocks by text query
#[tauri::command]
pub async fn search_content_blocks(query: String, limit: Option<usize>) -> Result<Vec<ContentBlock>, String> {
    list_content_blocks(Some(ContentSearchParams {
        query: Some(query),
        limit: limit.unwrap_or(20),
        ..Default::default()
    }))
    .await
}

/// Get most used content blocks
#[tauri::command]
pub async fn get_popular_blocks(limit: Option<usize>) -> Result<Vec<ContentBlock>, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;
    let limit = limit.unwrap_or(10);

    let mut response = db
        .db
        .query(format!(
            "SELECT * FROM content_block ORDER BY usage_count DESC LIMIT {}",
            limit
        ))
        .await
        .map_err(|e| e.to_string())?;

    let blocks: Vec<ContentBlock> = response.take(0).map_err(|e| e.to_string())?;
    Ok(blocks)
}

/// Get recently updated content blocks
#[tauri::command]
pub async fn get_recent_blocks(limit: Option<usize>) -> Result<Vec<ContentBlock>, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;
    let limit = limit.unwrap_or(10);

    let mut response = db
        .db
        .query(format!(
            "SELECT * FROM content_block ORDER BY updated_at DESC LIMIT {}",
            limit
        ))
        .await
        .map_err(|e| e.to_string())?;

    let blocks: Vec<ContentBlock> = response.take(0).map_err(|e| e.to_string())?;
    Ok(blocks)
}

/// Store embedding for a content block (for semantic search)
#[tauri::command]
pub async fn store_block_embedding(id: String, embedding: Vec<f32>) -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    db.db
        .query("UPDATE content_block SET embedding = $embedding WHERE id = $id")
        .bind(("id", format!("content_block:{}", id)))
        .bind(("embedding", embedding))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get content block statistics
#[tauri::command]
pub async fn get_content_stats() -> Result<serde_json::Value, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let mut response = db
        .db
        .query(
            r#"
            SELECT
                count() as total_blocks,
                math::sum(usage_count) as total_usage,
                array::group(category) as categories
            FROM content_block GROUP ALL
        "#,
        )
        .await
        .map_err(|e| e.to_string())?;

    let stats: Option<serde_json::Value> = response.take(0).map_err(|e| e.to_string())?;
    Ok(stats.unwrap_or(serde_json::json!({
        "total_blocks": 0,
        "total_usage": 0,
        "categories": []
    })))
}
