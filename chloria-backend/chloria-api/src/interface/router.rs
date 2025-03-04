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
use crate::execution::workshop::Workshop;

pub(crate) struct RouterConfig {
    pub(crate) jwt_key: String,
}

pub(crate) fn new(config: RouterConfig, workshop: Workshop) -> Router {
    let state = RouterState {
        jwt: RouterStateJwt {
            decoding_key: DecodingKey::from_secret(config.jwt_key.as_bytes()),
            encoding_key: EncodingKey::from_secret(config.jwt_key.as_bytes()),
        },
        workshop,
    };
    let public_router = Router::new()
        .route("/authenticate", post(authenticate))
        .with_state(state.clone());
    let authorized_router = Router::new()
        .route("/get-news", get(get_news))
        .route_layer(ServiceBuilder::new().layer(middleware::from_fn_with_state(state, authorize)));
    let router = Router::new().merge(public_router).merge(authorized_router);
    router
}

async fn get_news() {}
