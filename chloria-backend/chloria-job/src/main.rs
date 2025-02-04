mod domain;
mod execution;
mod infrastructure;
mod interface;

use std::env;
use std::sync::Arc;

use anyhow::Result;
use env_logger::Env;
use infrastructure::reqwest::ReqwestTool;

use crate::execution::factory::Factory;
use crate::infrastructure::minio::MinioClient;
use crate::infrastructure::newsdata::NewsdataClient;
use crate::interface::commander::Commander;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(Env::new().filter("CHLORIA_LOG"));
    // Read env vars
    let newsdata_api_key = env::var("NEWSDATA_API_KEY")?;
    let newsdata_pages_num_limit = env::var("NEWSDATA_PAGES_NUM_LIMIT")?.parse::<u16>().ok();
    let minio_operator_sts_endpoint = env::var("MINIO_OPERATOR_STS_ENDPOINT")?;
    let minio_operator_cacert_file = env::var("MINIO_OPERATOR_CACERT_FILE").ok();
    let minio_web_identity_token_file = env::var("MINIO_WEB_IDENTITY_TOKEN_FILE")?;
    let minio_tenant_endpoint = env::var("MINIO_TENANT_ENDPOINT")?;
    let chloria_origin_bucket_name = env::var("CHLORIA_ORIGIN_BUCKET_NAME")?;
    // Initialize infrastructure
    let newsdata_client = NewsdataClient::new(newsdata_api_key, newsdata_pages_num_limit);
    let reqwest_tool = ReqwestTool::new();
    let minio_client = MinioClient::new(
        minio_operator_sts_endpoint,
        minio_operator_cacert_file,
        minio_web_identity_token_file,
        minio_tenant_endpoint,
        chloria_origin_bucket_name,
    )?;
    // Initialize execution
    let factory = Factory::new(&newsdata_client, Arc::new(reqwest_tool), Arc::new(minio_client));
    // Initialize interface
    let commander = Commander::new(&factory);
    commander.collect_news().await?;
    Ok(())
}
