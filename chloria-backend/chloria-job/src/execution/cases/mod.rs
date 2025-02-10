mod collect_news;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait(?Send)]
pub(super) trait LocalCase: Send + Sync + 'static {
    type Output: Send;

    async fn execute(self) -> Result<Self::Output>;
}
