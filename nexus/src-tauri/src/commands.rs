use tauri::State;
use crate::browser::BrowserManager;
use crate::search::search_content;
use html_to_markdown_rs::convert;

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
) -> Result<String, String> {
    crate::agent::run_agent_loop(prompt).await
}
