use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use tower::ServiceBuilder;

use super::{
    adapters::{
        auth::{authenticate, authorize},
        news::{create_news_insight, read_news},
    },
    state::{RouterState, RouterStateJwt},
};
use crate::execution::workshop::Workshop;

pub(crate) struct RouterConfig {
    pub(crate) jwt_key: String,
    pub(crate) jwt_lifetime: u64,
}

pub(crate) fn new(config: RouterConfig, workshop: Workshop) -> Router {
    let state = RouterState {
        jwt: RouterStateJwt {
            decoding_key: DecodingKey::from_secret(config.jwt_key.as_bytes()),
            encoding_key: EncodingKey::from_secret(config.jwt_key.as_bytes()),
            lifetime: config.jwt_lifetime,
        },
        workshop,
    };
    let public_router = Router::new()
        .route("/authenticate", post(authenticate))
        .with_state(state.clone());
    let authorized_router = Router::new()
        .route("/news", get(read_news))
        .route("/news_insight", post(create_news_insight))
        .route_layer(ServiceBuilder::new().layer(middleware::from_fn_with_state(state.clone(), authorize)))
        .with_state(state.clone());
    let router = Router::new().merge(public_router).merge(authorized_router);
    router
}
