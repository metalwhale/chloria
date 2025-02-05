use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use super::{
    super::ports::{
        file_storage::{FileObjectKind, FileStorage, UploadFileInput},
        http_helper::HttpHelper,
        news_fetcher::FetchNewsOutput,
    },
    Task,
};
use crate::domain::file::FileEntity;

pub(in super::super) struct SaveNewsTask {
    fetch_news_output: FetchNewsOutput,
    http_helper: Arc<dyn HttpHelper + Send + Sync>,
    file_storage: Arc<dyn FileStorage + Send + Sync>,
}

impl SaveNewsTask {
    pub(in super::super) fn new(
        fetch_news_output: FetchNewsOutput,
        http_helper: Arc<dyn HttpHelper + Send + Sync>,
        file_storage: Arc<dyn FileStorage + Send + Sync>,
    ) -> Self {
        Self {
            fetch_news_output,
            http_helper,
            file_storage,
        }
    }
}

#[async_trait]
impl Task for SaveNewsTask {
    type Output = ();

    async fn perform(self) -> Result<Self::Output> {
        if let Some(image_url) = &self.fetch_news_output.image_url {
            let file = FileEntity::new(self.fetch_news_output.id, self.fetch_news_output.published_time);
            let image_bytes = self.http_helper.get(&image_url).await?;
            self.file_storage
                .upload_file(UploadFileInput {
                    kind: FileObjectKind::Origin,
                    source_name: self.fetch_news_output.source_name,
                    key: file.key,
                    bytes: image_bytes,
                    created_time: file.created_time,
                })
                .await?;
        }
        Ok(())
    }
}
