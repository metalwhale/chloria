use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn select_client_api_secret(&self, api_key_input: &str) -> Result<Option<String>>;
}
