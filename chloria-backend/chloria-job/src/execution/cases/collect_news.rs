use std::sync::Arc;

use anyhow::Result;

use super::super::{
    factory::Factory,
    ports::news_fetcher::NewsFetcher,
    tasks::{run, save_news::SaveNewsTask},
};

pub(crate) struct CollectNewsCase<'c> {
    news_fetcher: &'c (dyn NewsFetcher + Send + Sync),
    factory: &'c Factory<'c>,
}

impl<'f> Factory<'f> {
    pub(crate) fn new_collect_news_case(&self) -> CollectNewsCase {
        CollectNewsCase {
            news_fetcher: self.news_fetcher,
            factory: &self,
        }
    }
}

impl<'c> CollectNewsCase<'c> {
    pub(crate) async fn execute(&self) -> Result<()> {
        let http_helper = Arc::clone(&self.factory.http_helper);
        let file_storage = Arc::clone(&self.factory.file_storage);
        for handle in self
            .news_fetcher
            .fetch_news(Box::new(move |n| {
                run(SaveNewsTask::new(
                    n,
                    Arc::clone(&http_helper),
                    Arc::clone(&file_storage),
                ))
            }))
            .await
        {
            handle.await??;
        }
        Ok(())
    }
}
