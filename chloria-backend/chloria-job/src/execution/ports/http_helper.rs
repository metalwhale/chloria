use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub(crate) trait HttpHelper {
    async fn get(&self, url: &str) -> Result<Vec<u8>>;
}
