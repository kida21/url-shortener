use utoipa::OpenApi;

use crate::models::{
    ErrorBody, ErrorDetail, HealthResponse,
    ShortenRequest, ShortenResponse, UrlStats,
};
use crate::routes;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "URL Shortener API",
        version = "0.1.0",
        description = "A fast, lightweight URL shortener built with Rust, Axum, and SQLite.",
        contact(
            name = "API Support",
            url = "https://github.com/yourusername/url-shortener"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development"),
    ),
    paths(
        routes::shorten::shorten_url,
        routes::redirect::redirect_to_url,
        routes::stats::get_stats,
        routes::stats::delete_url,
        routes::health::health_check,
    ),
    components(
        schemas(
            ShortenRequest,
            ShortenResponse,
            UrlStats,
            HealthResponse,
            ErrorBody,
            ErrorDetail,
        )
    ),
    tags(
        (name = "URLs", description = "URL shortening and redirection"),
        (name = "Statistics", description = "URL click statistics"),
        (name = "System", description = "System health and info")
    )
)]
pub struct ApiDoc;