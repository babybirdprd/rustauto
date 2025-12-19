use radkit::agent::LlmWorker;
use radkit::models::providers::{AnthropicLlm, OpenAILlm, OpenRouterLlm, GeminiLlm, GrokLlm, DeepSeekLlm};
use radkit::models::BaseLlm;
use radkit::macros::tool;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use crate::browser::GLOBAL_BROWSER;
use crate::memory::GLOBAL_MEMORY;
use crate::GLOBAL_APP;
use crate::config::Config;
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
struct UploadArgs {
    selector: String,
    file_path: String,
}

#[derive(Deserialize, JsonSchema)]
struct MemorizeArgs {
    note: String,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
struct RecallArgs {
    query: Option<String>,
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

#[tool(description = "Upload a file to an input element. selector must point to an input[type='file'].")]
async fn upload(args: UploadArgs) -> ToolResult {
    emit_event("tool_call", format!("Uploading {} to {}", args.file_path, args.selector));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => return ToolResult::error("Browser not initialized"),
    };

    match browser.upload_file(&args.selector, &args.file_path).await {
        Ok(html) => {
            let content = process_content(html);
            emit_event("tool_result", format!("Uploaded file. Content length: {}", content.len()));
            ToolResult::success(json!({
                "content": content
            }))
        },
        Err(e) => {
            emit_event("error", format!("Failed to upload: {}", e));
            ToolResult::error(e.to_string())
        }
    }
}

#[tool(description = "Store a note or finding in the agent's memory for later recall.")]
async fn memorize(args: MemorizeArgs) -> ToolResult {
    emit_event("tool_call", format!("Memorizing note: {}", args.note));

    if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        if let Ok(mut mem) = mem_lock.lock() {
            let tags = args.tags.unwrap_or_default();
            mem.add(args.note.clone(), tags.clone());
            emit_event("tool_result", "Note memorized.".to_string());
            return ToolResult::success(json!({ "status": "memorized", "note": args.note, "tags": tags }));
        }
    }
    ToolResult::error("Failed to access memory".to_string())
}

#[tool(description = "Recall memorized notes. Optionally filter by a query string.")]
async fn recall(args: RecallArgs) -> ToolResult {
    emit_event("tool_call", format!("Recalling memories. Query: {:?}", args.query));
     if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        if let Ok(mem) = mem_lock.lock() {
             let notes = if let Some(q) = args.query {
                mem.search(&q)
            } else {
                mem.get_all()
            };
            emit_event("tool_result", format!("Recalled {} notes", notes.len()));
            return ToolResult::success(json!({ "notes": notes }));
        }
    }
    ToolResult::error("Failed to access memory".to_string())
}

async fn run_worker<L: BaseLlm + 'static>(llm: L, prompt: String) -> Result<String, String> {
    let worker = LlmWorker::<String>::builder(llm)
        .with_system_instructions("You are Nexus, an intelligent browser agent. You can navigate the web, click elements, type text, scroll, search, and manage memory.

GOAL-DIRECTED BEHAVIOR:
If a user prompt does not include a specific URL, you must take the initiative to find the relevant information. For example:
- Use a search engine (e.g., Google) to find news, products, or answers.
- Navigate to known websites (e.g., Hacker News, Amazon, Walmart, Target) to fulfill specific requests.
- Compare information across multiple sites by navigating between them and using memory.
- Take your best guess at the user's intent and start by searching or navigating to a likely source.

CRITICAL INSTRUCTIONS:
1. Explain your reasoning before taking actions using a <thinking> tag.
2. If a tool fails, analyze the error and try a different approach (e.g., different selector, search query, or URL). Do not give up immediately.
3. Use the `memorize` tool to save important information found during browsing.
4. Use the `recall` tool to retrieve information if needed.
")
        .with_tool(navigate)
        .with_tool(search)
        .with_tool(click)
        .with_tool(type_input)
        .with_tool(scroll)
        .with_tool(upload)
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

pub async fn run_agent_loop(prompt: String, config: Config) -> Result<String, String> {
    emit_event("system", format!("Agent started with prompt: {}", prompt));

    let provider = config.provider.to_lowercase();
    let model_name = config.model.clone();
    let api_key = config.api_key.clone();

    // Set environment variable for radkit to pick up if needed, though most providers have from_env
    // But radkit's from_env usually reads specific env vars.
    // If we want to use the key from config, we should set the env var temporarily or check if radkit supports direct key passing.
    // Looking at radkit_docs, it seems to prefer env vars.
    match provider.as_str() {
        "anthropic" => std::env::set_var("ANTHROPIC_API_KEY", &api_key),
        "openai" => std::env::set_var("OPENAI_API_KEY", &api_key),
        "openrouter" => std::env::set_var("OPENROUTER_API_KEY", &api_key),
        "gemini" => std::env::set_var("GEMINI_API_KEY", &api_key),
        "grok" => std::env::set_var("XAI_API_KEY", &api_key),
        "deepseek" => std::env::set_var("DEEPSEEK_API_KEY", &api_key),
        _ => {}
    }

    match provider.as_str() {
        "anthropic" => {
            emit_event("system", format!("Using Anthropic provider with model: {}", model_name));
            let llm = AnthropicLlm::from_env(model_name).map_err(|e| e.to_string())?;
            run_worker(llm, prompt).await
        },
        "openai" => {
             emit_event("system", format!("Using OpenAI provider with model: {}", model_name));
             let mut llm = OpenAILlm::from_env(model_name).map_err(|e| e.to_string())?;
             if let Some(base_url) = config.base_url {
                 if !base_url.is_empty() {
                     llm = llm.with_base_url(base_url);
                 }
             }
             run_worker(llm, prompt).await
        },
        "openrouter" => {
            emit_event("system", format!("Using OpenRouter provider with model: {}", model_name));
            let llm = OpenRouterLlm::from_env(model_name)
                .map_err(|e| e.to_string())?
                .with_site_url("https://nexus.local")
                .with_app_name("Nexus Agent");
            run_worker(llm, prompt).await
        },
        "gemini" => {
            emit_event("system", format!("Using Gemini provider with model: {}", model_name));
            let llm = GeminiLlm::from_env(model_name).map_err(|e| e.to_string())?;
            run_worker(llm, prompt).await
        },
        "grok" => {
            emit_event("system", format!("Using Grok provider with model: {}", model_name));
            let llm = GrokLlm::from_env(model_name).map_err(|e| e.to_string())?;
            run_worker(llm, prompt).await
        },
        "deepseek" => {
            emit_event("system", format!("Using DeepSeek provider with model: {}", model_name));
            let llm = DeepSeekLlm::from_env(model_name).map_err(|e| e.to_string())?;
            run_worker(llm, prompt).await
        },
        _ => {
            let msg = format!("Unsupported LLM_PROVIDER: {}", provider);
            emit_event("error", msg.clone());
            Err(msg)
        }
    }
}
