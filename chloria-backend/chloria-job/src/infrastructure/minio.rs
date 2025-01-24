use std::{fs, sync::RwLock};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, FixedOffset, Local, TimeDelta};
use minio::s3::{
    args::BucketExistsArgs,
    client::Client as S3Client,
    creds::{Credentials, Provider},
    http::BaseUrl,
};
use reqwest::{Certificate, Client as HttpClient};
use serde::Deserialize;
use tokio::runtime::Handle;

use crate::execution::ports::file_storage::{FileObjectKind, FileStorage, UploadFileInput};

pub(crate) struct MinioClient {
    client: S3Client,
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
        let client = {
            let operator_cacert = match operator_cacert_file {
                Some(cacert_file) => Some(fs::read_to_string(cacert_file)?),
                None => None,
            };
            let web_identity_token = fs::read_to_string(web_identity_token_file)?;
            let provider = WebIdentityProvider::new(operator_sts_endpoint, operator_cacert, web_identity_token);
            S3Client::new(
                tenant_endpoint.parse::<BaseUrl>()?,
                Some(Box::new(provider)),
                None,
                None,
            )?
        };
        Ok(Self {
            client,
            origin_bucket_name,
        })
    }
}

#[async_trait]
impl FileStorage for MinioClient {
    async fn upload_file(&self, input: UploadFileInput) -> Result<()> {
        let bucket_name = match input.kind {
            FileObjectKind::Origin => self.origin_bucket_name.clone(),
        };
        let result = self
            .client
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

#[derive(Debug)]
struct WebIdentityProvider {
    operator_sts_endpoint: String,
    operator_cacert: Option<Certificate>,
    web_identity_token: String,
    creds: RwLock<Option<Credentials>>,
    expiration: RwLock<Option<DateTime<FixedOffset>>>,
}

impl WebIdentityProvider {
    fn new(operator_sts_endpoint: String, operator_cacert: Option<String>, web_identity_token: String) -> Self {
        let operator_cacert = match operator_cacert {
            Some(cacert) => {
                Some(Certificate::from_pem(cacert.as_bytes()).expect("Failed to read the operator's CA cert."))
            }
            None => None,
        };
        Self {
            operator_sts_endpoint,
            operator_cacert,
            web_identity_token,
            creds: RwLock::new(None),
            expiration: RwLock::new(None),
        }
    }

    async fn retrieve_web_identity(&self) -> Result<Credentials> {
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
        let mut expiration = self.expiration.write().unwrap();
        *expiration = Some(DateTime::parse_from_rfc3339(&response.result.credentials.expiration)?);
        return Ok(Credentials {
            access_key: response.result.credentials.access_key_id,
            secret_key: response.result.credentials.secret_access_key,
            session_token: Some(response.result.credentials.session_token),
        });
    }
}

impl Provider for WebIdentityProvider {
    fn fetch(&self) -> Credentials {
        let should_renew = match self.expiration.read().unwrap().clone() {
            Some(expiration) => {
                const EXPIRATION_BUFFER: TimeDelta = Duration::minutes(1);
                expiration - EXPIRATION_BUFFER < Local::now()
            }
            None => true,
        };
        if should_renew {
            // See: https://stackoverflow.com/questions/66035290/how-do-i-await-a-future-inside-a-non-async-method-which-was-called-from-an-async
            let handle = Handle::current();
            let _ = handle.enter();
            let mut creds = self.creds.write().unwrap();
            *creds = Some(
                futures::executor::block_on(self.retrieve_web_identity()).expect("Cannot retrieve the web identity"),
            );
        }
        return self
            .creds
            .read()
            .unwrap()
            .clone()
            .expect("The web identity has not been retrieved yet");
    }
}
