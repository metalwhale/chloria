mod execution;
mod infrastructure;
mod interface;
mod schema;

use std::env;
use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;

use crate::execution::workshop::{Config, Workshop};
use crate::infrastructure::{hashing_algorithm::argon2::Argon2Tool, repository::postgresql::PostgresqlClient};
use crate::interface::router::{self, RouterConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Read env vars
    let database_url = env::var("DATABASE_URL")?;
    let chloria_jwt_key = env::var("CHLORIA_JWT_KEY")?;
    let chloria_jwt_lifetime = env::var("CHLORIA_JWT_LIFETIME")?.parse()?; // In seconds
    let chloria_api_port: i32 = env::var("CHLORIA_API_PORT")?.parse()?;
    let chloria_case_permits_num = env::var("CHLORIA_CASE_PERMITS_NUM")?.parse().unwrap_or(10);
    // Initialize infrastructure
    let postgresql_client = PostgresqlClient::new(database_url)?;
    let argon2_tool = Argon2Tool::new();
    // Initialize execution
    let workshop = Workshop::new(
        Arc::new(postgresql_client),
        Box::new(argon2_tool),
        Config {
            case_permits_num: chloria_case_permits_num,
        },
    );
    // Initialize interface
    let router = router::new(
        RouterConfig {
            jwt_key: chloria_jwt_key,
            jwt_lifetime: chloria_jwt_lifetime,
        },
        workshop,
    );
    let listener = TcpListener::bind(format!("0.0.0.0:{}", chloria_api_port)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
