use std::sync::Arc;

use anyhow::Result;
use tokio::{runtime::Handle, sync::Semaphore, task::LocalSet};

use super::{
    cases::LocalCase,
    ports::{file_storage::FileStorage, http_helper::HttpHelper, news_fetcher::NewsFetcher},
};

pub(crate) struct Config {
    pub(crate) case_permits_num: usize,
    pub(crate) task_permits_num: usize,
}

pub(crate) struct Workshop {
    pub(super) news_fetcher: Arc<dyn NewsFetcher + Send + Sync>,
    pub(super) http_helper: Arc<dyn HttpHelper + Send + Sync>,
    pub(super) file_storage: Arc<dyn FileStorage + Send + Sync>,
    pub(super) config: Config,
    semaphore: Arc<Semaphore>,
}

impl Workshop {
    pub(crate) fn new(
        news_fetcher: Arc<dyn NewsFetcher + Send + Sync>,
        http_helper: Arc<dyn HttpHelper + Send + Sync>,
        file_storage: Arc<dyn FileStorage + Send + Sync>,
        config: Config,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.case_permits_num));
        Self {
            news_fetcher,
            http_helper,
            file_storage,
            config,
            semaphore,
        }
    }

    pub(super) async fn run_local_case<C: LocalCase + Send + Sync + 'static>(&self, case: C) -> Result<C::Output> {
        let semaphore = Arc::clone(&self.semaphore);
        // See: https://github.com/tokio-rs/tokio/issues/2095#issuecomment-573330413
        // TODO: Consider alternative approaches
        // - https://docs.rs/tokio/1.43.0/tokio/task/struct.LocalSet.html#use-inside-tokiospawn
        tokio::task::spawn_blocking(move || {
            Handle::current().block_on(async {
                Ok(LocalSet::new()
                    .run_until(async {
                        let _permit = semaphore.acquire().await?;
                        case.execute().await
                    })
                    .await?)
            })
        })
        .await?
    }
}
