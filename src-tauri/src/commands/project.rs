//! Project management commands

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub doc_type: String, // "typst" or "latex"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub doc_type: String,
    pub main_file: String,
    pub output_dir: String,
}

/// Get project info from a directory
#[tauri::command]
pub async fn get_project_info(path: String) -> Result<ProjectInfo, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() || !path_buf.is_dir() {
        return Err("Invalid project path".to_string());
    }

    let name = path_buf
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string());

    // Detect project type by looking for main files
    let doc_type = if path_buf.join("main.typ").exists() {
        "typst"
    } else if path_buf.join("main.tex").exists() {
        "latex"
    } else {
        // Check for any .typ or .tex files
        let has_typst = fs::read_dir(&path_buf)
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "typ")
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if has_typst {
            "typst"
        } else {
            "latex"
        }
    };

    Ok(ProjectInfo {
        name,
        path,
        doc_type: doc_type.to_string(),
    })
}

/// Initialize a new project
#[tauri::command]
pub async fn init_project(path: String, name: String, doc_type: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);

    // Create project directory
    fs::create_dir_all(&path_buf).map_err(|e| e.to_string())?;

    // Create main file based on doc type
    let main_file = match doc_type.as_str() {
        "typst" => {
            let content = format!(
                r#"#set document(title: "{}")
#set page(paper: "us-letter", margin: 1in)
#set text(font: "New Computer Modern", size: 11pt)

= {}

Your content here.
"#,
                name, name
            );
            let main_path = path_buf.join("main.typ");
            fs::write(&main_path, content).map_err(|e| e.to_string())?;
            "main.typ"
        }
        "latex" => {
            let content = format!(
                r#"\documentclass[11pt]{{article}}
\usepackage[utf8]{{inputenc}}
\usepackage{{geometry}}
\geometry{{letterpaper, margin=1in}}

\title{{{}}}
\author{{}}
\date{{\today}}

\begin{{document}}

\maketitle

Your content here.

\end{{document}}
"#,
                name
            );
            let main_path = path_buf.join("main.tex");
            fs::write(&main_path, content).map_err(|e| e.to_string())?;
            "main.tex"
        }
        _ => return Err(format!("Unknown document type: {}", doc_type)),
    };

    // Create output directory
    fs::create_dir_all(path_buf.join("output")).map_err(|e| e.to_string())?;

    // Create project config
    let config = ProjectConfig {
        name: name.clone(),
        doc_type,
        main_file: main_file.to_string(),
        output_dir: "output".to_string(),
    };

    let config_path = path_buf.join("rfpmaker.json");
    let config_json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, config_json).map_err(|e| e.to_string())?;

    Ok(())
}
