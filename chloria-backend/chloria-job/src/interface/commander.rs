use anyhow::Result;

use crate::execution::factory::Factory;

pub(crate) struct Commander<'c> {
    factory: &'c Factory<'c>,
}

impl<'c> Commander<'c> {
    pub(crate) fn new(factory: &'c Factory) -> Self {
        Self { factory }
    }

    pub(crate) async fn collect_news(&self) -> Result<()> {
        let case = self.factory.new_collect_news_case();
        case.execute().await?;
        Ok(())
    }
}
