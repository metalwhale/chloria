use std::{future::Future, pin::Pin};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use mockall::mock;
use tokio::task::JoinHandle;

pub(crate) struct FetchNewsArticle {
    pub(crate) source_name: String,        // Code name of the source used to fetch the news
    pub(crate) id: Option<String>,         // Unique ID for news from the same source
    pub(crate) link: Option<String>,       // Link to the original content
    pub(crate) title: Option<String>,      // Title of the content
    pub(crate) short_text: Option<String>, // Short description or summary of the content
    pub(crate) long_text: Option<String>,  // Full text or detailed content
    pub(crate) image_url: Option<String>,  // Url of representative image in the original content
    pub(crate) published_time: Option<DateTime<Local>>, // Date and time when the news was published
}

pub(crate) type FetchNewsOutput = JoinHandle<Result<bool>>;
pub(crate) type FetchNewsHandler = Box<dyn Fn(FetchNewsArticle) -> FetchNewsOutput + Send>;

#[async_trait]
pub(crate) trait NewsFetcher {
    async fn fetch_news(&self, handler: FetchNewsHandler) -> Vec<FetchNewsOutput>;
}

mock! {
    pub(in super::super) NewsFetcher {}

    impl NewsFetcher for NewsFetcher {
        fn fetch_news<'life0, 'async_trait>(
            &'life0 self,
            handler: FetchNewsHandler,
        ) -> Pin<Box<dyn Future<Output = Vec<FetchNewsOutput>> + Send + 'async_trait>>
        where
            'life0: 'async_trait,
            Self: 'async_trait;
    }
}
