use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json, RequestExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, Header, Validation};
use serde::{Deserialize, Serialize};

use super::{super::state::RouterState, ErrorResponse};
use crate::execution::cases::authenticate::{AuthenticateCaseInput, AuthenticateCaseOutput};

#[derive(Deserialize)]
pub(in super::super) struct AuthenticateRequest {
    api_key: String,
    api_secret: String,
}

#[derive(Serialize)]
pub(in super::super) struct AuthenticateResponse {
    token: String,
}

#[derive(Deserialize, Serialize)]
struct Claim {
    exp: u64,
}

pub(in super::super) async fn authenticate(
    State(state): State<RouterState>,
    Json(request): Json<AuthenticateRequest>,
) -> Result<Json<AuthenticateResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .workshop
        .execute_authenticate_case(AuthenticateCaseInput {
            api_key: request.api_key,
            api_secret: request.api_secret,
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string().into())))?
    {
        AuthenticateCaseOutput::Success => {}
        output => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    code: Some(output.to_string()),
                    ..Default::default()
                }),
            ));
        }
    }
    let claim = Claim {
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string().into())))?
            .as_secs()
            + state.jwt.lifetime,
    };
    let token = encode(&Header::default(), &claim, &state.jwt.encoding_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string().into())))?;
    Ok(Json(AuthenticateResponse { token }))
}

pub(in super::super) async fn authorize(
    State(state): State<RouterState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let TypedHeader(Authorization(bearer)) = request
        .extract_parts::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|r| (StatusCode::BAD_REQUEST, Json(r.to_string().into())))?;
    decode::<Claim>(bearer.token(), &state.jwt.decoding_key, &Validation::default()).map_err(|error| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                code: Some(error.to_string()),
                ..Default::default()
            }),
        )
    })?;
    let response = next.run(request).await;
    Ok(response)
}
