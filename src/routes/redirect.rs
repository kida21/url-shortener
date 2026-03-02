use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};

use crate::error::AppError;
use crate::services;
use crate::AppState;

pub async fn redirect_to_url(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Resolving short code: {}", code);

    let original_url = services::resolve_url(&state.db, &code).await?;

    tracing::info!("Redirecting {} -> {}", code, original_url);

    Ok((
        StatusCode::TEMPORARY_REDIRECT,
        [(header::LOCATION, original_url)],
    ))
}