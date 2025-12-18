use tauri::State;
use crate::browser::BrowserManager;
use crate::search::search_content;

#[tauri::command]
pub async fn fetch_and_search(
    url: String,
    query: String,
    state: State<'_, BrowserManager>,
) -> Result<Vec<String>, String> {
    let content = state
        .navigate_and_get_content(&url)
        .await
        .map_err(|e| e.to_string())?;

    let matches = search_content(&content, &query)
        .map_err(|e| e.to_string())?;

    Ok(matches)
}
