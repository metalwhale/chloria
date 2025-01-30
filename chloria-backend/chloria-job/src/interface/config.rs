use crate::execution::ports::{file_storage::FileStorage, news_fetcher::NewsFetcher};

pub(crate) struct Config<'c> {
    pub(crate) news_fetcher: &'c dyn NewsFetcher,
    pub(crate) file_storage: &'c dyn FileStorage,
}

impl<'c> Config<'c> {
    pub(crate) fn new(news_fetcher: &'c dyn NewsFetcher, file_storage: &'c dyn FileStorage) -> Self {
        Self {
            news_fetcher,
            file_storage,
        }
    }
}
