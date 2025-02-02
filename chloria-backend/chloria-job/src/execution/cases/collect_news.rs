use std::sync::Arc;

use anyhow::Result;

use super::super::{
    tasks::{run, save_news::SaveNewsTask},
    workshop::Workshop,
};

pub(crate) struct CollectNewsCase<'c> {
    workshop: &'c Workshop<'c>,
}

impl<'w> Workshop<'w> {
    pub(crate) fn new_collect_news_case(&self) -> CollectNewsCase {
        CollectNewsCase { workshop: &self }
    }
}

impl<'c> CollectNewsCase<'c> {
    pub(crate) async fn execute(&self) -> Result<()> {
        let file_storage = Arc::clone(&self.workshop.file_storage);
        for handle in self
            .workshop
            .news_fetcher
            .fetch_news(Box::new(move |n| run(SaveNewsTask::new(n, Arc::clone(&file_storage)))))
            .await
        {
            handle.await??;
        }
        Ok(())
    }
}
