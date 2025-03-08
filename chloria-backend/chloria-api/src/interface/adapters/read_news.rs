use anyhow::Result;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderName, StatusCode},
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;

use super::{super::state::RouterState, ErrorResponse};
use crate::execution::cases::read_news::ReadNewsCaseInput;

#[derive(Deserialize)]
pub(in super::super) struct ReadNewsRequest {
    date: String,
}

pub(in super::super) async fn read_news(
    State(state): State<RouterState>,
    Query(request): Query<ReadNewsRequest>,
) -> Result<([(HeaderName, &'static str); 2], Body), (StatusCode, Json<ErrorResponse>)> {
    let date = NaiveDate::parse_from_str(&request.date, "%Y-%m-%d")
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(e.to_string().into())))?;
    let output = state
        .workshop
        .execute_read_news_case(ReadNewsCaseInput { date })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string().into())))?;
    let body = Body::from_stream(output.articles_stream);
    let headers = [
        (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
        (header::CONTENT_DISPOSITION, "attachment; filename=\"articles.csv\""),
    ];
    Ok((headers, body))
}
