use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use tokio::task::JoinHandle;

pub(crate) struct FetchNewsOutput {
    pub(crate) source_name: String,        // Code name of the source used to fetch the news
    pub(crate) id: Option<String>,         // Unique ID for news from the same source
    pub(crate) link: Option<String>,       // Link to the original content
    pub(crate) title: Option<String>,      // Title of the content
    pub(crate) short_text: Option<String>, // Short description or summary of the content
    pub(crate) long_text: Option<String>,  // Full text or detailed content
    pub(crate) image_url: Option<String>,  // Url of representative image in the original content
    pub(crate) published_time: Option<DateTime<Local>>, // Date and time when the news was published
}

#[async_trait]
pub(crate) trait NewsFetcher {
    async fn fetch_news(
        &self,
        handler: Box<dyn Fn(FetchNewsOutput) -> JoinHandle<Result<()>> + Send>,
    ) -> Vec<JoinHandle<Result<()>>>;
}
