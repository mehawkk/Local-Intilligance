mod commands;
mod config;
mod crawler;
mod db;
mod indexer;
mod parser;
mod search;
mod watcher;

use db::models::IndexStatus;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use watcher::WatcherHandle;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub db_arc: Arc<Mutex<Connection>>,
    pub index_status: Mutex<IndexStatus>,
    pub index_status_arc: Arc<Mutex<IndexStatus>>,
    pub watcher_handle: Mutex<Option<WatcherHandle>>,
}

// AppState needs to be Send + Sync for Tauri. The Mutex wrapping makes this safe.
// rusqlite::Connection is Send but not Sync, but Mutex<Connection> is both.
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Determine app data directory for the database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));

            // Ensure the directory exists
            std::fs::create_dir_all(&app_data_dir).ok();

            let db_path = app_data_dir.join("file_index.db");
            log::info!("Database path: {}", db_path.display());

            // Initialize database
            let conn =
                db::init_db(&db_path).expect("Failed to initialize database");

            // Create shared state using Arc for the parts that need to be shared
            // with background threads (indexer, watcher)
            let db_arc = Arc::new(Mutex::new(
                db::init_db(&db_path).expect("Failed to open second DB connection"),
            ));
            let index_status = IndexStatus::default();
            let index_status_arc = Arc::new(Mutex::new(index_status.clone()));

            let state = AppState {
                db: Mutex::new(conn),
                db_arc: db_arc.clone(),
                index_status: Mutex::new(index_status),
                index_status_arc: index_status_arc.clone(),
                watcher_handle: Mutex::new(None),
            };

            app.manage(state);

            // Start file watchers for existing indexed roots
            let app_handle = app.handle().clone();
            let db_for_watcher = db_arc.clone();

            std::thread::spawn(move || {
                start_watchers(&app_handle, &db_for_watcher);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::folders::add_indexed_folder,
            commands::folders::remove_indexed_folder,
            commands::folders::list_indexed_folders,
            commands::exclusions::add_exclusion,
            commands::exclusions::remove_exclusion,
            commands::exclusions::list_exclusions,
            commands::indexing::start_scan,
            commands::indexing::trigger_rescan,
            commands::indexing::get_index_status,
            commands::search::search_files,
            commands::files::open_file,
            commands::files::open_containing_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Start file watchers for all indexed roots.
fn start_watchers(app_handle: &tauri::AppHandle, db: &Arc<Mutex<Connection>>) {
    let roots: Vec<(i64, String)> = {
        let conn = db.lock().unwrap();
        let mut stmt = match conn.prepare("SELECT id, path FROM indexed_roots") {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to query roots: {}", e);
                return;
            }
        };
        let rows = match stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?))) {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to fetch roots: {}", e);
                return;
            }
        };
        let mut result = Vec::new();
        for row in rows {
            if let Ok(val) = row {
                result.push(val);
            }
        }
        result
    };

    if roots.is_empty() {
        return;
    }

    let paths: Vec<(i64, PathBuf)> = roots
        .into_iter()
        .filter(|(_, p)| PathBuf::from(p).exists())
        .map(|(id, p)| (id, PathBuf::from(p)))
        .collect();

    if paths.is_empty() {
        return;
    }

    match watcher::start_watching(app_handle.clone(), db.clone(), paths) {
        Ok(handle) => {
            // Store the watcher handle in AppState
            let state = app_handle.state::<AppState>();
            let mut watcher = state.watcher_handle.lock().unwrap();
            *watcher = Some(handle);
            log::info!("File watchers started successfully");
        }
        Err(e) => {
            log::error!("Failed to start file watchers: {}", e);
        }
    }
}
