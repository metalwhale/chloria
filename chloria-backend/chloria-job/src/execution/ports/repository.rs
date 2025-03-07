use std::{future::Future, pin::Pin};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use mockall::mock;

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

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn insert_news(&self, inputs: Vec<InsertNewsInput>) -> Result<Vec<i32>>;
}

mock! {
    pub(in super::super) Repository {}

    impl Repository for Repository {
        fn insert_news<'life0, 'async_trait>(
            &'life0 self,
            inputs: Vec<InsertNewsInput>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<i32>>> + Send + 'async_trait>>
        where
            'life0: 'async_trait,
            Self: 'async_trait;
    }
}
