pub(super) mod save_news;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait(?Send)]
pub(super) trait Task {
    type Output: Send;

    async fn perform(self) -> Result<Self::Output>;
}
