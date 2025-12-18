use anyhow::Result;
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

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
        let timeout_duration = Duration::from_secs(30);

        let result = timeout(timeout_duration, async {
            let page = self.browser.new_page(url).await?;
            // Wait for page to load
            page.wait_for_navigation().await?;
            // Get content
            let content = page.content().await?;
            Ok::<_, anyhow::Error>((page, content))
        }).await;

        match result {
            Ok(Ok((page, content))) => {
                let mut guard = self.current_page.lock().await;
                if let Some(old_page) = guard.take() {
                    // Best effort close
                    let _ = old_page.close().await;
                }
                *guard = Some(page);
                Ok(content)
            },
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow::anyhow!("Navigation timed out after 30 seconds")),
        }
    }

    pub async fn click_element(&self, selector: &str) -> Result<String> {
        let guard = self.current_page.lock().await;
        if let Some(page) = guard.as_ref() {
            let timeout_duration = Duration::from_secs(30);
            let selector = selector.to_string();
            let page_clone = page.clone();

            let result = timeout(timeout_duration, async move {
                 let element = page_clone.find_element(&selector).await?;
                 element.click().await?;
                 let content = page_clone.content().await?;
                 Ok::<_, anyhow::Error>(content)
            }).await;

            match result {
                Ok(r) => r,
                Err(_) => Err(anyhow::anyhow!("Click action timed out after 30 seconds")),
            }
        } else {
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
                match page_clone.find_element(":focus").await {
                    Ok(element) => {
                        element.type_str(&text).await?;
                    },
                    Err(_) => {
                         return Err(anyhow::anyhow!("Could not find focused element to type into."));
                    }
                }
                let content = page_clone.content().await?;
                Ok::<_, anyhow::Error>(content)
             }).await;

             match result {
                Ok(r) => r,
                Err(_) => Err(anyhow::anyhow!("Type action timed out after 30 seconds")),
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
                page_clone.evaluate(format!("window.scrollBy(0, {})", delta)).await?;
                let content = page_clone.content().await?;
                Ok::<_, anyhow::Error>(content)
            }).await;

             match result {
                Ok(r) => r,
                Err(_) => Err(anyhow::anyhow!("Scroll action timed out")),
             }
        } else {
             Err(anyhow::anyhow!("No active page. Navigate to a URL first."))
        }
    }
}
