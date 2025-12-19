pub mod agent;
pub mod browser;
pub mod commands;
pub mod config;
pub mod memory;
pub mod search;

use browser::BrowserManager;
use config::ConfigManager;
use tauri::Manager;
use std::sync::{OnceLock, Mutex};

pub static GLOBAL_APP: OnceLock<tauri::AppHandle> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let _ = GLOBAL_APP.set(app.handle().clone());
            memory::init_memory();

            let config_manager = ConfigManager::new(app.handle());
            app.manage(Mutex::new(config_manager));

            let browser = match tauri::async_runtime::block_on(async {
                BrowserManager::new().await
            }) {
                Ok(b) => b,
                Err(e) => return Err(e.into()),
            };

            // Set global instance for agent tools
            let _ = browser::GLOBAL_BROWSER.set(browser.clone());

            app.manage(browser);
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
            commands::save_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
