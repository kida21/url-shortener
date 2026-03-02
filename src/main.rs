mod config;
mod db;
mod error;
mod models;
mod services;

#[tokio::main]
async fn main() {
    
    tracing_subscriber::fmt::init();

    
    dotenvy::dotenv().ok();

    
    let config = config::AppConfig::from_env();
    tracing::info!("Configuration loaded: {:?}", config);

    
    let pool = db::init_db(&config.database_url)
        .await
        .expect("Failed to initialize database");

    tracing::info!("Database ready");
    tracing::info!("Server will start at {}", config.server_addr());

    
    println!("complete! All layers compiled successfully.");
    println!("will add API routes and wire everything together.");
}