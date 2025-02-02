use std::sync::Arc;

use super::ports::{file_storage::FileStorage, news_fetcher::NewsFetcher};

pub(crate) struct Workshop<'w> {
    pub(super) news_fetcher: &'w (dyn NewsFetcher + Send + Sync),
    pub(super) file_storage: Arc<dyn FileStorage + Send + Sync>,
}

impl<'w> Workshop<'w> {
    pub(crate) fn new(
        news_fetcher: &'w (dyn NewsFetcher + Send + Sync),
        file_storage: Arc<dyn FileStorage + Send + Sync>,
    ) -> Self {
        Self {
            news_fetcher,
            file_storage,
        }
    }
}
