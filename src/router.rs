use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::middleware::RateLimiter;
use crate::routes;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    
    let rate_limiter = RateLimiter::new(100, 60);
    rate_limiter.start_cleanup_task();

   
    let api_routes = Router::new()
        .route("/shorten", post(routes::shorten_url))
        .route("/stats/{code}", get(routes::get_stats))
        .route("/urls/{code}", delete(routes::delete_url));

   
    Router::new()
        .route("/health", get(routes::health_check))
        .nest("/api", api_routes)
        .route("/{code}", get(routes::redirect_to_url))
        // Middleware layers
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(axum::Extension(rate_limiter))
        
        .with_state(state)
}