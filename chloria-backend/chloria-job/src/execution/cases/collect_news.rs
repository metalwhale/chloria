use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::Local;
use log::error;
use tokio::sync::{mpsc, Semaphore};

use super::{
    super::{
        ports::{
            file_storage::{FileObjectKind, FileStorage, UploadFileInput},
            http_helper::HttpHelper,
            news_fetcher::NewsFetcher,
            repository::{InsertNewsInput, Repository},
        },
        workshop::Workshop,
    },
    LocalCase,
};
use crate::domain::news::NewsEntity;

type CollectNewsCaseOutput = (usize, usize);

struct CollectNewsCase {
    news_fetchers: Vec<Arc<dyn NewsFetcher>>,
    http_helper: Arc<dyn HttpHelper>,
    file_storage: Arc<dyn FileStorage>,
    repository: Arc<dyn Repository>,
    task_permits_num: usize,
    insert_batch_size: usize,
}

impl Workshop {
    pub(crate) async fn execute_collect_news_case(
        &self,
        task_permits_num: usize,
        insert_batch_size: usize,
    ) -> Result<CollectNewsCaseOutput> {
        let case = CollectNewsCase {
            news_fetchers: self.news_fetchers.iter().map(|f| Arc::clone(f)).collect(),
            http_helper: Arc::clone(&self.http_helper),
            file_storage: Arc::clone(&self.file_storage),
            repository: Arc::clone(&self.repository),
            task_permits_num,
            insert_batch_size,
        };
        self.run_local_case(case).await
    }
}

#[async_trait(?Send)]
impl LocalCase for CollectNewsCase {
    type Output = CollectNewsCaseOutput;

    async fn execute(self) -> Result<Self::Output> {
        const CHANNEL_CAPACITY: usize = 100;
        let (sender, mut receiver) = mpsc::channel(CHANNEL_CAPACITY);
        // Save news to the database
        let receiver_handle = tokio::spawn(async move {
            let mut insert_news_inputs = vec![];
            let mut inserted_news_count = 0;
            while let Some(input) = receiver.recv().await {
                insert_news_inputs.push(input);
                if insert_news_inputs.len() >= self.insert_batch_size {
                    let inputs = insert_news_inputs.drain(..).collect();
                    match self.repository.insert_news(inputs).await {
                        Ok(news_ids) => {
                            inserted_news_count += news_ids.len();
                        }
                        Err(error) => error!("error={}", error),
                    }
                }
            }
            // Remaining news after the channel closed
            match self.repository.insert_news(insert_news_inputs).await {
                Ok(news_ids) => {
                    inserted_news_count += news_ids.len();
                }
                Err(error) => error!("error={}", error),
            }
            inserted_news_count
        });
        // Fetch news from providers
        let semaphore = Arc::new(Semaphore::new(self.task_permits_num));
        let mut total_news_count = 0;
        for news_fetcher in self.news_fetchers {
            let http_helper = Arc::clone(&self.http_helper);
            let file_storage = Arc::clone(&self.file_storage);
            let sender = sender.clone();
            let semaphore = Arc::clone(&semaphore);
            let handles = news_fetcher
                .fetch_news(Arc::new(move |article| {
                    let news = NewsEntity::new(article.id);
                    let http_helper = Arc::clone(&http_helper);
                    let file_storage = Arc::clone(&file_storage);
                    let sender = sender.clone();
                    let semaphore = Arc::clone(&semaphore);
                    tokio::task::spawn_local(async move {
                        let _permit = semaphore.acquire().await?;
                        let image_path = match article.image_url {
                            Some(image_url) => match http_helper.get(&image_url).await {
                                Ok(image_bytes) => file_storage
                                    .upload_file(UploadFileInput {
                                        kind: FileObjectKind::Origin,
                                        source_name: article.source_name.clone(),
                                        key: format!("{}.jpg", news.article_id),
                                        bytes: image_bytes,
                                        created_time: Local::now(),
                                    })
                                    .await
                                    .ok(),
                                Err(_) => None,
                            },
                            None => None,
                        };
                        sender
                            .send(InsertNewsInput {
                                source_name: article.source_name,
                                article_id: news.article_id,
                                link: article.link,
                                title: article.title,
                                short_text: article.short_text,
                                long_text: article.long_text,
                                image_path,
                                published_time: article.published_time,
                            })
                            .await?;
                        Ok(())
                    })
                }))
                .await;
            total_news_count += handles.len();
        }
        drop(sender); // Drop early (before awaiting the receiver) to prevent the sender from blocking the channel from closing
        let inserted_news_count = receiver_handle.await?;
        Ok((total_news_count, inserted_news_count))
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use anyhow::Result;
    use chrono::Local;
    use tokio::time;

    use super::super::super::{
        ports::{
            file_storage::MockFileStorage,
            http_helper::MockHttpHelper,
            news_fetcher::{FetchNewsArticle, FetchNewsHandler, FetchNewsOutput, MockNewsFetcher},
            repository::MockRepository,
        },
        workshop::{Config, Workshop},
    };

    #[tokio::test]
    async fn check_required_duration() -> Result<()> {
        const CASE_PERMITS_NUM: usize = 2;
        const CASES_NUM: usize = 3;
        const PAGES_NUM: usize = 2;
        const PAGE_LOAD_DURATION: usize = 1000;
        const PAGE_NEWS_NUM: usize = 6;
        const NEWS_LOAD_DURATION: usize = 2000;
        const TASK_PERMITS_NUM: usize = 10;
        const INSERT_BATCH_SIZE: usize = 5;
        // Some assumptions for simpler calculation
        assert!(CASES_NUM > CASE_PERMITS_NUM);
        assert!(PAGES_NUM * PAGE_NEWS_NUM > TASK_PERMITS_NUM);
        assert!(NEWS_LOAD_DURATION > PAGE_LOAD_DURATION);
        async fn fetch_news(handler: FetchNewsHandler) -> Vec<FetchNewsOutput> {
            let mut outputs = vec![];
            for _ in 0..PAGES_NUM {
                time::sleep(Duration::from_millis(PAGE_LOAD_DURATION as u64)).await;
                for _ in 0..PAGE_NEWS_NUM {
                    outputs.push(handler(FetchNewsArticle {
                        source_name: "NewsData".to_string(),
                        id: None,
                        link: None,
                        title: None,
                        short_text: None,
                        long_text: None,
                        image_url: Some("".to_string()),
                        published_time: None,
                    }));
                }
            }
            outputs
        }
        async fn get() -> Result<Vec<u8>> {
            time::sleep(Duration::from_millis(NEWS_LOAD_DURATION as u64)).await;
            Ok(vec![])
        }
        let mut mock_news_fetcher = MockNewsFetcher::new();
        mock_news_fetcher
            .expect_fetch_news()
            .returning(|h| Box::pin(fetch_news(h)));
        let mut mock_http_helper = MockHttpHelper::new();
        mock_http_helper
            .expect_get()
            .times(CASES_NUM * PAGES_NUM * PAGE_NEWS_NUM)
            .returning(|_| Box::pin(get()));
        let mut mock_file_storage = MockFileStorage::new();
        mock_file_storage
            .expect_upload_file()
            .times(CASES_NUM * PAGES_NUM * PAGE_NEWS_NUM)
            .returning(|_| Box::pin(async { Ok("".to_string()) }));
        let mut mock_repository = MockRepository::new();
        mock_repository
            .expect_insert_news()
            .times(CASES_NUM * ((PAGES_NUM * PAGE_NEWS_NUM) as f64 / INSERT_BATCH_SIZE as f64).ceil() as usize)
            .returning(|_| Ok(vec![]));
        let workshop = Workshop::new(
            vec![Arc::new(mock_news_fetcher)],
            Arc::new(mock_http_helper),
            Arc::new(mock_file_storage),
            Arc::new(mock_repository),
            Config {
                case_permits_num: CASE_PERMITS_NUM,
            },
        );
        let start_time = Local::now();
        let mut cases = vec![];
        for _ in 0..CASES_NUM {
            cases.push(workshop.execute_collect_news_case(TASK_PERMITS_NUM, INSERT_BATCH_SIZE));
        }
        futures::future::join_all(cases).await;
        let measured_duration = (Local::now() - start_time).num_milliseconds() as usize;
        let estimated_duration = (PAGE_LOAD_DURATION
            + ((PAGES_NUM * PAGE_NEWS_NUM) as f64 / TASK_PERMITS_NUM as f64).ceil() as usize * NEWS_LOAD_DURATION)
            * (CASES_NUM as f64 / CASE_PERMITS_NUM as f64).ceil() as usize;
        const BUFFER_FACTOR: f64 = 0.005;
        assert!(
            measured_duration >= estimated_duration
                && (measured_duration as f64) < (estimated_duration as f64 * (1.0 + BUFFER_FACTOR))
        );
        Ok(())
    }
}
