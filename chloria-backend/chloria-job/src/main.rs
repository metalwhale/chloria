mod execution;
mod infrastructure;
mod interface;

use std::env;

use anyhow::Result;

use crate::infrastructure::minio::MinioClient;
use crate::interface::commander::Commander;
use crate::interface::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let minio_operator_sts_endpoint = env::var("MINIO_OPERATOR_STS_ENDPOINT")?;
    let minio_operator_cacert_file = env::var("MINIO_OPERATOR_CACERT_FILE").ok();
    let minio_tenant_endpoint = env::var("MINIO_TENANT_ENDPOINT")?;
    let minio_web_identity_token_file = env::var("MINIO_WEB_IDENTITY_TOKEN_FILE")?;
    let chloria_origin_bucket_name = env::var("CHLORIA_ORIGIN_BUCKET_NAME")?;
    let minio_client = MinioClient::new(
        minio_operator_sts_endpoint,
        minio_operator_cacert_file,
        minio_web_identity_token_file,
        minio_tenant_endpoint,
        chloria_origin_bucket_name,
    )?;
    let config = Config::new(&minio_client);
    let commander = Commander::new(config);
    commander.collect_news().await?;
    Ok(())
}
