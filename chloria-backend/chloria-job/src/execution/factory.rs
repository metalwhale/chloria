use std::sync::Arc;

use super::ports::{file_storage::FileStorage, news_fetcher::NewsFetcher};

pub(crate) struct Factory<'f> {
    pub(super) news_fetcher: &'f (dyn NewsFetcher + Send + Sync),
    pub(super) file_storage: Arc<dyn FileStorage + Send + Sync>,
}

impl<'f> Factory<'f> {
    pub(crate) fn new(
        news_fetcher: &'f (dyn NewsFetcher + Send + Sync),
        file_storage: Arc<dyn FileStorage + Send + Sync>,
    ) -> Self {
        Self {
            news_fetcher,
            file_storage,
        }
    }
}
