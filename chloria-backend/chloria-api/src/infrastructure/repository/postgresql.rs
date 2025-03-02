use anyhow::Result;
use async_trait::async_trait;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

use crate::{execution::ports::repository::Repository, schema::client_credentials::dsl::*};

pub(crate) struct PostgresqlClient {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl PostgresqlClient {
    pub(crate) fn new(database_url: String) -> Result<Self> {
        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(ConnectionManager::<PgConnection>::new(database_url))?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl Repository for PostgresqlClient {
    async fn select_client_api_secret(&self, api_key_input: &str) -> Result<Option<String>> {
        let api_secret_value = client_credentials
            .filter(api_key.eq(api_key_input))
            .select(api_secret)
            .first(&mut self.pool.get()?)
            .optional()?;
        Ok(api_secret_value)
    }
}
