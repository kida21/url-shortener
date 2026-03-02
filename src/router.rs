use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::routes;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    
    let api_routes = Router::new()
        .route("/shorten", post(routes::shorten_url))
        .route("/stats/{code}", get(routes::get_stats))
        .route("/urls/{code}", delete(routes::delete_url));

   
    Router::new()
        
        .route("/health", get(routes::health_check))
        
        .nest("/api", api_routes)
        
        .route("/{code}", get(routes::redirect_to_url))
        
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        
        .with_state(state)
}