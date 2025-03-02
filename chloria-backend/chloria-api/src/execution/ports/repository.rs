use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;

#[async_trait]
#[automock] // See: https://github.com/asomers/mockall/issues/189#issuecomment-689145249
pub(crate) trait Repository: Send + Sync {
    async fn select_client_api_secret(&self, api_key_input: &str) -> Result<Option<String>>;
}
