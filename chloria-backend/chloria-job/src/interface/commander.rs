use anyhow::Result;

use crate::execution::workshop::Workshop;

pub(crate) struct Commander<'c> {
    workshop: &'c Workshop<'c>,
}

impl<'c> Commander<'c> {
    pub(crate) fn new(workshop: &'c Workshop) -> Self {
        Self { workshop }
    }

    pub(crate) async fn collect_news(&self) -> Result<()> {
        let case = self.workshop.new_collect_news_case();
        case.execute().await?;
        Ok(())
    }
}
