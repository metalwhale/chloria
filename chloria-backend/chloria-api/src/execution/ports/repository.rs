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
    pub(crate) source_name: String,
    pub(crate) article_id: String,
    pub(crate) title: Option<String>,
    pub(crate) text: Option<String>,
    pub(crate) image_path: Option<String>,
}

pub(crate) struct InsertNewsInsightInput {
    pub(crate) source_name: String,
    pub(crate) article_id: String,
    pub(crate) fields: String,
}

#[async_trait]
#[automock] // See: https://github.com/asomers/mockall/issues/189#issuecomment-689145249
pub(crate) trait Repository: Send + Sync {
    async fn select_client_api_secret(&self, api_key_input: &str) -> Result<Option<String>>;
    async fn select_news(&self, input: SelectNewsInput) -> Result<Vec<SelectNewsOutput>>;
    async fn insert_news_insight(&self, input: InsertNewsInsightInput) -> Result<()>;
}
