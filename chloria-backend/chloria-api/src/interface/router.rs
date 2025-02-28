use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use tower::ServiceBuilder;

use super::{
    adapters::auth::{authenticate, authorize},
    state::{RouterState, RouterStateJwt},
};

pub(crate) struct RouterConfig {
    pub(crate) jwt_key: String,
    pub(crate) jwt_lifetime: u64,
}

pub(crate) fn new(config: RouterConfig) -> Router {
    let state = RouterState {
        jwt: RouterStateJwt {
            decoding_key: DecodingKey::from_secret(config.jwt_key.as_bytes()),
            encoding_key: EncodingKey::from_secret(config.jwt_key.as_bytes()),
            lifetime: config.jwt_lifetime,
        },
    };
    let public_router = Router::new()
        .route("/authenticate", post(authenticate))
        .with_state(state.clone());
    let authorized_router = Router::new()
        .route("/news", get(read_news))
        .route_layer(ServiceBuilder::new().layer(middleware::from_fn_with_state(state, authorize)));
    let router = Router::new().merge(public_router).merge(authorized_router);
    router
}

async fn read_news() {}
