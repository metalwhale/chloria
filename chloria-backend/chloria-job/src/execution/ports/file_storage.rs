use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use mockall::automock;

pub(crate) enum FileObjectKind {
    Origin,
}

pub(crate) struct UploadFileInput {
    pub(crate) kind: FileObjectKind,
    pub(crate) source_name: String, // Code name of the source used to fetch the news
    pub(crate) key: String,         // Unique key to differentiate this file from others
    pub(crate) bytes: Vec<u8>,      // Content of the file
    pub(crate) created_time: DateTime<Local>, // Date and time when the file was created
}

#[async_trait(?Send)]
#[automock] // See: https://github.com/asomers/mockall/issues/189#issuecomment-689145249
pub(crate) trait FileStorage {
    async fn upload_file(&self, input: UploadFileInput) -> Result<String>;
}
