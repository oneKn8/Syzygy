//! Settings and Preferences Storage

use crate::storage::get_db;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

/// A user setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub key: String,
    pub value: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Project metadata stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_opened: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
}

/// Recent file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
    pub opened_at: DateTime<Utc>,
}

/// Pipeline template for saving/loading pipeline configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTemplate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub nodes: Vec<serde_json::Value>,
    pub edges: Vec<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// === Settings Commands ===

/// Get a setting by key
#[tauri::command]
pub async fn get_setting(key: String) -> Result<Option<serde_json::Value>, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let mut response = db
        .db
        .query("SELECT value FROM settings WHERE key = $key")
        .bind(("key", key))
        .await
        .map_err(|e| e.to_string())?;

    let result: Option<Setting> = response.take(0).map_err(|e| e.to_string())?;
    Ok(result.map(|s| s.value))
}

/// Set a setting value
#[tauri::command]
pub async fn set_setting(key: String, value: serde_json::Value) -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    // Upsert the setting
    db.db
        .query(
            r#"
            UPSERT settings SET
                key = $key,
                value = $value,
                updated_at = time::now()
            WHERE key = $key
        "#,
        )
        .bind(("key", key))
        .bind(("value", value))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Delete a setting
#[tauri::command]
pub async fn delete_setting(key: String) -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    db.db
        .query("DELETE FROM settings WHERE key = $key")
        .bind(("key", key))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get all settings
#[tauri::command]
pub async fn get_all_settings() -> Result<Vec<Setting>, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let settings: Vec<Setting> = db.db.select("settings").await.map_err(|e| e.to_string())?;

    Ok(settings)
}

// === Project Commands ===

/// Register a project in the database
#[tauri::command]
pub async fn register_project(
    name: String,
    path: String,
    description: Option<String>,
    template: Option<String>,
) -> Result<ProjectMeta, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    // Check if project already exists
    let mut response = db
        .db
        .query("SELECT * FROM project WHERE path = $path")
        .bind(("path", path.clone()))
        .await
        .map_err(|e| e.to_string())?;

    let existing: Option<ProjectMeta> = response.take(0).map_err(|e| e.to_string())?;

    if let Some(mut project) = existing {
        // Update last_opened
        project.last_opened = Utc::now();
        let _: Option<ProjectMeta> = db
            .db
            .update(("project", project.id.clone().unwrap().id.to_string()))
            .merge(serde_json::json!({
                "last_opened": Utc::now(),
            }))
            .await
            .map_err(|e| e.to_string())?;
        return Ok(project);
    }

    // Create new project entry
    let project: Option<ProjectMeta> = db
        .db
        .create("project")
        .content(ProjectMeta {
            id: None,
            name,
            path,
            description,
            template,
            created_at: Utc::now(),
            last_opened: Utc::now(),
            settings: None,
        })
        .await
        .map_err(|e| e.to_string())?;

    project.ok_or_else(|| "Failed to register project".to_string())
}

/// Get recent projects
#[tauri::command]
pub async fn get_recent_projects(limit: Option<usize>) -> Result<Vec<ProjectMeta>, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;
    let limit = limit.unwrap_or(10);

    let mut response = db
        .db
        .query(format!(
            "SELECT * FROM project ORDER BY last_opened DESC LIMIT {}",
            limit
        ))
        .await
        .map_err(|e| e.to_string())?;

    let projects: Vec<ProjectMeta> = response.take(0).map_err(|e| e.to_string())?;
    Ok(projects)
}

/// Update project settings
#[tauri::command]
pub async fn update_project_settings(path: String, settings: serde_json::Value) -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    db.db
        .query("UPDATE project SET settings = $settings WHERE path = $path")
        .bind(("path", path))
        .bind(("settings", settings))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Remove a project from database
#[tauri::command]
pub async fn remove_project(path: String) -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    db.db
        .query("DELETE FROM project WHERE path = $path")
        .bind(("path", path))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// === Recent Files Commands ===

/// Add a file to recent files
#[tauri::command]
pub async fn add_recent_file(path: String, project_path: Option<String>) -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    // Upsert the recent file
    db.db
        .query(
            r#"
            UPSERT recent_file SET
                path = $path,
                project_path = $project_path,
                opened_at = time::now()
            WHERE path = $path
        "#,
        )
        .bind(("path", path))
        .bind(("project_path", project_path))
        .await
        .map_err(|e| e.to_string())?;

    // Limit to 50 recent files
    db.db
        .query(
            r#"
            DELETE FROM recent_file WHERE id NOT IN
            (SELECT id FROM recent_file ORDER BY opened_at DESC LIMIT 50)
        "#,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get recent files
#[tauri::command]
pub async fn get_recent_files(limit: Option<usize>) -> Result<Vec<RecentFile>, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;
    let limit = limit.unwrap_or(20);

    let mut response = db
        .db
        .query(format!(
            "SELECT * FROM recent_file ORDER BY opened_at DESC LIMIT {}",
            limit
        ))
        .await
        .map_err(|e| e.to_string())?;

    let files: Vec<RecentFile> = response.take(0).map_err(|e| e.to_string())?;
    Ok(files)
}

/// Clear recent files
#[tauri::command]
pub async fn clear_recent_files() -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    db.db
        .query("DELETE FROM recent_file")
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// === Pipeline Template Commands ===

/// Save a pipeline template
#[tauri::command]
pub async fn save_pipeline_template(
    name: String,
    description: Option<String>,
    nodes: Vec<serde_json::Value>,
    edges: Vec<serde_json::Value>,
) -> Result<PipelineTemplate, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let template: Option<PipelineTemplate> = db
        .db
        .create("pipeline_template")
        .content(PipelineTemplate {
            id: None,
            name,
            description,
            nodes,
            edges,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await
        .map_err(|e| e.to_string())?;

    template.ok_or_else(|| "Failed to save pipeline template".to_string())
}

/// Get all pipeline templates
#[tauri::command]
pub async fn get_pipeline_templates() -> Result<Vec<PipelineTemplate>, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let templates: Vec<PipelineTemplate> = db
        .db
        .select("pipeline_template")
        .await
        .map_err(|e| e.to_string())?;

    Ok(templates)
}

/// Delete a pipeline template
#[tauri::command]
pub async fn delete_pipeline_template(id: String) -> Result<(), String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let _: Option<PipelineTemplate> = db
        .db
        .delete(("pipeline_template", id.as_str()))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Load a pipeline template by ID
#[tauri::command]
pub async fn load_pipeline_template(id: String) -> Result<PipelineTemplate, String> {
    let db = get_db().ok_or("Database not initialized")?;
    let db = db.read().await;

    let template: Option<PipelineTemplate> = db
        .db
        .select(("pipeline_template", id.as_str()))
        .await
        .map_err(|e| e.to_string())?;

    template.ok_or_else(|| "Pipeline template not found".to_string())
}
