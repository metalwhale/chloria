mod interface;
mod schema;

use std::env;

use anyhow::Result;
use tokio::net::TcpListener;

use crate::interface::router::{self, RouterConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Read env vars
    let chloria_jwt_key = env::var("CHLORIA_JWT_KEY")?;
    let chloria_jwt_lifetime = env::var("CHLORIA_JWT_LIFETIME")?.parse()?; // In seconds
    let chloria_api_port: i32 = env::var("CHLORIA_API_PORT")?.parse()?;
    // Initialize interface
    let router = router::new(RouterConfig {
        jwt_key: chloria_jwt_key,
        jwt_lifetime: chloria_jwt_lifetime,
    });
    let listener = TcpListener::bind(format!("0.0.0.0:{}", chloria_api_port)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
