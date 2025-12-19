pub mod agent;
pub mod browser;
pub mod commands;
pub mod config;
pub mod memory;
pub mod search;
pub mod tracing;

use browser::BrowserManager;
use config::ConfigManager;
use std::sync::{Mutex, OnceLock};
use tauri::Manager;

pub static GLOBAL_APP: OnceLock<tauri::AppHandle> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing first - before anything else
    tracing::init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(
            tauri_plugin_sql::Builder::new()
                .add_migrations("sqlite:traces.db", tracing::get_migrations())
                .build(),
        )
        .setup(|app| {
            let _ = GLOBAL_APP.set(app.handle().clone());

            crate::trace_info!("nexus::init", "Nexus application starting");

            memory::init_memory();
            crate::trace_debug!("nexus::init", "Memory system initialized");

            let config_manager = ConfigManager::new(app.handle());
            app.manage(Mutex::new(config_manager));
            crate::trace_debug!("nexus::init", "Config manager initialized");

            let browser =
                match tauri::async_runtime::block_on(async { BrowserManager::new().await }) {
                    Ok(b) => {
                        crate::trace_info!("nexus::init", "Browser launched successfully");
                        b
                    }
                    Err(e) => {
                        crate::trace_error!(
                            "nexus::init",
                            "Browser launch failed",
                            error = e.to_string()
                        );
                        return Err(e.into());
                    }
                };

            // Set global instance for agent tools
            let _ = browser::GLOBAL_BROWSER.set(browser.clone());

            app.manage(browser);

            crate::trace_info!("nexus::init", "Nexus initialization complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::fetch_and_search,
            commands::run_agent,
            commands::get_memories,
            commands::clear_memories,
            commands::take_screenshot,
            commands::get_current_url,
            commands::get_config,
            commands::save_config,
            commands::reset_session,
            commands::get_traces,
            commands::clear_traces,
            commands::get_trace_count
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
