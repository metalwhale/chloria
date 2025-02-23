use anyhow::Result;
use log::info;

use crate::execution::workshop::Workshop;

pub(crate) struct Commander<'c> {
    workshop: &'c Workshop,
}

impl<'c> Commander<'c> {
    pub(crate) fn new(workshop: &'c Workshop) -> Self {
        Self { workshop }
    }

    pub(crate) async fn collect_news(&self) -> Result<()> {
        const TASK_PERMITS_NUM: usize = 50;
        const INSERT_BATCH_SIZE: usize = 100;
        let (total_news_count, inserted_news_count) = self
            .workshop
            .execute_collect_news_case(TASK_PERMITS_NUM, INSERT_BATCH_SIZE)
            .await?;
        info!(
            "total_news_count={}, inserted_news_count={}",
            total_news_count, inserted_news_count
        );
        Ok(())
    }
}
