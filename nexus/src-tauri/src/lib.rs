pub mod agent;
pub mod browser;
pub mod commands;
pub mod memory;
pub mod search;

use browser::BrowserManager;
use tauri::Manager;
use std::sync::OnceLock;

pub static GLOBAL_APP: OnceLock<tauri::AppHandle> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let _ = GLOBAL_APP.set(app.handle().clone());
            memory::init_memory();

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
            commands::run_agent
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
