pub(super) mod save_news;

use anyhow::Result;
use async_trait::async_trait;
use tokio::task::JoinHandle;

#[async_trait]
pub(super) trait Task {
    type Output: Send;

    async fn perform(self) -> Result<Self::Output>;
}

pub(super) fn run<T: Task + Send + Sync + 'static>(task: T) -> JoinHandle<Result<T::Output>> {
    tokio::spawn(async move { task.perform().await })
}
