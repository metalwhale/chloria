mod domain;
mod execution;
mod infrastructure;
mod interface;

use std::env;
use std::sync::Arc;

use anyhow::Result;
use env_logger::Env;

use crate::execution::workshop::{Config, Workshop};
use crate::infrastructure::minio::MinioClient;
use crate::infrastructure::newsdata::NewsdataClient;
use crate::infrastructure::reqwest::ReqwestTool;
use crate::interface::commander::Commander;

#[tokio::main]
async fn main() -> Result<()> {
    // Read env vars
    let newsdata_api_key = env::var("NEWSDATA_API_KEY")?;
    let newsdata_pages_num_limit = env::var("NEWSDATA_PAGES_NUM_LIMIT")?.parse().ok();
    let minio_operator_sts_endpoint = env::var("MINIO_OPERATOR_STS_ENDPOINT")?;
    let minio_operator_cacert_file = env::var("MINIO_OPERATOR_CACERT_FILE").ok();
    let minio_tenant_endpoint = env::var("MINIO_TENANT_ENDPOINT")?;
    let minio_web_identity_token_file = env::var("MINIO_WEB_IDENTITY_TOKEN_FILE")?;
    let chloria_origin_bucket_name = env::var("CHLORIA_ORIGIN_BUCKET_NAME")?;
    let chloria_case_permits_num = env::var("CHLORIA_CASE_PERMITS_NUM")?.parse().unwrap_or(10);
    let chloria_task_permits_num = env::var("CHLORIA_TASK_PERMITS_NUM")?.parse().unwrap_or(10);
    env_logger::init_from_env(Env::new().filter("CHLORIA_LOG_LEVEL"));
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
    let workshop = Workshop::new(
        Arc::new(newsdata_client),
        Arc::new(reqwest_tool),
        Arc::new(minio_client),
        Config {
            case_permits_num: chloria_case_permits_num,
            task_permits_num: chloria_task_permits_num,
        },
    );
    // Initialize interface
    let commander = Commander::new(&workshop);
    commander.collect_news().await?;
    Ok(())
}
