use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::execution::ports::hashing_algorithm::HashingAlgorithm;

#[derive(Clone)]
pub(crate) struct Argon2Tool {}

impl Argon2Tool {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl HashingAlgorithm for Argon2Tool {
    fn verify(&self, secret: &str, hashed_secret: &str) -> Result<bool> {
        Ok(Argon2::default()
            .verify_password(secret.as_bytes(), &PasswordHash::new(&hashed_secret)?)
            .is_ok())
    }
}
