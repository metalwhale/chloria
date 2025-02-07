use std::sync::Arc;

use anyhow::{Error, Result};
use tokio::task::LocalSet;

use super::super::{
    tasks::{save_news::SaveNewsTask, Task},
    workshop::Workshop,
};

pub(crate) struct CollectNewsCase<'c> {
    workshop: &'c Workshop<'c>,
}

impl<'w> Workshop<'w> {
    pub(crate) fn new_collect_news_case(&self) -> CollectNewsCase {
        CollectNewsCase { workshop: &self }
    }
}

impl<'c> CollectNewsCase<'c> {
    pub(crate) async fn execute(&self) -> Result<i32> {
        let http_helper = Arc::clone(&self.workshop.http_helper);
        let file_storage = Arc::clone(&self.workshop.file_storage);
        let local = LocalSet::new();
        let count = local
            .run_until(async {
                let mut count = 0;
                for handle in self
                    .workshop
                    .news_fetcher
                    .fetch_news(Box::new(move |a| {
                        let task = SaveNewsTask::new(a, Arc::clone(&http_helper), Arc::clone(&file_storage));
                        tokio::task::spawn_local(async {
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
                Ok::<i32, Error>(count)
            })
            .await?;
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
        workshop::Workshop,
    };

    #[tokio::test]
    async fn check_required_duration() -> Result<()> {
        const PAGES_NUM: u64 = 3;
        const PAGE_LOAD_DURATION: u64 = 1000;
        const PAGE_NEWS_NUM: u64 = 20; // Should not affect the required time
        const NEWS_LOAD_DURATION: u64 = 2000;
        async fn fetch_news(handler: FetchNewsHandler) -> Vec<FetchNewsOutput> {
            let mut outputs = vec![];
            for _ in 0..PAGES_NUM {
                time::sleep(Duration::from_millis(PAGE_LOAD_DURATION)).await;
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
            time::sleep(Duration::from_millis(NEWS_LOAD_DURATION)).await;
            Ok(vec![])
        }
        let mut mock_news_fetcher = MockNewsFetcher::new();
        mock_news_fetcher
            .expect_fetch_news()
            .returning(|h| Box::pin(fetch_news(h)));
        let mut mock_http_helper = MockHttpHelper::new();
        mock_http_helper
            .expect_get()
            .times((PAGES_NUM * PAGE_NEWS_NUM) as usize)
            .returning(|_| Box::pin(get()));
        let mut mock_file_storage = MockFileStorage::new();
        mock_file_storage
            .expect_upload_file()
            .times((PAGES_NUM * PAGE_NEWS_NUM) as usize)
            .returning(|_| Box::pin(async { Ok("".to_string()) }));
        let workshop = Workshop::new(
            &mock_news_fetcher,
            Arc::new(mock_http_helper),
            Arc::new(mock_file_storage),
        );
        let case = workshop.new_collect_news_case();
        let start_time = Local::now();
        case.execute().await?;
        let required_duration = (Local::now() - start_time).num_milliseconds() as u64;
        const BUFFER_DURATION: u64 = 50;
        assert!(required_duration < PAGES_NUM * PAGE_LOAD_DURATION + NEWS_LOAD_DURATION + BUFFER_DURATION);
        Ok(())
    }
}
