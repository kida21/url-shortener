use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::error::AppError;
use crate::models::{ErrorBody, UrlStats};
use crate::services;
use crate::AppState;


#[utoipa::path(
    get,
    path = "/api/stats/{code}",
    tag = "Statistics",
    params(
        ("code" = String, Path, description = "The short code to get stats for")
    ),
    responses(
        (status = 200, description = "URL statistics", body = UrlStats),
        (status = 404, description = "Short code not found", body = ErrorBody)
    )
)]
pub async fn get_stats(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<UrlStats>, AppError> {
    tracing::info!("Getting stats for: {}", code);

    let stats = services::get_stats(&state.db, &code).await?;

    Ok(Json(stats))
}



#[utoipa::path(
    delete,
    path = "/api/urls/{code}",
    tag = "URLs",
    params(
        ("code" = String, Path, description = "The short code to delete")
    ),
    responses(
        (status = 204, description = "URL deleted successfully"),
        (status = 404, description = "Short code not found", body = ErrorBody)
    )
)]
pub async fn delete_url(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<StatusCode, AppError> {
    tracing::info!("Deleting short URL: {}", code);

    services::delete_short_url(&state.db, &code).await?;

    tracing::info!("Deleted short URL: {}", code);

    Ok(StatusCode::NO_CONTENT)
}