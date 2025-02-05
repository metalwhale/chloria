use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

use crate::execution::ports::http_helper::HttpHelper;

pub(crate) struct ReqwestTool {}

impl ReqwestTool {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl HttpHelper for ReqwestTool {
    async fn get(&self, url: &str) -> Result<Vec<u8>> {
        Ok(Client::new().get(url).send().await?.bytes().await?.into())
    }
}
