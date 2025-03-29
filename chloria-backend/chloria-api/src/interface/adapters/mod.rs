pub(super) mod auth;
pub(super) mod news;

use serde::Serialize;
use serde_json::Value;

#[serde_with::skip_serializing_none]
#[derive(Default, Serialize)]
pub(super) struct ErrorResponse {
    code: Option<String>,
    reason: Option<String>,
    extra: Option<Value>,
}

impl From<String> for ErrorResponse {
    fn from(value: String) -> Self {
        Self {
            reason: Some(value),
            ..Default::default()
        }
    }
}
