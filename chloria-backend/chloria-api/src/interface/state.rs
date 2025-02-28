use jsonwebtoken::{DecodingKey, EncodingKey};

#[derive(Clone)]
pub(super) struct RouterState {
    pub(super) jwt: RouterStateJwt,
}

#[derive(Clone)]
pub(super) struct RouterStateJwt {
    pub(super) decoding_key: DecodingKey,
    pub(super) encoding_key: EncodingKey,
    pub(super) lifetime: u64,
}
