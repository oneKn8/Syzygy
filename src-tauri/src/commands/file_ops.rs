//! File operation commands for the frontend

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub extension: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub language: String,
}

/// Read directory contents
#[tauri::command]
pub async fn read_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    let mut entries = Vec::new();

    let read_dir = fs::read_dir(&path).map_err(|e| e.to_string())?;

    for entry in read_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files
        if file_name.starts_with('.') {
            continue;
        }

        let extension = if metadata.is_file() {
            entry
                .path()
                .extension()
                .map(|e| e.to_string_lossy().to_string())
        } else {
            None
        };

        entries.push(FileEntry {
            name: file_name,
            path: entry.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            extension,
        });
    }

    // Sort: directories first, then files, both alphabetically
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

/// Read file content
#[tauri::command]
pub async fn read_file(path: String) -> Result<FileContent, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if !path_buf.is_file() {
        return Err(format!("Path is not a file: {}", path));
    }

    let content = fs::read_to_string(&path_buf).map_err(|e| e.to_string())?;

    let language = detect_language(&path_buf);

    Ok(FileContent {
        path,
        content,
        language,
    })
}

/// Write file content
#[tauri::command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| e.to_string())
}

/// Create a new file
#[tauri::command]
pub async fn create_file(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);

    if path_buf.exists() {
        return Err(format!("File already exists: {}", path));
    }

    // Create parent directories if they don't exist
    if let Some(parent) = path_buf.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::write(&path, "").map_err(|e| e.to_string())
}

/// Create a new directory
#[tauri::command]
pub async fn create_directory(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| e.to_string())
}

/// Delete a file or directory
#[tauri::command]
pub async fn delete_path(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if path_buf.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| e.to_string())
    } else {
        fs::remove_file(&path).map_err(|e| e.to_string())
    }
}

/// Rename a file or directory
#[tauri::command]
pub async fn rename_path(old_path: String, new_path: String) -> Result<(), String> {
    fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

fn detect_language(path: &PathBuf) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("typ") => "typst".to_string(),
        Some("tex") | Some("latex") => "latex".to_string(),
        Some("md") | Some("markdown") => "markdown".to_string(),
        Some("json") => "json".to_string(),
        Some("yaml") | Some("yml") => "yaml".to_string(),
        Some("toml") => "toml".to_string(),
        Some("rs") => "rust".to_string(),
        Some("ts") | Some("tsx") => "typescript".to_string(),
        Some("js") | Some("jsx") => "javascript".to_string(),
        Some("css") => "css".to_string(),
        Some("html") => "html".to_string(),
        _ => "plaintext".to_string(),
    }
}
