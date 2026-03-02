use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use crate::error::AppError;
use crate::models::{ShortenRequest, ShortenResponse};
use crate::services;
use crate::AppState;

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