use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::error::AppError;
use crate::models::UrlStats;
use crate::services;
use crate::AppState;

pub async fn get_stats(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<UrlStats>, AppError> {
    tracing::info!("Getting stats for: {}", code);

    let stats = services::get_stats(&state.db, &code).await?;

    Ok(Json(stats))
}

pub async fn delete_url(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<StatusCode, AppError> {
    tracing::info!("Deleting short URL: {}", code);

    services::delete_short_url(&state.db, &code).await?;

    tracing::info!("Deleted short URL: {}", code);

    Ok(StatusCode::NO_CONTENT)
}