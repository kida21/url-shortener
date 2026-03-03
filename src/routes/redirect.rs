use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};

use crate::error::AppError;
use crate::models::ErrorBody;
use crate::services;
use crate::AppState;


#[utoipa::path(
    get,
    path = "/{code}",
    tag = "URLs",
    params(
        ("code" = String, Path, description = "The short code to redirect")
    ),
    responses(
        (status = 307, description = "Redirect to original URL",
            headers(
                ("Location" = String, description = "Original URL")
            )
        ),
        (status = 404, description = "Short code not found", body = ErrorBody),
        (status = 410, description = "Short URL has expired", body = ErrorBody)
    )
)]
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