pub mod browser;
pub mod commands;
pub mod search;

use browser::BrowserManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let browser = match tauri::async_runtime::block_on(async {
                BrowserManager::new().await
            }) {
                Ok(b) => b,
                Err(e) => return Err(e.into()),
            };

            app.manage(browser);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::fetch_and_search])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
