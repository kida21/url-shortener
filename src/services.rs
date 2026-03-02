use chrono::{Duration, Utc};
use url::Url;

use crate::config::AppConfig;
use crate::db::{self, DbPool};
use crate::error::AppError;
use crate::models::{ShortenRequest, ShortenResponse, UrlStats};

const SHORT_CODE_LENGTH: usize = 6;
const SHORT_CODE_ALPHABET: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
    'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E',
    'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q',
    'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '2',
    '3', '4', '5', '6', '7', '8', '9',
];


fn validate_url(input: &str) -> Result<String, AppError> {
    let url = Url::parse(input).map_err(|_| {
        AppError::InvalidUrl(format!("'{}' is not a valid URL", input))
    })?;

    match url.scheme() {
        "http" | "https" => Ok(url.to_string()),
        scheme => Err(AppError::InvalidUrl(
            format!("Unsupported scheme '{}'. Use http or https", scheme),
        )),
    }
}


fn validate_custom_code(code: &str) -> Result<(), AppError> {
    if code.len() < 3 || code.len() > 20 {
        return Err(AppError::InvalidUrl(
            "Custom code must be between 3 and 20 characters".to_string(),
        ));
    }

    if !code.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::InvalidUrl(
            "Custom code can only contain letters, numbers, hyphens, and underscores".to_string(),
        ));
    }

    Ok(())
}


fn generate_short_code() -> String {
    nanoid::nanoid!(SHORT_CODE_LENGTH, SHORT_CODE_ALPHABET)
}


fn calculate_expiry(seconds: u64) -> String {
    let expiry = Utc::now() + Duration::seconds(seconds as i64);
    expiry.format("%Y-%m-%d %H:%M:%S").to_string()
}


fn is_expired(expires_at: &Option<String>) -> bool {
    if let Some(exp) = expires_at {
        if let Ok(expiry) = chrono::NaiveDateTime::parse_from_str(exp, "%Y-%m-%d %H:%M:%S") {
            let now = Utc::now().naive_utc();
            return now > expiry;
        }
    }
    false
}


pub async fn shorten_url(
    pool: &DbPool,
    config: &AppConfig,
    request: ShortenRequest,
) -> Result<ShortenResponse, AppError> {
   
    let original_url = validate_url(&request.url)?;

    
    if request.custom_code.is_none() {
        if let Some(existing) = db::get_url_by_original(pool, &original_url).await? {
            if !is_expired(&existing.expires_at) {
                return Ok(ShortenResponse {
                    short_code: existing.short_code.clone(),
                    short_url: format!("{}/{}", config.base_url, existing.short_code),
                    original_url: existing.original_url,
                    created_at: existing.created_at,
                    expires_at: existing.expires_at,
                });
            }
        }
    }

   
    let short_code = match &request.custom_code {
        Some(code) => {
            validate_custom_code(code)?;
            code.clone()
        }
        None => {
            
            let mut code = generate_short_code();
            let mut attempts = 0;
            while db::get_url_by_code(pool, &code).await?.is_some() {
                code = generate_short_code();
                attempts += 1;
                if attempts > 10 {
                    return Err(AppError::InternalError(
                        "Failed to generate unique code".to_string(),
                    ));
                }
            }
            code
        }
    };

  
    let expires_at = request.expires_in.map(calculate_expiry);

    
    let record = db::insert_url(
        pool,
        &short_code,
        &original_url,
        expires_at.as_deref(),
    )
    .await?;

   
    Ok(ShortenResponse {
        short_code: record.short_code.clone(),
        short_url: format!("{}/{}", config.base_url, record.short_code),
        original_url: record.original_url,
        created_at: record.created_at,
        expires_at: record.expires_at,
    })
}


pub async fn resolve_url(
    pool: &DbPool,
    code: &str,
) -> Result<String, AppError> {
    let record = db::get_url_by_code(pool, code)
        .await?
        .ok_or_else(|| AppError::CodeNotFound(code.to_string()))?;

    
    if is_expired(&record.expires_at) {
        return Err(AppError::UrlExpired(code.to_string()));
    }

    
    db::increment_click_count(pool, code).await?;

    Ok(record.original_url)
}


pub async fn get_stats(
    pool: &DbPool,
    code: &str,
) -> Result<UrlStats, AppError> {
    let record = db::get_url_by_code(pool, code)
        .await?
        .ok_or_else(|| AppError::CodeNotFound(code.to_string()))?;

    Ok(UrlStats {
        short_code: record.short_code,
        original_url: record.original_url,
        click_count: record.click_count,
        created_at: record.created_at,
        expires_at: record.expires_at,
    })
}


pub async fn delete_short_url(
    pool: &DbPool,
    code: &str,
) -> Result<(), AppError> {
    let deleted = db::delete_url(pool, code).await?;

    if !deleted {
        return Err(AppError::CodeNotFound(code.to_string()));
    }

    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_urls() {
        assert!(validate_url("https://www.google.com").is_ok());
        assert!(validate_url("http://example.com/path?q=1").is_ok());
        assert!(validate_url("https://sub.domain.com/a/b/c").is_ok());
    }

    #[test]
    fn test_validate_invalid_urls() {
        assert!(validate_url("not-a-url").is_err());
        assert!(validate_url("ftp://files.com").is_err());
        assert!(validate_url("").is_err());
        assert!(validate_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_validate_custom_code() {
        assert!(validate_custom_code("my-url").is_ok());
        assert!(validate_custom_code("abc123").is_ok());
        assert!(validate_custom_code("a_b").is_ok());

        assert!(validate_custom_code("ab").is_err());       // too short
        assert!(validate_custom_code("a".repeat(21).as_str()).is_err()); // too long
        assert!(validate_custom_code("ab cd").is_err());     // space
        assert!(validate_custom_code("ab@cd").is_err());     // special char
    }

    #[test]
    fn test_generate_short_code_length() {
        let code = generate_short_code();
        assert_eq!(code.len(), SHORT_CODE_LENGTH);
    }

    #[test]
    fn test_generate_short_code_uniqueness() {
        let codes: Vec<String> = (0..100).map(|_| generate_short_code()).collect();
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(codes.len(), unique.len());
    }

    #[test]
    fn test_is_expired() {
        
        let future = (Utc::now() + Duration::hours(1))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        assert!(!is_expired(&Some(future)));

       
        let past = (Utc::now() - Duration::hours(1))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        assert!(is_expired(&Some(past)));

        
        assert!(!is_expired(&None));
    }
}