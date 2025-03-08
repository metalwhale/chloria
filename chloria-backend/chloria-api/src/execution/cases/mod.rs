pub(crate) mod authenticate;
pub(crate) mod read_news;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub(super) trait Case: Send + Sync + 'static {
    type Output: Send;

    async fn execute(self) -> Result<Self::Output>;
}
