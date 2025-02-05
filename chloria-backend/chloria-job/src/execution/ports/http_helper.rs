use std::{future::Future, pin::Pin};

use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

#[async_trait]
pub(crate) trait HttpHelper {
    async fn get(&self, url: &str) -> Result<Vec<u8>>;
}

mock! {
    pub(in super::super) HttpHelper {}

    impl HttpHelper for HttpHelper {
        fn get<'life0, 'life1, 'async_trait>(
            &'life0 self,
            url: &'life1 str,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'async_trait>>
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait;
    }
}
