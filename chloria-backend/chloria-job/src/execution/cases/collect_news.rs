use anyhow::Result;
use reqwest::Client;

use crate::{
    domain::file::FileEntity,
    execution::ports::{
        file_storage::{FileObjectKind, FileStorage, UploadFileInput},
        news_fetcher::NewsFetcher,
    },
};

pub(crate) struct CollectNewsCase<'c> {
    news_fetcher: &'c dyn NewsFetcher,
    file_storage: &'c dyn FileStorage,
}

impl<'c> CollectNewsCase<'c> {
    pub(crate) fn new(news_fetcher: &'c dyn NewsFetcher, file_storage: &'c dyn FileStorage) -> Self {
        Self {
            news_fetcher,
            file_storage,
        }
    }

    pub(crate) async fn execute(&self) -> Result<()> {
        for news in self.news_fetcher.fetch_news().await? {
            if let Some(image_url) = news.image_url {
                let file = FileEntity::new(news.id, news.published_time);
                let image_bytes: Vec<u8> = Client::new().get(image_url).send().await?.bytes().await?.into();
                self.file_storage
                    .upload_file(UploadFileInput {
                        kind: FileObjectKind::Origin,
                        source_name: news.source_name,
                        key: file.key,
                        bytes: image_bytes,
                        created_time: file.created_time,
                    })
                    .await?;
            }
        }
        Ok(())
    }
}
