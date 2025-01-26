use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use mockall::automock;

pub(crate) struct InsertNewsInput {
    pub(crate) source_name: String,        // Code name of the source used to fetch the news
    pub(crate) article_id: String,         // Unique ID of the article
    pub(crate) link: Option<String>,       // Link to the original content
    pub(crate) title: Option<String>,      // Title of the content
    pub(crate) short_text: Option<String>, // Short description or summary of the content
    pub(crate) long_text: Option<String>,  // Full text or detailed content
    pub(crate) image_path: Option<String>, // Path of representative image saved in file storage
    pub(crate) published_time: Option<DateTime<Local>>, // Date and time when the news was published
}

#[automock]
#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn insert_news(&self, input: InsertNewsInput) -> Result<Option<i32>>;
}
