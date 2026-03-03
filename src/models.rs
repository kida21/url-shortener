use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct UrlRecord {
    pub id: i64,
    pub short_code: String,
    pub original_url: String,
    pub click_count: i64,
    pub created_at: String,
    pub expires_at: Option<String>,
}


#[derive(Debug, Deserialize, ToSchema)]
pub struct ShortenRequest {
    
    #[schema(example = "https://www.rust-lang.org/learn/get-started")]
    pub url: String,

   
    #[schema(example = "my-link")]
    pub custom_code: Option<String>,

  
    #[schema(example = 3600)]
    pub expires_in: Option<u64>,
}


#[derive(Debug, Serialize, ToSchema)]
pub struct ShortenResponse {
    
    #[schema(example = "aB3kX9")]
    pub short_code: String,

    
    #[schema(example = "http://localhost:3000/aB3kX9")]
    pub short_url: String,

    
    #[schema(example = "https://www.rust-lang.org/learn/get-started")]
    pub original_url: String,

   
    #[schema(example = "2025-01-15 10:30:00")]
    pub created_at: String,

    
    #[schema(example = "2025-01-15 11:30:00")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UrlStats {
    
    #[schema(example = "aB3kX9")]
    pub short_code: String,

   
    #[schema(example = "https://www.rust-lang.org/learn/get-started")]
    pub original_url: String,

   
    #[schema(example = 42)]
    pub click_count: i64,

    
    #[schema(example = "2025-01-15 10:30:00")]
    pub created_at: String,

    
    #[schema(example = "2025-01-15 11:30:00")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    
    #[schema(example = "healthy")]
    pub status: String,

    
    #[schema(example = "0.1.0")]
    pub version: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
   
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
   
    #[schema(example = "INVALID_URL")]
    pub code: String,

   
    #[schema(example = "URL format is invalid")]
    pub message: String,
}