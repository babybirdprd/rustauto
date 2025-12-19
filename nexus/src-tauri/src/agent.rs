use crate::browser::GLOBAL_BROWSER;
use crate::config::Config;
use crate::memory::GLOBAL_MEMORY;
use crate::search::search_content;
use crate::GLOBAL_APP;
use html_to_markdown_rs::convert;
use radkit::agent::LlmWorker;
use radkit::macros::{tool, LLMOutput};
use radkit::models::providers::{
    AnthropicLlm, DeepSeekLlm, GeminiLlm, GrokLlm, OpenAILlm, OpenRouterLlm,
};
use radkit::models::BaseLlm;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::Emitter;

// --- Structured Output Types ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, LLMOutput)]
pub struct NexusReport {
    /// A detailed, synthesized report in Markdown format.
    pub markdown_report: String,
    /// Key bullet points or discoveries found during the session.
    pub key_discoveries: Vec<String>,
    /// List of URLs or sources used to compile the report.
    pub sources: Vec<String>,
}

// --- Tool Arguments ---

#[derive(Deserialize, JsonSchema)]
struct NavigateArgs {
    /// The URL to navigate to.
    url: String,
}

#[derive(Deserialize, JsonSchema)]
struct FindInPageArgs {
    /// The text to find in the current page.
    query: String,
}

#[derive(Deserialize, JsonSchema)]
struct ClickArgs {
    /// CSS selector of the element to click.
    selector: String,
}

#[derive(Deserialize, JsonSchema)]
struct TypeArgs {
    /// The text to type into the focused element.
    text: String,
}

#[derive(Deserialize, JsonSchema)]
struct ScrollArgs {
    /// Direction: "up" or "down".
    direction: String,
    /// Amount in pixels (default 500).
    amount: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
struct UploadArgs {
    /// CSS selector for the file input.
    selector: String,
    /// Absolute path to the file.
    file_path: String,
}

#[derive(Deserialize, JsonSchema)]
struct MemorizeArgs {
    /// Fact or note to remember.
    note: String,
    /// Optional tags for categorization.
    tags: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
struct RecallArgs {
    /// Optional query to filter memories.
    query: Option<String>,
}

// --- Helper Functions ---

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
    // Increased limit to 15k for better context on long pages (e.g. HN)
    let limit = 15000;
    let truncated: String = md.chars().take(limit).collect();
    if truncated.len() < md.len() {
        format!("{}... (truncated, total length: {})", truncated, md.len())
    } else {
        md
    }
}

// --- Tools ---

#[tool(
    description = "Navigate to a URL and return its body content as Markdown. Use this to visit specific sites."
)]
async fn navigate(args: NavigateArgs) -> ToolResult {
    crate::trace_info!("nexus::agent::navigate", "Tool called", url = args.url);
    emit_event("tool_call", format!("Navigating to {}", args.url));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => {
            crate::trace_debug!("nexus::agent::navigate", "Got browser reference");
            b
        }
        None => {
            crate::trace_error!("nexus::agent::navigate", "Browser not initialized");
            return ToolResult::error("Browser not initialized");
        }
    };

    crate::trace_debug!("nexus::agent::navigate", "Calling navigate_and_get_content");
    match browser.navigate_and_get_content(&args.url).await {
        Ok(html) => {
            crate::trace_debug!(
                "nexus::agent::navigate",
                "Got HTML response",
                html_len = html.len()
            );
            let content = process_content(html);
            crate::trace_info!(
                "nexus::agent::navigate",
                "Navigation complete",
                content_len = content.len()
            );
            emit_event(
                "tool_result",
                format!(
                    "Navigated to {}. Content length: {}",
                    args.url,
                    content.len()
                ),
            );
            ToolResult::success(json!({
                "url": args.url,
                "content": content
            }))
        }
        Err(e) => {
            crate::trace_error!(
                "nexus::agent::navigate",
                "Navigation failed",
                error = e.to_string()
            );
            emit_event("error", format!("Failed to navigate: {}", e));
            ToolResult::error(e.to_string())
        }
    }
}

#[tool(
    description = "Search for a specific string within the ALREADY LOADED content of the current page."
)]
async fn find_in_page(args: FindInPageArgs, _ctx: &radkit::tools::ToolContext<'_>) -> ToolResult {
    emit_event("tool_call", format!("Finding '{}' in page", args.query));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => return ToolResult::error("Browser not initialized"),
    };

    match browser.get_content().await {
        Ok(html) => {
            let content = convert(&html, None).unwrap_or_default();
            match search_content(&content, &args.query) {
                Ok(matches) => {
                    emit_event("tool_result", format!("Found {} matches", matches.len()));
                    ToolResult::success(json!({ "matches": matches }))
                }
                Err(e) => {
                    emit_event("error", format!("Find failed: {}", e));
                    ToolResult::error(e.to_string())
                }
            }
        }
        Err(e) => {
            emit_event("error", format!("Failed to get content: {}", e));
            ToolResult::error(e.to_string())
        }
    }
}

#[tool(description = "Click an element by CSS selector and return updated content.")]
async fn click(args: ClickArgs) -> ToolResult {
    crate::trace_info!(
        "nexus::agent::click",
        "Tool called",
        selector = args.selector
    );
    emit_event("tool_call", format!("Clicking '{}'", args.selector));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => {
            crate::trace_error!("nexus::agent::click", "Browser not initialized");
            return ToolResult::error("Browser not initialized");
        }
    };

    crate::trace_debug!("nexus::agent::click", "Calling click_element");
    match browser.click_element(&args.selector).await {
        Ok(html) => {
            crate::trace_debug!(
                "nexus::agent::click",
                "Click succeeded",
                html_len = html.len()
            );
            let content = process_content(html);
            crate::trace_info!(
                "nexus::agent::click",
                "Click complete",
                content_len = content.len()
            );
            emit_event(
                "tool_result",
                format!(
                    "Clicked '{}'. Content length: {}",
                    args.selector,
                    content.len()
                ),
            );
            ToolResult::success(json!({
                "content": content
            }))
        }
        Err(e) => {
            crate::trace_error!("nexus::agent::click", "Click failed", error = e.to_string());
            emit_event("error", format!("Failed to click: {}", e));
            ToolResult::error(e.to_string())
        }
    }
}

#[tool(description = "Type text into the focused element.")]
async fn type_input(args: TypeArgs) -> ToolResult {
    emit_event("tool_call", format!("Typing '{}'", args.text));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => return ToolResult::error("Browser not initialized"),
    };

    match browser.type_text(&args.text).await {
        Ok(html) => {
            let content = process_content(html);
            emit_event(
                "tool_result",
                format!("Typed text. Content length: {}", content.len()),
            );
            ToolResult::success(json!({
                "content": content
            }))
        }
        Err(e) => {
            emit_event("error", format!("Failed to type: {}", e));
            ToolResult::error(e.to_string())
        }
    }
}

#[tool(description = "Scroll the page up or down.")]
async fn scroll(args: ScrollArgs) -> ToolResult {
    emit_event("tool_call", format!("Scrolling {}", args.direction));

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => return ToolResult::error("Browser not initialized"),
    };

    match browser.scroll_page(&args.direction, args.amount).await {
        Ok(html) => {
            let content = process_content(html);
            emit_event(
                "tool_result",
                format!(
                    "Scrolled {}. Content length: {}",
                    args.direction,
                    content.len()
                ),
            );
            ToolResult::success(json!({
                "content": content
            }))
        }
        Err(e) => {
            emit_event("error", format!("Failed to scroll: {}", e));
            ToolResult::error(e.to_string())
        }
    }
}

#[tool(description = "Upload a file to a specific file input selector.")]
async fn upload(args: UploadArgs) -> ToolResult {
    emit_event(
        "tool_call",
        format!("Uploading {} to {}", args.file_path, args.selector),
    );

    let browser = match GLOBAL_BROWSER.get() {
        Some(b) => b,
        None => return ToolResult::error("Browser not initialized"),
    };

    match browser.upload_file(&args.selector, &args.file_path).await {
        Ok(html) => {
            let content = process_content(html);
            emit_event(
                "tool_result",
                format!("Uploaded file. Content length: {}", content.len()),
            );
            ToolResult::success(json!({
                "content": content
            }))
        }
        Err(e) => {
            emit_event("error", format!("Failed to upload: {}", e));
            ToolResult::error(e.to_string())
        }
    }
}

#[tool(description = "Store context or findings in your long-term memory.")]
async fn memorize(args: MemorizeArgs) -> ToolResult {
    crate::trace_info!("nexus::agent::memorize", "Tool called", note = args.note);
    emit_event("tool_call", format!("Memorizing note: {}", args.note));

    if let Some(mem_lock) = GLOBAL_MEMORY.get() {
        crate::trace_debug!("nexus::agent::memorize", "Got memory lock reference");
        if let Ok(mut mem) = mem_lock.lock() {
            let tags = args.tags.unwrap_or_default();
            crate::trace_debug!(
                "nexus::agent::memorize",
                "Adding to memory",
                tags_count = tags.len()
            );
            mem.add(args.note.clone(), tags.clone());
            crate::trace_info!("nexus::agent::memorize", "Note memorized successfully");
            emit_event("tool_result", "Note memorized.".to_string());
            return ToolResult::success(
                json!({ "status": "memorized", "note": args.note, "tags": tags }),
            );
        } else {
            crate::trace_error!("nexus::agent::memorize", "Failed to acquire memory lock");
        }
    } else {
        crate::trace_error!("nexus::agent::memorize", "GLOBAL_MEMORY not initialized");
    }
    ToolResult::error("Failed to access memory".to_string())
}

#[tool(description = "Recall information from your long-term memory.")]
async fn recall(args: RecallArgs) -> ToolResult {
    emit_event(
        "tool_call",
        format!("Recalling memories. Query: {:?}", args.query),
    );
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

async fn execute_nexus_worker<L: BaseLlm + 'static>(
    llm: L,
    prompt: String,
) -> Result<String, String> {
    crate::trace_info!("nexus::agent::worker", "Building LlmWorker");

    // We use the worker directly as we don't need the full A2A runtime server for this loop
    let worker = LlmWorker::<NexusReport>::builder(llm)
        .with_system_instructions("You are Nexus, a premium, autonomous browser agent. Your mission is to provide high-quality, structured reports.")
        .with_tool(navigate)
        .with_tool(find_in_page)
        .with_tool(click)
        .with_tool(type_input)
        .with_tool(scroll)
        .with_tool(upload)
        .with_tool(memorize)
        .with_tool(recall)
        .build();

    crate::trace_info!(
        "nexus::agent::worker",
        "Worker built, starting execution",
        prompt_len = prompt.len()
    );

    match worker.run(prompt).await {
        Ok(report) => {
            crate::trace_info!(
                "nexus::agent::worker",
                "Worker completed successfully",
                report_len = report.markdown_report.len(),
                discoveries = report.key_discoveries.len(),
                sources = report.sources.len()
            );
            emit_event(
                "success",
                format!("Agent finished: {}", report.markdown_report),
            );
            Ok(report.markdown_report)
        }
        Err(e) => {
            crate::trace_error!(
                "nexus::agent::worker",
                "Worker execution failed",
                error = e.to_string()
            );
            emit_event("error", format!("Agent execution failed: {}", e));
            Err(e.to_string())
        }
    }
}

pub async fn run_agent_loop(prompt: String, config: Config) -> Result<String, String> {
    crate::trace_info!(
        "nexus::agent::loop",
        "Agent loop starting",
        prompt_len = prompt.len()
    );
    emit_event("system", format!("Agent started with prompt: {}", prompt));

    let provider = config.provider.to_lowercase();
    let model_name = config.model.clone();
    let api_key = config.api_key.clone();

    crate::trace_info!(
        "nexus::agent::loop",
        "Configuring LLM",
        provider = provider,
        model = model_name
    );

    // Set environment variable for radkit
    match provider.as_str() {
        "anthropic" => std::env::set_var("ANTHROPIC_API_KEY", &api_key),
        "openai" => std::env::set_var("OPENAI_API_KEY", &api_key),
        "openrouter" => std::env::set_var("OPENROUTER_API_KEY", &api_key),
        "gemini" => std::env::set_var("GEMINI_API_KEY", &api_key),
        "grok" => std::env::set_var("XAI_API_KEY", &api_key),
        "deepseek" => std::env::set_var("DEEPSEEK_API_KEY", &api_key),
        _ => {}
    }
    crate::trace_debug!("nexus::agent::loop", "API key set in environment");

    crate::trace_info!(
        "nexus::agent::loop",
        "Creating LLM instance",
        provider = provider
    );
    match provider.as_str() {
        "anthropic" => {
            let llm = AnthropicLlm::from_env(model_name).map_err(|e| {
                crate::trace_error!(
                    "nexus::agent::loop",
                    "Failed to create Anthropic LLM",
                    error = e.to_string()
                );
                e.to_string()
            })?;
            crate::trace_debug!("nexus::agent::loop", "Anthropic LLM created");
            execute_nexus_worker(llm, prompt).await
        }
        "openai" => {
            let mut llm = OpenAILlm::from_env(model_name).map_err(|e| {
                crate::trace_error!(
                    "nexus::agent::loop",
                    "Failed to create OpenAI LLM",
                    error = e.to_string()
                );
                e.to_string()
            })?;
            if let Some(base_url) = config.base_url {
                if !base_url.is_empty() {
                    crate::trace_debug!(
                        "nexus::agent::loop",
                        "Using custom base URL",
                        base_url = base_url
                    );
                    llm = llm.with_base_url(base_url);
                }
            }
            crate::trace_debug!("nexus::agent::loop", "OpenAI LLM created");
            execute_nexus_worker(llm, prompt).await
        }
        "openrouter" => {
            let llm = OpenRouterLlm::from_env(model_name)
                .map_err(|e| {
                    crate::trace_error!(
                        "nexus::agent::loop",
                        "Failed to create OpenRouter LLM",
                        error = e.to_string()
                    );
                    e.to_string()
                })?
                .with_site_url("https://nexus.local")
                .with_app_name("Nexus Agent");
            crate::trace_debug!("nexus::agent::loop", "OpenRouter LLM created");
            execute_nexus_worker(llm, prompt).await
        }
        "gemini" => {
            let llm = GeminiLlm::from_env(model_name).map_err(|e| {
                crate::trace_error!(
                    "nexus::agent::loop",
                    "Failed to create Gemini LLM",
                    error = e.to_string()
                );
                e.to_string()
            })?;
            crate::trace_debug!("nexus::agent::loop", "Gemini LLM created");
            execute_nexus_worker(llm, prompt).await
        }
        "grok" => {
            let llm = GrokLlm::from_env(model_name).map_err(|e| {
                crate::trace_error!(
                    "nexus::agent::loop",
                    "Failed to create Grok LLM",
                    error = e.to_string()
                );
                e.to_string()
            })?;
            crate::trace_debug!("nexus::agent::loop", "Grok LLM created");
            execute_nexus_worker(llm, prompt).await
        }
        "deepseek" => {
            let llm = DeepSeekLlm::from_env(model_name).map_err(|e| {
                crate::trace_error!(
                    "nexus::agent::loop",
                    "Failed to create DeepSeek LLM",
                    error = e.to_string()
                );
                e.to_string()
            })?;
            crate::trace_debug!("nexus::agent::loop", "DeepSeek LLM created");
            execute_nexus_worker(llm, prompt).await
        }
        _ => {
            crate::trace_error!(
                "nexus::agent::loop",
                "Unsupported provider",
                provider = provider
            );
            Err(format!("Unsupported LLM_PROVIDER: {}", provider))
        }
    }
}
