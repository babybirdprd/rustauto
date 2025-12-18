use radkit::agent::LlmWorker;
use radkit::models::providers::AnthropicLlm;
use radkit::macros::tool;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::Deserialize; // Removed Serialize
use serde_json::json;
use crate::browser::GLOBAL_BROWSER;
use crate::GLOBAL_APP;
use html_to_markdown_rs::convert;
use crate::search::search_content;
use tauri::Emitter;
// Removed async_trait::async_trait

// Use serde_json::Value for output to avoid LlmDeserialize derive issues
// #[derive(Debug, Serialize, Deserialize, JsonSchema)]
// pub struct AgentOutput {
//    pub answer: String,
// }

#[derive(Deserialize, JsonSchema)]
struct NavigateArgs {
    url: String,
}

#[derive(Deserialize, JsonSchema)]
struct SearchArgs {
    query: String,
    url: Option<String>,
}

fn emit_event(event_type: &str, message: String) {
    if let Some(app) = GLOBAL_APP.get() {
        let _ = app.emit("agent-event", json!({
            "type": event_type,
            "message": message,
            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
        }));
    }
}

#[tool(description = "Navigate to a URL and return its content as Markdown")]
async fn navigate(args: NavigateArgs) -> ToolResult {
    emit_event("tool_call", format!("Navigating to {}", args.url));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => return ToolResult::error("Browser not initialized"),
    };

    match browser.navigate_and_get_content(&args.url).await {
        Ok(html) => {
            let md = convert(&html, None).unwrap_or_else(|e| format!("Conversion failed: {}", e));
            let snippet = if md.len() > 10000 {
                format!("{}... (truncated)", &md[..10000])
            } else {
                md
            };
            emit_event("tool_result", format!("Navigated to {}. Content length: {}", args.url, snippet.len()));
            ToolResult::success(json!({
                "url": args.url,
                "content": snippet
            }))
        },
        Err(e) => {
            emit_event("error", format!("Failed to navigate: {}", e));
            ToolResult::error(e.to_string())
        },
    }
}

#[tool(description = "Search for text in the current page or a specific URL")]
async fn search(args: SearchArgs) -> ToolResult {
    emit_event("tool_call", format!("Searching for '{}'", args.query));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => return ToolResult::error("Browser not initialized"),
    };

    let content = if let Some(url) = args.url {
         match browser.navigate_and_get_content(&url).await {
            Ok(html) => convert(&html, None).unwrap_or_else(|e| format!("Conversion failed: {}", e)),
            Err(e) => return ToolResult::error(e.to_string()),
        }
    } else {
        return ToolResult::error("URL is required for search in this version");
    };

    match search_content(&content, &args.query) {
        Ok(matches) => {
             emit_event("tool_result", format!("Found {} matches", matches.len()));
             ToolResult::success(json!({
                "matches": matches
            }))
        },
        Err(e) => {
            emit_event("error", format!("Search failed: {}", e));
            ToolResult::error(e.to_string())
        },
    }
}

pub async fn run_agent_loop(prompt: String) -> Result<String, String> {
    emit_event("system", format!("Agent started with prompt: {}", prompt));

    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
         emit_event("error", "ANTHROPIC_API_KEY not found in environment".to_string());
         return Err("ANTHROPIC_API_KEY not found in environment".to_string());
    }

    let llm = match AnthropicLlm::from_env("claude-3-sonnet-20240229") {
        Ok(l) => l,
        Err(e) => return Err(e.to_string()),
    };

    let worker = LlmWorker::<String>::builder(llm)
        .with_system_instructions("You are Nexus, an intelligent browser agent. You can navigate the web and search for information.")
        .with_tool(navigate)
        .with_tool(search)
        .build();

    match worker.run(&prompt).await {
        Ok(result) => {
            emit_event("success", format!("Agent finished: {}", result));
            Ok(result)
        }
        Err(e) => {
             emit_event("error", format!("Agent execution failed: {}", e));
             Err(e.to_string())
        }
    }
}
