use std::sync::Arc;

use super::ports::{file_storage::FileStorage, http_helper::HttpHelper, news_fetcher::NewsFetcher};

pub(crate) struct Factory<'f> {
    pub(super) news_fetcher: &'f (dyn NewsFetcher + Send + Sync),
    pub(super) http_helper: Arc<dyn HttpHelper + Send + Sync>,
    pub(super) file_storage: Arc<dyn FileStorage + Send + Sync>,
}

impl<'f> Factory<'f> {
    pub(crate) fn new(
        news_fetcher: &'f (dyn NewsFetcher + Send + Sync),
        http_helper: Arc<dyn HttpHelper + Send + Sync>,
        file_storage: Arc<dyn FileStorage + Send + Sync>,
    ) -> Self {
        Self {
            news_fetcher,
            http_helper,
            file_storage,
        }
    }
}
