use anyhow::Result;

use super::config::Config;
use crate::execution::cases::collect_news::CollectNewsCase;

pub(crate) struct Commander<'c> {
    config: Config<'c>,
}

impl<'c> Commander<'c> {
    pub(crate) fn new(config: Config<'c>) -> Self {
        Self { config }
    }

    pub(crate) async fn collect_news(&self) -> Result<()> {
        let case = CollectNewsCase::new(self.config.file_storage);
        case.execute().await?;
        Ok(())
    }
}
