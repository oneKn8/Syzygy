//! Syzygy - RFP and proposal authoring tool
//!
//! A document-engineering platform for RFPs and proposals, with:
//! - Typst and LaTeX support
//! - Visual document pipelines
//! - AI-powered assistance
//! - RFP analysis and compliance tracking

use tauri::Manager;

// Core modules
mod ai;
mod commands;
mod engine;
mod rfp;
mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // File operations
            commands::read_directory,
            commands::read_file,
            commands::write_file,
            commands::create_file,
            commands::create_directory,
            commands::delete_path,
            commands::rename_path,
            // Project management
            commands::get_project_info,
            commands::init_project,
            // Templates
            commands::get_templates,
            commands::create_from_template,
            // Document compilation
            engine::compile_typst,
            engine::compile_typst_source,
            engine::get_typst_version,
            // Pipeline execution
            engine::run_pipeline,
            engine::run_pipeline_stream,
            engine::validate_pipeline,
            engine::create_pipeline,
            // Export commands
            engine::export_to_format,
            engine::get_export_formats,
            engine::batch_export,
            // AI commands
            ai::check_ollama_status,
            ai::list_ollama_models,
            ai::chat_with_ollama,
            ai::chat_with_ollama_stream,
            ai::generate_completion,
            ai::suggest_improvements,
            ai::analyze_rfp,
            ai::set_ollama_url,
            // Content Library commands
            storage::create_content_block,
            storage::get_content_block,
            storage::update_content_block,
            storage::delete_content_block,
            storage::list_content_blocks,
            storage::increment_block_usage,
            storage::search_content_blocks,
            storage::get_popular_blocks,
            storage::get_recent_blocks,
            storage::store_block_embedding,
            storage::get_content_stats,
            // Category commands
            storage::create_category,
            storage::list_categories,
            storage::delete_category,
            storage::get_blocks_by_category,
            // Settings commands
            storage::get_setting,
            storage::set_setting,
            storage::delete_setting,
            storage::get_all_settings,
            // Project metadata commands
            storage::register_project,
            storage::get_recent_projects,
            storage::update_project_settings,
            storage::remove_project,
            // Recent files commands
            storage::add_recent_file,
            storage::get_recent_files,
            storage::clear_recent_files,
            // Pipeline template commands
            storage::save_pipeline_template,
            storage::get_pipeline_templates,
            storage::delete_pipeline_template,
            storage::load_pipeline_template,
            // RFP Analysis commands
            rfp::analyze_rfp_document,
            rfp::extract_rfp_requirements,
            rfp::extract_rfp_deadlines,
            rfp::extract_rfp_critical_terms,
            rfp::update_requirement_status,
            // Compliance Matrix commands
            rfp::generate_matrix,
            rfp::update_compliance_entry,
            rfp::analyze_gaps,
            rfp::export_matrix,
            rfp::get_compliance_summary,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize modules
            storage::init_storage();
            ai::init_ai();
            rfp::init_rfp();

            // Initialize database asynchronously
            tauri::async_runtime::spawn(async move {
                match storage::init_database().await {
                    Ok(_) => log::info!("Database initialized successfully"),
                    Err(e) => log::error!("Failed to initialize database: {}", e),
                }
            });

            let window = app.get_webview_window("main").unwrap();
            window.set_title("Syzygy - RFP & Proposal Authoring")?;

            log::info!("Syzygy initialized successfully");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
