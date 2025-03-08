use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;
use mockall::automock;
use serde::Serialize;

pub(crate) struct SelectNewsInput {
    pub(crate) date: NaiveDate,
}

#[derive(Serialize)]
pub(crate) struct SelectNewsOutput {
    pub(crate) text: Option<String>,
    pub(crate) image_path: Option<String>,
}

#[automock]
#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn select_client_api_secret(&self, api_key_input: &str) -> Result<Option<String>>;
    async fn select_news(&self, input: SelectNewsInput) -> Result<Vec<SelectNewsOutput>>;
}
