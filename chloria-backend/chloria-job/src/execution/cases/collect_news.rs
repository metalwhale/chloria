use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::{error, info};
use tokio::sync::Semaphore;

use super::{
    super::{
        ports::{
            file_storage::FileStorage, http_helper::HttpHelper, news_fetcher::NewsFetcher, repository::Repository,
        },
        tasks::{
            save_news::{SaveNewsInput, SaveNewsTask},
            LocalTask,
        },
        workshop::Workshop,
    },
    LocalCase,
};
use crate::domain::news::NewsEntity;

type CollectNewsCaseOutput = i32;

struct CollectNewsCase {
    news_fetchers: Vec<Arc<dyn NewsFetcher>>,
    http_helper: Arc<dyn HttpHelper>,
    file_storage: Arc<dyn FileStorage>,
    repository: Arc<dyn Repository>,
    task_permits_num: usize,
}

impl Workshop {
    pub(crate) async fn execute_collect_news_case(&self) -> Result<CollectNewsCaseOutput> {
        let case = CollectNewsCase {
            news_fetchers: self.news_fetchers.iter().map(|f| Arc::clone(f)).collect(),
            http_helper: Arc::clone(&self.http_helper),
            file_storage: Arc::clone(&self.file_storage),
            repository: Arc::clone(&self.repository),
            task_permits_num: self.config.task_permits_num,
        };
        self.run_local_case(case).await
    }
}

#[async_trait(?Send)]
impl LocalCase for CollectNewsCase {
    type Output = CollectNewsCaseOutput;

    async fn execute(self) -> Result<Self::Output> {
        let semaphore = Arc::new(Semaphore::new(self.task_permits_num));
        let mut count = 0;
        let mut outputs = vec![];
        for news_fetcher in self.news_fetchers {
            let http_helper = Arc::clone(&self.http_helper);
            let file_storage = Arc::clone(&self.file_storage);
            let repository = Arc::clone(&self.repository);
            let semaphore = Arc::clone(&semaphore);
            for (article_id, handle) in news_fetcher
                .fetch_news(Box::new(move |article| {
                    let news = NewsEntity::new(article.id);
                    let task = SaveNewsTask::new(
                        SaveNewsInput {
                            source_name: article.source_name,
                            article_id: news.article_id.clone(),
                            link: article.link,
                            title: article.title,
                            short_text: article.short_text,
                            long_text: article.long_text,
                            image_url: article.image_url,
                            published_time: article.published_time,
                        },
                        Arc::clone(&http_helper),
                        Arc::clone(&file_storage),
                        Arc::clone(&repository),
                    );
                    let semaphore = Arc::clone(&semaphore);
                    (
                        news.article_id,
                        tokio::task::spawn_local(async move {
                            let _permit = semaphore.acquire().await?;
                            let news_id = task.perform().await?;
                            Ok(news_id)
                        }),
                    )
                }))
                .await
            {
                outputs.push((article_id, handle));
            }
        }
        for (article_id, handle) in outputs {
            if let Ok(result) = handle.await {
                match result {
                    Ok(news_id) => {
                        if let Some(news_id) = news_id {
                            info!("news_id={}, article_id={}", news_id, article_id);
                            count += 1;
                        }
                    }
                    Err(error) => error!("article_id={}, error={}", article_id, error),
                }
            }
        }
        Ok(count)
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
        const TASK_PERMITS_NUM: usize = 5;
        const CASES_NUM: usize = 3;
        const PAGES_NUM: usize = 2;
        const PAGE_LOAD_DURATION: usize = 1000;
        const PAGE_NEWS_NUM: usize = 4;
        const NEWS_LOAD_DURATION: usize = 2000;
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
            .times(CASES_NUM * PAGES_NUM * PAGE_NEWS_NUM)
            .returning(|_| Ok(Some(0)));
        let workshop = Workshop::new(
            vec![Arc::new(mock_news_fetcher)],
            Arc::new(mock_http_helper),
            Arc::new(mock_file_storage),
            Arc::new(mock_repository),
            Config {
                case_permits_num: CASE_PERMITS_NUM,
                task_permits_num: TASK_PERMITS_NUM,
            },
        );
        let start_time = Local::now();
        let mut cases = vec![];
        for _ in 0..CASES_NUM {
            cases.push(workshop.execute_collect_news_case());
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
