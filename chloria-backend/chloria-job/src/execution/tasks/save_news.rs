use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};

use super::{
    super::ports::{
        file_storage::{FileObjectKind, FileStorage, UploadFileInput},
        http_helper::HttpHelper,
        repository::{InsertNewsInput, Repository},
    },
    LocalTask,
};

pub(in super::super) struct SaveNewsInput {
    pub(in super::super) source_name: String, // Code name of the source used to fetch the news
    pub(in super::super) article_id: String,  // Unique article ID for news from the same source
    pub(in super::super) link: Option<String>, // Link to the original content
    pub(in super::super) title: Option<String>, // Title of the content
    pub(in super::super) short_text: Option<String>, // Short description or summary of the content
    pub(in super::super) long_text: Option<String>, // Full text or detailed content
    pub(in super::super) image_url: Option<String>, // Url of representative image in the original content
    pub(in super::super) published_time: Option<DateTime<Local>>, // Date and time when the news was published
}

pub(in super::super) struct SaveNewsTask {
    input: SaveNewsInput,
    http_helper: Arc<dyn HttpHelper>,
    file_storage: Arc<dyn FileStorage>,
    repository: Arc<dyn Repository>,
}

impl SaveNewsTask {
    pub(in super::super) fn new(
        input: SaveNewsInput,
        http_helper: Arc<dyn HttpHelper>,
        file_storage: Arc<dyn FileStorage>,
        repository: Arc<dyn Repository>,
    ) -> Self {
        Self {
            input,
            http_helper,
            file_storage,
            repository,
        }
    }
}

#[async_trait(?Send)]
impl LocalTask for SaveNewsTask {
    type Output = Option<i32>;

    async fn perform(self) -> Result<Self::Output> {
        let image_path = match &self.input.image_url {
            Some(image_url) => {
                if let Ok(image_bytes) = self.http_helper.get(&image_url).await {
                    self.file_storage
                        .upload_file(UploadFileInput {
                            kind: FileObjectKind::Origin,
                            source_name: self.input.source_name.clone(),
                            key: format!("{}.jpg", self.input.article_id),
                            bytes: image_bytes,
                            created_time: Local::now(),
                        })
                        .await
                        .ok()
                } else {
                    None
                }
            }
            None => None,
        };
        let news_id = tokio::spawn(async move {
            self.repository
                .insert_news(InsertNewsInput {
                    source_name: self.input.source_name,
                    article_id: self.input.article_id,
                    link: self.input.link,
                    title: self.input.title,
                    short_text: self.input.short_text,
                    long_text: self.input.long_text,
                    image_path,
                    published_time: self.input.published_time,
                })
                .await
        })
        .await??;
        Ok(news_id)
    }
}
