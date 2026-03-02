#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://urls.db".to_string()),
            host: std::env::var("HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a number"),
            base_url: std::env::var("BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        }
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}