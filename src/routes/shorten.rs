use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use crate::error::AppError;
use crate::models::{ShortenRequest, ShortenResponse, ErrorBody};
use crate::services;
use crate::AppState;


#[utoipa::path(
    post,
    path = "/api/shorten",
    tag = "URLs",
    request_body = ShortenRequest,
    responses(
        (status = 201, description = "Short URL created successfully", body = ShortenResponse),
        (status = 400, description = "Invalid URL format", body = ErrorBody),
        (status = 409, description = "Custom code already exists", body = ErrorBody)
    )
)]
pub async fn shorten_url(
    State(state): State<AppState>,
    Json(payload): Json<ShortenRequest>,
) -> Result<(StatusCode, Json<ShortenResponse>), AppError> {
    tracing::info!("Shortening URL: {}", payload.url);

    let response = services::shorten_url(
        &state.db,
        &state.config,
        payload,
    )
    .await?;

    tracing::info!("Created short URL: {}", response.short_url);

    Ok((StatusCode::CREATED, Json(response)))
}