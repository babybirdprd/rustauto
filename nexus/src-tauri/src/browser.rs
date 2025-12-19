use anyhow::Result;
use chromiumoxide::cdp::browser_protocol::dom::SetFileInputFilesParams;
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout, Duration};

pub static GLOBAL_BROWSER: OnceLock<BrowserManager> = OnceLock::new();

#[derive(Clone)]
pub struct BrowserManager {
    browser: Arc<Browser>,
    current_page: Arc<Mutex<Option<Page>>>,
}

impl BrowserManager {
    pub async fn new() -> Result<Self> {
        crate::trace_info!("nexus::browser", "Launching headless browser");

        // Launch headless browser
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .build()
                .map_err(|e| anyhow::anyhow!(e))?,
        )
        .await?;

        crate::trace_debug!(
            "nexus::browser",
            "Browser process started, spawning handler"
        );

        // Spawn the handler loop
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        crate::trace_info!("nexus::browser", "BrowserManager initialized successfully");
        Ok(Self {
            browser: Arc::new(browser),
            current_page: Arc::new(Mutex::new(None)),
        })
    }

    async fn wait_for_selector(page: &Page, selector: &str) -> Result<chromiumoxide::Element> {
        crate::trace_debug!(
            "nexus::browser",
            "Waiting for selector",
            selector = selector
        );
        let start = std::time::Instant::now();
        let wait_timeout = Duration::from_secs(5);

        loop {
            match page.find_element(selector).await {
                Ok(element) => {
                    crate::trace_debug!(
                        "nexus::browser",
                        "Selector found",
                        selector = selector,
                        elapsed_ms = start.elapsed().as_millis() as u64
                    );
                    return Ok(element);
                }
                Err(_) => {
                    if start.elapsed() > wait_timeout {
                        crate::trace_error!(
                            "nexus::browser",
                            "Selector timeout",
                            selector = selector
                        );
                        return Err(anyhow::anyhow!(
                            "Element '{}' not found after 5 seconds",
                            selector
                        ));
                    }
                    sleep(Duration::from_millis(200)).await;
                }
            }
        }
    }

    pub async fn navigate_and_get_content(&self, url: &str) -> Result<String> {
        crate::trace_info!("nexus::browser", "Starting navigation", url = url);
        let timeout_duration = Duration::from_secs(30);

        let result = timeout(timeout_duration, async {
            crate::trace_debug!("nexus::browser", "Creating new page");
            let page = self.browser.new_page(url).await?;
            crate::trace_debug!("nexus::browser", "Page created, waiting for navigation");
            // Wait for page to load
            page.wait_for_navigation().await?;
            crate::trace_debug!("nexus::browser", "Navigation complete, getting content");
            // Get content
            let content = page.content().await?;
            crate::trace_debug!(
                "nexus::browser",
                "Content retrieved",
                content_len = content.len()
            );
            Ok::<_, anyhow::Error>((page, content))
        })
        .await;

        match result {
            Ok(Ok((page, content))) => {
                crate::trace_debug!("nexus::browser", "Updating current page reference");
                let mut guard = self.current_page.lock().await;
                if let Some(old_page) = guard.take() {
                    crate::trace_debug!("nexus::browser", "Closing previous page");
                    // Best effort close
                    let _ = old_page.close().await;
                }
                *guard = Some(page);

                // Emit event for UI update
                if let Some(app) = crate::GLOBAL_APP.get() {
                    use serde_json::json;
                    use tauri::Emitter;
                    let _ = app.emit(
                        "browser-update",
                        json!({
                            "url": url,
                        }),
                    );
                }

                crate::trace_info!(
                    "nexus::browser",
                    "Navigation successful",
                    url = url,
                    content_len = content.len()
                );
                Ok(content)
            }
            Ok(Err(e)) => {
                crate::trace_error!(
                    "nexus::browser",
                    "Navigation error",
                    url = url,
                    error = e.to_string()
                );
                Err(e)
            }
            Err(_) => {
                crate::trace_error!("nexus::browser", "Navigation timeout", url = url);
                Err(anyhow::anyhow!("Navigation timed out after 30 seconds"))
            }
        }
    }

    pub async fn get_current_url(&self) -> Result<String> {
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            let url = page
                .url()
                .await?
                .unwrap_or_else(|| "about:blank".to_string());
            Ok(url)
        } else {
            Ok("".to_string())
        }
    }

    pub async fn take_screenshot(&self) -> Result<String> {
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            // chromiumoxide's screenshot returns Vec<u8>
            let screenshot_data = page.screenshot(
                chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams::builder()
                    .format(chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png)
                    .build()
            ).await?;

            use base64::{engine::general_purpose, Engine as _};
            let base64_image = general_purpose::STANDARD.encode(screenshot_data);
            Ok(format!("data:image/png;base64,{}", base64_image))
        } else {
            Err(anyhow::anyhow!("No active page to screenshot"))
        }
    }

    pub async fn click_element(&self, selector: &str) -> Result<String> {
        crate::trace_info!(
            "nexus::browser",
            "Click element requested",
            selector = selector
        );
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            let timeout_duration = Duration::from_secs(30);
            let selector_owned = selector.to_string();
            let selector_for_log = selector_owned.clone();
            let page_clone = page.clone();

            crate::trace_debug!("nexus::browser", "Starting click operation with timeout");
            let result = timeout(timeout_duration, async move {
                crate::trace_debug!("nexus::browser", "Waiting for element");
                let element = Self::wait_for_selector(&page_clone, &selector_owned).await?;
                crate::trace_debug!("nexus::browser", "Element found, clicking");
                element.click().await?;
                crate::trace_debug!("nexus::browser", "Click executed, getting page content");
                let content = page_clone.content().await?;
                Ok::<_, anyhow::Error>(content)
            })
            .await;

            match result {
                Ok(r) => {
                    crate::trace_info!(
                        "nexus::browser",
                        "Click operation successful",
                        selector = selector_for_log
                    );
                    r
                }
                Err(_) => {
                    crate::trace_error!(
                        "nexus::browser",
                        "Click operation timeout",
                        selector = selector_for_log
                    );
                    Err(anyhow::anyhow!("Click action timed out after 30 seconds"))
                }
            }
        } else {
            crate::trace_error!("nexus::browser", "No active page for click operation");
            Err(anyhow::anyhow!("No active page. Navigate to a URL first."))
        }
    }

    pub async fn type_text(&self, text: &str) -> Result<String> {
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            let timeout_duration = Duration::from_secs(30);
            let text = text.to_string();
            let page_clone = page.clone();

            let result = timeout(timeout_duration, async move {
                // For typing, we usually type into the focused element or we should accept a selector.
                // The current implementation finds ":focus".
                match page_clone.find_element(":focus").await {
                    Ok(element) => {
                        element.type_str(&text).await?;
                    }
                    Err(_) => {
                        return Err(anyhow::anyhow!(
                            "Could not find focused element to type into."
                        ));
                    }
                }
                let content = page_clone.content().await?;
                Ok::<_, anyhow::Error>(content)
            })
            .await;

            match result {
                Ok(r) => r,
                Err(_) => Err(anyhow::anyhow!("Type action timed out after 30 seconds")),
            }
        } else {
            Err(anyhow::anyhow!("No active page. Navigate to a URL first."))
        }
    }

    pub async fn upload_file(&self, selector: &str, file_path: &str) -> Result<String> {
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            let timeout_duration = Duration::from_secs(30);
            let selector = selector.to_string();
            let file_path = file_path.to_string();
            let page_clone = page.clone();

            let result = timeout(timeout_duration, async move {
                let element = Self::wait_for_selector(&page_clone, &selector).await?;
                // We use CDP directly since set_input_files helper is missing
                page_clone
                    .execute(
                        SetFileInputFilesParams::builder()
                            .files(vec![file_path])
                            .node_id(element.node_id)
                            .build()
                            .unwrap(),
                    )
                    .await?;
                let content = page_clone.content().await?;
                Ok::<_, anyhow::Error>(content)
            })
            .await;

            match result {
                Ok(r) => r,
                Err(e) => Err(anyhow::anyhow!("Upload action failed: {}", e)),
            }
        } else {
            Err(anyhow::anyhow!("No active page. Navigate to a URL first."))
        }
    }

    pub async fn scroll_page(&self, direction: &str, amount: Option<i32>) -> Result<String> {
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            let timeout_duration = Duration::from_secs(10); // Scroll should be fast
            let val = amount.unwrap_or(500);
            let delta = if direction == "up" { -val } else { val };
            let page_clone = page.clone();

            let result = timeout(timeout_duration, async move {
                page_clone
                    .evaluate(format!("window.scrollBy(0, {})", delta))
                    .await?;
                let content = page_clone.content().await?;
                Ok::<_, anyhow::Error>(content)
            })
            .await;

            match result {
                Ok(r) => r,
                Err(_) => Err(anyhow::anyhow!("Scroll action timed out")),
            }
        } else {
            Err(anyhow::anyhow!("No active page. Navigate to a URL first."))
        }
    }

    pub async fn get_content(&self) -> Result<String> {
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            let content = page.content().await?;
            Ok(content)
        } else {
            Err(anyhow::anyhow!("No active page. Navigate to a URL first."))
        }
    }

    pub async fn reset(&self) -> Result<()> {
        let mut guard = self.current_page.lock().await;
        if let Some(page) = guard.take() {
            let _ = page.close().await;
        }
        Ok(())
    }
}
