use jsonwebtoken::{DecodingKey, EncodingKey};

use crate::execution::workshop::Workshop;

#[derive(Clone)]
pub(super) struct RouterState {
    pub(super) jwt: RouterStateJwt,
    pub(super) workshop: Workshop,
}

#[derive(Clone)]
pub(super) struct RouterStateJwt {
    pub(super) decoding_key: DecodingKey,
    pub(super) encoding_key: EncodingKey,
    pub(super) lifetime: u64,
}
