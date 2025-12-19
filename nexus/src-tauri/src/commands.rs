use tauri::State;
use crate::browser::BrowserManager;
use crate::search::search_content;
use crate::memory::{GLOBAL_MEMORY, MemoryEntry};
use crate::config::{Config, ConfigManager};
use html_to_markdown_rs::convert;
use std::sync::Mutex;

#[tauri::command]
pub async fn fetch_and_search(
    url: String,
    query: String,
    state: State<'_, BrowserManager>,
) -> Result<Vec<String>, String> {
    let content_html = state
        .navigate_and_get_content(&url)
        .await
        .map_err(|e| e.to_string())?;

    // Convert HTML to Markdown
    let content_md = convert(&content_html, None).map_err(|e| e.to_string())?;

    let matches = search_content(&content_md, &query)
        .map_err(|e| e.to_string())?;

    Ok(matches)
}

#[tauri::command]
pub async fn run_agent(
    prompt: String,
    _app_handle: tauri::AppHandle,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<String, String> {
    let config = config_manager.lock().unwrap().load();
    crate::agent::run_agent_loop(prompt, config).await
}

#[tauri::command]
pub fn get_memories() -> Result<Vec<MemoryEntry>, String> {
    if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        if let Ok(mem) = mem_lock.lock() {
            return Ok(mem.get_all());
        }
    }
    Err("Failed to access memory".to_string())
}

#[tauri::command]
pub fn clear_memories() -> Result<(), String> {
    if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        if let Ok(mut mem) = mem_lock.lock() {
            mem.clear();
            return Ok(());
        }
    }
    Err("Failed to access memory".to_string())
}

#[tauri::command]
pub async fn take_screenshot(state: State<'_, BrowserManager>) -> Result<String, String> {
    state.take_screenshot().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_current_url(state: State<'_, BrowserManager>) -> Result<String, String> {
    state.get_current_url().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config(config_manager: State<'_, Mutex<ConfigManager>>) -> Result<Config, String> {
    Ok(config_manager.lock().unwrap().load())
}

#[tauri::command]
pub fn save_config(
    config: Config,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), String> {
    config_manager.lock().unwrap().save(&config)
}

#[tauri::command]
pub async fn reset_session(
    browser: State<'_, crate::browser::BrowserManager>,
) -> Result<(), String> {
    if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        if let Ok(mut mem) = mem_lock.lock() {
            mem.clear();
        }
    }

    browser.reset().await.map_err(|e| e.to_string())?;
    
    Ok(())
}
