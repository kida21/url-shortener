use tokio::net::TcpListener;
use tokio::signal;

use url_shortener::{
    config::AppConfig,
    db,
    router::create_router,
    AppState,
};

#[tokio::main]
async fn main() {
   
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
        .init();

    
    dotenvy::dotenv().ok();

    tracing::info!("🚀 Starting URL Shortener...");

   
    let config = AppConfig::from_env();
    tracing::info!("Base URL: {}", config.base_url);

    
    let pool = db::init_db(&config.database_url)
        .await
        .expect("Failed to initialize database");
    tracing::info!("Database initialized");

    
    let state = AppState::new(pool, config.clone());
    let app = create_router(state);
    let addr = config.server_addr();

    tracing::info!("─────────────────────────────────────────");
    tracing::info!("  🌐 Server listening on http://{}", addr);
    tracing::info!("─────────────────────────────────────────");
    tracing::info!("  POST   /api/shorten        → Create short URL");
    tracing::info!("  GET    /{{code}}              → Redirect");
    tracing::info!("  GET    /api/stats/{{code}}    → URL statistics");
    tracing::info!("  DELETE /api/urls/{{code}}     → Delete short URL");
    tracing::info!("  GET    /health              → Health check");
    tracing::info!("─────────────────────────────────────────");

    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    // Graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server failed");

    tracing::info!("👋 Server shut down gracefully");
}


async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down...");
        }
    }
}