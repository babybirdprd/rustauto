use crate::browser::BrowserManager;
use crate::config::{Config, ConfigManager};
use crate::memory::{MemoryEntry, GLOBAL_MEMORY};
use crate::search::search_content;
use crate::tracing::{TraceEvent, TRACE_STORE};
use html_to_markdown_rs::convert;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub async fn fetch_and_search(
    url: String,
    query: String,
    state: State<'_, BrowserManager>,
) -> Result<Vec<String>, String> {
    crate::trace_info!(
        "nexus::commands",
        "fetch_and_search called",
        url = url,
        query = query
    );

    let content_html = state
        .navigate_and_get_content(&url)
        .await
        .map_err(|e| e.to_string())?;

    // Convert HTML to Markdown
    let content_md = convert(&content_html, None).map_err(|e| e.to_string())?;

    let matches = search_content(&content_md, &query).map_err(|e| e.to_string())?;

    crate::trace_info!(
        "nexus::commands",
        "fetch_and_search complete",
        matches = matches.len()
    );
    Ok(matches)
}

#[tauri::command]
pub async fn run_agent(
    prompt: String,
    _app_handle: tauri::AppHandle,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<String, String> {
    crate::trace_info!("nexus::commands", "run_agent called", prompt = prompt);

    let config = config_manager.lock().unwrap().load();
    crate::trace_debug!(
        "nexus::commands",
        "Config loaded",
        provider = config.provider,
        model = config.model
    );

    let result = crate::agent::run_agent_loop(prompt, config).await;

    match &result {
        Ok(_) => crate::trace_info!("nexus::commands", "run_agent completed successfully"),
        Err(e) => crate::trace_error!("nexus::commands", "run_agent failed", error = e),
    }

    result
}

#[tauri::command]
pub fn get_memories() -> Result<Vec<MemoryEntry>, String> {
    crate::trace_debug!("nexus::commands", "get_memories called");

    if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        if let Ok(mem) = mem_lock.lock() {
            let memories = mem.get_all();
            crate::trace_debug!(
                "nexus::commands",
                "get_memories returning",
                count = memories.len()
            );
            return Ok(memories);
        }
    }
    crate::trace_error!("nexus::commands", "Failed to access memory");
    Err("Failed to access memory".to_string())
}

#[tauri::command]
pub fn clear_memories() -> Result<(), String> {
    crate::trace_info!("nexus::commands", "clear_memories called");

    if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        if let Ok(mut mem) = mem_lock.lock() {
            mem.clear();
            crate::trace_info!("nexus::commands", "Memories cleared");
            return Ok(());
        }
    }
    crate::trace_error!("nexus::commands", "Failed to clear memory");
    Err("Failed to access memory".to_string())
}

#[tauri::command]
pub async fn take_screenshot(state: State<'_, BrowserManager>) -> Result<String, String> {
    crate::trace_debug!("nexus::commands", "take_screenshot called");
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
    crate::trace_info!(
        "nexus::commands",
        "save_config called",
        provider = config.provider,
        model = config.model
    );
    config_manager.lock().unwrap().save(&config)
}

#[tauri::command]
pub async fn reset_session(
    browser: State<'_, crate::browser::BrowserManager>,
) -> Result<(), String> {
    crate::trace_info!("nexus::commands", "reset_session called");

    if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        if let Ok(mut mem) = mem_lock.lock() {
            mem.clear();
        }
    }

    browser.reset().await.map_err(|e| e.to_string())?;

    // Also reset traces
    if let Some(store) = TRACE_STORE.get() {
        if let Ok(mut guard) = store.try_lock() {
            guard.reset();
        }
    }

    crate::trace_info!("nexus::commands", "Session reset complete");
    Ok(())
}

// ============================================================================
// Trace API Commands
// ============================================================================

#[tauri::command]
pub fn get_traces(level: Option<String>) -> Result<Vec<TraceEvent>, String> {
    if let Some(store) = TRACE_STORE.get() {
        if let Ok(guard) = store.try_lock() {
            let events = if let Some(lvl) = level {
                guard.get_events_by_level(&lvl)
            } else {
                guard.get_events()
            };
            return Ok(events);
        }
    }
    Err("Failed to access trace store".to_string())
}

#[tauri::command]
pub fn clear_traces() -> Result<(), String> {
    if let Some(store) = TRACE_STORE.get() {
        if let Ok(mut guard) = store.try_lock() {
            guard.reset();
            return Ok(());
        }
    }
    Err("Failed to access trace store".to_string())
}

#[tauri::command]
pub fn get_trace_count() -> Result<usize, String> {
    if let Some(store) = TRACE_STORE.get() {
        if let Ok(guard) = store.try_lock() {
            return Ok(guard.count());
        }
    }
    Err("Failed to access trace store".to_string())
}
