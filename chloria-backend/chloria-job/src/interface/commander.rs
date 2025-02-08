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
        let count = self.workshop.execute_collect_news_case().await?;
        info!("count={}", count);
        Ok(())
    }
}
