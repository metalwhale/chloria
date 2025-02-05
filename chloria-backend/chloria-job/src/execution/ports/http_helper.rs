use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;

#[async_trait]
#[automock] // See: https://github.com/asomers/mockall/issues/189#issuecomment-689145249
pub(crate) trait HttpHelper {
    async fn get(&self, url: &str) -> Result<Vec<u8>>;
}
