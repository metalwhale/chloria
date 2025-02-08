use anyhow::Result;
use log::info;

use crate::execution::workshop::Workshop;

pub(crate) struct Commander<'c> {
    workshop: &'c Workshop<'c>,
}

impl<'c> Commander<'c> {
    pub(crate) fn new(workshop: &'c Workshop) -> Self {
        Self { workshop }
    }

    pub(crate) async fn collect_news(&self) -> Result<()> {
        const TASK_PERMITS_NUM: usize = 10;
        let case = self.workshop.new_collect_news_case(TASK_PERMITS_NUM);
        let count = case.execute().await?;
        info!("count={}", count);
        Ok(())
    }
}
