use std::sync::Arc;

use anyhow::Result;
use tokio::{runtime::Handle, sync::Semaphore, task::LocalSet};

use super::{
    cases::LocalCase,
    ports::{file_storage::FileStorage, http_helper::HttpHelper, news_fetcher::NewsFetcher, repository::Repository},
};

pub(crate) struct Config {
    pub(crate) case_permits_num: usize,
    pub(crate) task_permits_num: usize,
}

pub(crate) struct Workshop {
    pub(super) news_fetchers: Vec<Arc<dyn NewsFetcher>>,
    pub(super) http_helper: Arc<dyn HttpHelper>,
    pub(super) file_storage: Arc<dyn FileStorage>,
    pub(super) repository: Arc<dyn Repository>,
    pub(super) config: Config,
    semaphore: Arc<Semaphore>,
}

impl Workshop {
    pub(crate) fn new(
        news_fetchers: Vec<Arc<dyn NewsFetcher>>,
        http_helper: Arc<dyn HttpHelper>,
        file_storage: Arc<dyn FileStorage>,
        repository: Arc<dyn Repository>,
        config: Config,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.case_permits_num));
        Self {
            news_fetchers,
            http_helper,
            file_storage,
            repository,
            config,
            semaphore,
        }
    }

    pub(super) async fn run_local_case<C: LocalCase>(&self, case: C) -> Result<C::Output> {
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
