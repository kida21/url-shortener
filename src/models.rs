use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct UrlRecord {
    pub id: i64,
    pub short_code: String,
    pub original_url: String,
    pub click_count: i64,
    pub created_at: String,
    pub expires_at: Option<String>,
}


#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    pub url: String,
    pub custom_code: Option<String>,
    pub expires_in: Option<u64>, 
}


#[derive(Debug, Serialize)]
pub struct ShortenResponse {
    pub short_code: String,
    pub short_url: String,
    pub original_url: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UrlStats {
    pub short_code: String,
    pub original_url: String,
    pub click_count: i64,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}