use std::fs;

use anyhow::{bail, Result};
use async_trait::async_trait;
use chrono::{DateTime, Duration, FixedOffset, Local, TimeDelta};
use minio::s3::{
    args::BucketExistsArgs,
    client::Client as S3Client,
    creds::{Credentials, StaticProvider},
    http::BaseUrl,
};
use reqwest::{Certificate, Client as HttpClient};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::execution::ports::file_storage::{FileObjectKind, FileStorage, UploadFileInput};

// Doc: https://min.io/docs/minio/linux/developers/security-token-service/AssumeRoleWithWebIdentity.html#response-elements
#[derive(Deserialize)]
struct AssumeRoleWithWebIdentityResponse {
    #[serde(rename = "AssumeRoleWithWebIdentityResult")]
    result: AssumeRoleWithWebIdentityResult,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AssumeRoleWithWebIdentityResult {
    credentials: AssumeRoleWithWebIdentityResultCredentials,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AssumeRoleWithWebIdentityResultCredentials {
    access_key_id: String,
    secret_access_key: String,
    expiration: String,
    session_token: String,
}

pub(crate) struct MinioClient {
    operator_sts_endpoint: String,
    operator_cacert: Option<Certificate>,
    web_identity_token: String,
    tenant_endpoint: String,
    expiration: RwLock<Option<DateTime<FixedOffset>>>,
    client: RwLock<Option<S3Client>>,
    origin_bucket_name: String,
}

impl MinioClient {
    pub(crate) fn new(
        operator_sts_endpoint: String,
        operator_cacert_file: Option<String>,
        web_identity_token_file: String,
        tenant_endpoint: String,
        origin_bucket_name: String,
    ) -> Result<Self> {
        let operator_cacert = match operator_cacert_file {
            Some(cacert_file) => fs::read_to_string(cacert_file).ok(),
            None => None,
        };
        let operator_cacert = match operator_cacert {
            Some(cacert) => Certificate::from_pem(cacert.as_bytes()).ok(),
            None => None,
        };
        let web_identity_token = fs::read_to_string(web_identity_token_file)?;
        Ok(Self {
            operator_sts_endpoint,
            operator_cacert,
            web_identity_token,
            tenant_endpoint,
            expiration: RwLock::new(None),
            client: RwLock::new(None),
            origin_bucket_name,
        })
    }

    async fn reload(&self) -> Result<()> {
        if !self.is_expired().await {
            return Ok(());
        }
        let mut client = self.client.write().await;
        // Check the expiration again, just in case another thread has updated it recently while we were waiting for the lock
        if !self.is_expired().await {
            return Ok(());
        }
        let credentials = self.assume_role().await?;
        let provider = StaticProvider::new(
            &credentials.access_key,
            &credentials.secret_key,
            credentials.session_token.as_deref(),
        );
        *client = Some(S3Client::new(
            self.tenant_endpoint.parse::<BaseUrl>()?,
            Some(Box::new(provider)),
            None,
            None,
        )?);
        Ok(())
    }

    async fn is_expired(&self) -> bool {
        let is_expired = match *self.expiration.read().await {
            Some(expiration) => {
                const EXPIRATION_BUFFER: TimeDelta = Duration::minutes(1);
                expiration - EXPIRATION_BUFFER < Local::now()
            }
            None => true,
        };
        is_expired
    }

    async fn assume_role(&self) -> Result<Credentials> {
        let client = match self.operator_cacert.clone() {
            Some(operator_cacert) => HttpClient::builder().add_root_certificate(operator_cacert).build()?,
            None => HttpClient::new(),
        };
        let response_text = client
            .post(&self.operator_sts_endpoint)
            // Doc: https://min.io/docs/minio/linux/developers/security-token-service/AssumeRoleWithWebIdentity.html#request-endpoint
            .form(&[
                ("Action", "AssumeRoleWithWebIdentity"),
                ("WebIdentityToken", &self.web_identity_token),
                ("Version", "2011-06-15"),
            ])
            .send()
            .await?
            .text()
            .await?;
        let response: AssumeRoleWithWebIdentityResponse = serde_xml_rs::from_str(&response_text)?;
        let mut expiration = self.expiration.write().await;
        *expiration = Some(DateTime::parse_from_rfc3339(&response.result.credentials.expiration)?);
        Ok(Credentials {
            access_key: response.result.credentials.access_key_id,
            secret_key: response.result.credentials.secret_access_key,
            session_token: Some(response.result.credentials.session_token),
        })
    }
}

#[async_trait]
impl FileStorage for MinioClient {
    async fn upload_file(&self, input: UploadFileInput) -> Result<()> {
        self.reload().await?;
        let client = self.client.read().await;
        let client = match &*client {
            Some(client) => client,
            None => bail!("The client has not been initialized."),
        };
        let bucket_name = match input.kind {
            FileObjectKind::Origin => &self.origin_bucket_name,
        };
        let result = client
            .bucket_exists(&BucketExistsArgs {
                bucket: &bucket_name,
                extra_headers: None,
                extra_query_params: None,
                region: None,
            })
            .await?;
        println!("{}", result);
        Ok(())
    }
}
