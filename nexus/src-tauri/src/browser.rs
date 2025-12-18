use anyhow::Result;
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Mutex;

pub static GLOBAL_BROWSER: OnceLock<BrowserManager> = OnceLock::new();

#[derive(Clone)]
pub struct BrowserManager {
    browser: Arc<Browser>,
    current_page: Arc<Mutex<Option<Page>>>,
}

impl BrowserManager {
    pub async fn new() -> Result<Self> {
        // Launch headless browser
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .build()
                .map_err(|e| anyhow::anyhow!(e))?
        ).await?;

        // Spawn the handler loop
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            browser: Arc::new(browser),
            current_page: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn navigate_and_get_content(&self, url: &str) -> Result<String> {
        let page = self.browser.new_page(url).await?;

        // Wait for page to load
        page.wait_for_navigation().await?;

        // Get content
        let content = page.content().await?;

        // Update current_page
        let mut guard = self.current_page.lock().await;
        if let Some(old_page) = guard.take() {
            // Best effort close
            let _ = old_page.close().await;
        }
        *guard = Some(page);

        Ok(content)
    }

    pub async fn click_element(&self, selector: &str) -> Result<String> {
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            let element = page.find_element(selector).await?;
            element.click().await?;
            // Return updated content
            Ok(page.content().await?)
        } else {
            Err(anyhow::anyhow!("No active page. Navigate to a URL first."))
        }
    }

    pub async fn type_text(&self, text: &str) -> Result<String> {
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            page.keyboard().type_str(text).await?;
            Ok(page.content().await?)
        } else {
             Err(anyhow::anyhow!("No active page. Navigate to a URL first."))
        }
    }

    pub async fn scroll_page(&self, direction: &str, amount: Option<i32>) -> Result<String> {
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            let val = amount.unwrap_or(500);
            let delta = if direction == "up" { -val } else { val };

            page.evaluate(format!("window.scrollBy(0, {})", delta)).await?;
            Ok(page.content().await?)
        } else {
             Err(anyhow::anyhow!("No active page. Navigate to a URL first."))
        }
    }
}
