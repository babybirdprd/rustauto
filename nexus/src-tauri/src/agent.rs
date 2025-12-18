use radkit::agent::LlmWorker;
use radkit::models::providers::{AnthropicLlm, OpenAILlm, OpenRouterLlm};
use radkit::models::BaseLlm;
use radkit::macros::tool;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use crate::browser::GLOBAL_BROWSER;
use crate::memory::GLOBAL_MEMORY;
use crate::GLOBAL_APP;
use html_to_markdown_rs::convert;
use crate::search::search_content;
use tauri::Emitter;

#[derive(Deserialize, JsonSchema)]
struct NavigateArgs {
    url: String,
}

#[derive(Deserialize, JsonSchema)]
struct SearchArgs {
    query: String,
    url: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct ClickArgs {
    selector: String,
}

#[derive(Deserialize, JsonSchema)]
struct TypeArgs {
    text: String,
}

#[derive(Deserialize, JsonSchema)]
struct ScrollArgs {
    direction: String, // "up" or "down"
    amount: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
struct MemorizeArgs {
    note: String,
}

#[derive(Deserialize, JsonSchema)]
struct RecallArgs {}

fn emit_event(event_type: &str, message: String) {
    if let Some(app) = GLOBAL_APP.get() {
        let _ = app.emit("agent-event", json!({
            "type": event_type,
            "message": message,
            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
        }));
    }
}

fn process_content(html: String) -> String {
    let md = convert(&html, None).unwrap_or_else(|e| format!("Conversion failed: {}", e));
    let truncated: String = md.chars().take(10000).collect();
    if truncated.len() < md.len() {
        format!("{}... (truncated)", truncated)
    } else {
        md
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
            let content = process_content(html);
            emit_event("tool_result", format!("Navigated to {}. Content length: {}", args.url, content.len()));
            ToolResult::success(json!({
                "url": args.url,
                "content": content
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
        // If no URL is provided, we should ideally use the current page content.
        // However, `search_content` works on string content.
        // We'd need to fetch current page content from browser if it exposes it.
        // For now, let's assume the agent uses `navigate` first or provides a URL.
        // But wait, `navigate` returns content.
        // If the agent wants to search *again* on the *same* page without reloading?
        // The BrowserManager maintains state, but `navigate_and_get_content` navigates.
        // Let's assume for this version URL is preferred or we error.
        return ToolResult::error("URL is required for search in this version. Navigate first if needed.");
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

#[tool(description = "Click an element by CSS selector and return updated content")]
async fn click(args: ClickArgs) -> ToolResult {
    emit_event("tool_call", format!("Clicking '{}'", args.selector));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => return ToolResult::error("Browser not initialized"),
    };

    match browser.click_element(&args.selector).await {
        Ok(html) => {
            let content = process_content(html);
            emit_event("tool_result", format!("Clicked '{}'. Content length: {}", args.selector, content.len()));
            ToolResult::success(json!({
                "content": content
            }))
        },
        Err(e) => {
            emit_event("error", format!("Failed to click: {}", e));
            ToolResult::error(e.to_string())
        }
    }
}

#[tool(description = "Type text into the focused element")]
async fn type_input(args: TypeArgs) -> ToolResult {
    emit_event("tool_call", format!("Typing '{}'", args.text));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => return ToolResult::error("Browser not initialized"),
    };

    match browser.type_text(&args.text).await {
        Ok(html) => {
            let content = process_content(html);
            emit_event("tool_result", format!("Typed text. Content length: {}", content.len()));
            ToolResult::success(json!({
                "content": content
            }))
        },
        Err(e) => {
            emit_event("error", format!("Failed to type: {}", e));
            ToolResult::error(e.to_string())
        }
    }
}

#[tool(description = "Scroll the page up or down")]
async fn scroll(args: ScrollArgs) -> ToolResult {
    emit_event("tool_call", format!("Scrolling {}", args.direction));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => return ToolResult::error("Browser not initialized"),
    };

    match browser.scroll_page(&args.direction, args.amount).await {
        Ok(html) => {
             let content = process_content(html);
             emit_event("tool_result", format!("Scrolled {}. Content length: {}", args.direction, content.len()));
             ToolResult::success(json!({
                "content": content
            }))
        },
        Err(e) => {
            emit_event("error", format!("Failed to scroll: {}", e));
            ToolResult::error(e.to_string())
        }
    }
}

#[tool(description = "Store a note or finding in the agent's memory for later recall.")]
async fn memorize(args: MemorizeArgs) -> ToolResult {
    emit_event("tool_call", format!("Memorizing note: {}", args.note));

    if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        if let Ok(mut mem) = mem_lock.lock() {
            mem.add(args.note.clone());
            emit_event("tool_result", "Note memorized.".to_string());
            return ToolResult::success(json!({ "status": "memorized", "note": args.note }));
        }
    }
    ToolResult::error("Failed to access memory".to_string())
}

#[tool(description = "Recall all memorized notes.")]
async fn recall(_args: RecallArgs) -> ToolResult {
    emit_event("tool_call", "Recalling memories".to_string());
     if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        if let Ok(mem) = mem_lock.lock() {
            let notes = mem.get_all();
            emit_event("tool_result", format!("Recalled {} notes", notes.len()));
            return ToolResult::success(json!({ "notes": notes }));
        }
    }
    ToolResult::error("Failed to access memory".to_string())
}

async fn run_worker<L: BaseLlm + 'static>(llm: L, prompt: String) -> Result<String, String> {
    let worker = LlmWorker::<String>::builder(llm)
        .with_system_instructions("You are Nexus, an intelligent browser agent. You can navigate the web, click elements, type text, scroll, search, and manage memory. Please explain your reasoning before taking actions using a <thinking> tag.")
        .with_tool(navigate)
        .with_tool(search)
        .with_tool(click)
        .with_tool(type_input)
        .with_tool(scroll)
        .with_tool(memorize)
        .with_tool(recall)
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

pub async fn run_agent_loop(prompt: String) -> Result<String, String> {
    emit_event("system", format!("Agent started with prompt: {}", prompt));

    let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "anthropic".to_string());
    // Default model depends on provider, but we'll let the user specify or fallback to a reasonable default if possible.
    // However, `from_env` usually takes a model name.
    let model = std::env::var("LLM_MODEL").ok();

    match provider.to_lowercase().as_str() {
        "anthropic" => {
            let model_name = model.unwrap_or_else(|| "claude-3-sonnet-20240229".to_string());
            emit_event("system", format!("Using Anthropic provider with model: {}", model_name));
            let llm = AnthropicLlm::from_env(model_name).map_err(|e| e.to_string())?;
            run_worker(llm, prompt).await
        },
        "openai" => {
             let model_name = model.unwrap_or_else(|| "gpt-4o".to_string());
             emit_event("system", format!("Using OpenAI provider with model: {}", model_name));
             let llm = OpenAILlm::from_env(model_name).map_err(|e| e.to_string())?;
             run_worker(llm, prompt).await
        },
        "openrouter" => {
            let model_name = model.unwrap_or_else(|| "anthropic/claude-3.5-sonnet".to_string());
            emit_event("system", format!("Using OpenRouter provider with model: {}", model_name));
            let llm = OpenRouterLlm::from_env(model_name)
                .map_err(|e| e.to_string())?
                .with_site_url("https://nexus.local") // TODO: Configurable
                .with_app_name("Nexus Agent");
            run_worker(llm, prompt).await
        },
        _ => {
            let msg = format!("Unsupported LLM_PROVIDER: {}", provider);
            emit_event("error", msg.clone());
            Err(msg)
        }
    }
        }
