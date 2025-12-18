use anyhow::Result;
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use std::sync::Arc;

#[derive(Clone)]
pub struct BrowserManager {
    browser: Arc<Browser>,
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

        Ok(Self { browser: Arc::new(browser) })
    }

    pub async fn navigate_and_get_content(&self, url: &str) -> Result<String> {
        let page = self.browser.new_page(url).await?;

        // Wait for page to load
        page.wait_for_navigation().await?;

        // Get content
        let content = page.content().await?;

        // Close page to free resources
        page.close().await?;

        Ok(content)
    }
}
