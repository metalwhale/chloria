use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Semaphore;

use super::{
    super::{
        ports::{file_storage::FileStorage, http_helper::HttpHelper, news_fetcher::NewsFetcher},
        tasks::{save_news::SaveNewsTask, LocalTask},
        workshop::Workshop,
    },
    LocalCase,
};

type CollectNewsCaseOutput = i32;

struct CollectNewsCase {
    news_fetcher: Arc<dyn NewsFetcher + Send + Sync>,
    http_helper: Arc<dyn HttpHelper + Send + Sync>,
    file_storage: Arc<dyn FileStorage + Send + Sync>,
    task_permits_num: usize,
}

impl Workshop {
    pub(crate) async fn execute_collect_news_case(&self) -> Result<CollectNewsCaseOutput> {
        let case = CollectNewsCase {
            news_fetcher: Arc::clone(&self.news_fetcher),
            http_helper: Arc::clone(&self.http_helper),
            file_storage: Arc::clone(&self.file_storage),
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
        for handle in self
            .news_fetcher
            .fetch_news(Box::new(move |a| {
                let task = SaveNewsTask::new(a, Arc::clone(&self.http_helper), Arc::clone(&self.file_storage));
                let semaphore = Arc::clone(&semaphore);
                tokio::task::spawn_local(async move {
                    let _permit = semaphore.acquire().await?;
                    let is_effective = task.perform().await?;
                    Ok(is_effective)
                })
            }))
            .await
        {
            if let Ok(Ok(is_effective)) = handle.await {
                if is_effective {
                    count += 1;
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
        let workshop = Workshop::new(
            Arc::new(mock_news_fetcher),
            Arc::new(mock_http_helper),
            Arc::new(mock_file_storage),
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
