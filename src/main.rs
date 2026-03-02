mod config;
mod db;
mod error;
mod models;
mod router;
mod routes;
mod services;
mod state;

pub use state::AppState;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Initialize tracing (logging)
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    tracing::info!("🚀 Starting URL Shortener...");

    // Load configuration
    let config = config::AppConfig::from_env();
    tracing::info!("Configuration loaded");
    tracing::info!("Base URL: {}", config.base_url);

    
    let pool = db::init_db(&config.database_url)
        .await
        .expect("Failed to initialize database");
    tracing::info!("Database initialized");

    
    let state = AppState::new(pool, config.clone());

  
    let app = router::create_router(state);

    
    let addr = config.server_addr();
    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("─────────────────────────────────────");
    tracing::info!("  Endpoints:");
    tracing::info!("  POST   /api/shorten       → Create short URL");
    tracing::info!("  GET    /{{code}}             → Redirect");
    tracing::info!("  GET    /api/stats/{{code}}   → URL statistics");
    tracing::info!("  DELETE /api/urls/{{code}}    → Delete short URL");
    tracing::info!("  GET    /health             → Health check");
    tracing::info!("─────────────────────────────────────");

    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}