use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;

use crate::error::AppError;
use crate::models::UrlRecord;

pub type DbPool = Pool<Sqlite>;


pub async fn init_db(database_url: &str) -> Result<DbPool, AppError> {
   
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        
        if path != ":memory:" {
            let db_path = Path::new(path);

           
            if let Some(parent) = db_path.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        AppError::DatabaseError(format!(
                            "Failed to create database directory '{}': {}",
                            parent.display(),
                            e
                        ))
                    })?;
                    tracing::info!("Created database directory: {}", parent.display());
                }
            }

           
            if !db_path.exists() {
                std::fs::File::create(db_path).map_err(|e| {
                    AppError::DatabaseError(format!(
                        "Failed to create database file '{}': {}",
                        db_path.display(),
                        e
                    ))
                })?;
                tracing::info!("Created database file: {}", db_path.display());
            }
        }
    }

   
    let connect_url = if database_url.contains("?") {
        format!("{}&create=true", database_url)
    } else {
        format!("{}?mode=rwc", database_url)
    };

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&connect_url)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to connect: {}", e)))?;

    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS urls (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            short_code   TEXT NOT NULL UNIQUE,
            original_url TEXT NOT NULL,
            click_count  INTEGER DEFAULT 0,
            created_at   TEXT DEFAULT (datetime('now')),
            expires_at   TEXT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_short_code ON urls(short_code);")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_original_url ON urls(original_url);")
        .execute(&pool)
        .await?;

    tracing::info!("Database initialized successfully");
    Ok(pool)
}


pub async fn insert_url(
    pool: &DbPool,
    short_code: &str,
    original_url: &str,
    expires_at: Option<&str>,
) -> Result<UrlRecord, AppError> {
    sqlx::query_as::<_, UrlRecord>(
        r#"
        INSERT INTO urls (short_code, original_url, expires_at)
        VALUES (?, ?, ?)
        RETURNING id, short_code, original_url, click_count, created_at, expires_at
        "#,
    )
    .bind(short_code)
    .bind(original_url)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.message().contains("UNIQUE") => {
            AppError::CodeAlreadyExists(short_code.to_string())
        }
        _ => AppError::DatabaseError(e.to_string()),
    })
}


pub async fn get_url_by_code(
    pool: &DbPool,
    code: &str,
) -> Result<Option<UrlRecord>, AppError> {
    let record = sqlx::query_as::<_, UrlRecord>(
        "SELECT id, short_code, original_url, click_count, created_at, expires_at FROM urls WHERE short_code = ?"
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;

    Ok(record)
}


pub async fn get_url_by_original(
    pool: &DbPool,
    original_url: &str,
) -> Result<Option<UrlRecord>, AppError> {
    let record = sqlx::query_as::<_, UrlRecord>(
        "SELECT id, short_code, original_url, click_count, created_at, expires_at FROM urls WHERE original_url = ?"
    )
    .bind(original_url)
    .fetch_optional(pool)
    .await?;

    Ok(record)
}


pub async fn increment_click_count(
    pool: &DbPool,
    code: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE urls SET click_count = click_count + 1 WHERE short_code = ?")
        .bind(code)
        .execute(pool)
        .await?;

    Ok(())
}


pub async fn delete_url(
    pool: &DbPool,
    code: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM urls WHERE short_code = ?")
        .bind(code)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}