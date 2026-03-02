use crate::config::AppConfig;
use crate::db::DbPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: AppConfig,
}

impl AppState {
    pub fn new(db: DbPool, config: AppConfig) -> Self {
        Self { db, config }
    }
}