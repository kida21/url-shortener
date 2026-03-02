use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt; // for `oneshot`
use http_body_util::BodyExt; // for `collect`


use url_shortener::{
    config::AppConfig,
    db,
    router::create_router,
    state::AppState,
};


async fn setup_test_app() -> axum::Router {
    let config = AppConfig {
        database_url: "sqlite::memory:".to_string(),
        host: "127.0.0.1".to_string(),
        port: 3000,
        base_url: "http://localhost:3000".to_string(),
    };

    let pool = db::init_db(&config.database_url)
        .await
        .expect("Failed to init test DB");

    let state = AppState::new(pool, config);
    create_router(state)
}

/// Helper: Make a JSON POST request
fn post_json(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Helper: Make a GET request
fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}


fn delete_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}


async fn response_json(response: axum::response::Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}



#[tokio::test]
async fn test_health_check() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(get_request("/health"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["status"], "healthy");
    assert!(body["version"].is_string());
}



#[tokio::test]
async fn test_shorten_valid_url() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(post_json(
            "/api/shorten",
            json!({"url": "https://www.rust-lang.org"}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_json(response).await;
    assert!(body["short_code"].is_string());
    assert!(body["short_url"].as_str().unwrap().starts_with("http://localhost:3000/"));
    assert_eq!(body["original_url"], "https://www.rust-lang.org/");
    assert!(body["created_at"].is_string());
}

#[tokio::test]
async fn test_shorten_with_custom_code() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(post_json(
            "/api/shorten",
            json!({
                "url": "https://www.rust-lang.org",
                "custom_code": "rustlang"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_json(response).await;
    assert_eq!(body["short_code"], "rustlang");
    assert_eq!(body["short_url"], "http://localhost:3000/rustlang");
}

#[tokio::test]
async fn test_shorten_with_expiration() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(post_json(
            "/api/shorten",
            json!({
                "url": "https://example.com",
                "expires_in": 3600
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_json(response).await;
    assert!(body["expires_at"].is_string());
}

#[tokio::test]
async fn test_shorten_invalid_url() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(post_json(
            "/api/shorten",
            json!({"url": "not-a-valid-url"}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "INVALID_URL");
}

#[tokio::test]
async fn test_shorten_ftp_url_rejected() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(post_json(
            "/api/shorten",
            json!({"url": "ftp://files.example.com"}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "INVALID_URL");
}

#[tokio::test]
async fn test_shorten_duplicate_returns_existing() {
    let app = setup_test_app().await;

    // First request
    let response1 = app
        .clone()
        .oneshot(post_json(
            "/api/shorten",
            json!({"url": "https://duplicate-test.com"}),
        ))
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);
    let body1 = response_json(response1).await;
    let code1 = body1["short_code"].as_str().unwrap().to_string();

  
    let response2 = app
        .oneshot(post_json(
            "/api/shorten",
            json!({"url": "https://duplicate-test.com"}),
        ))
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::CREATED);
    let body2 = response_json(response2).await;
    let code2 = body2["short_code"].as_str().unwrap().to_string();

    
    assert_eq!(code1, code2);
}

#[tokio::test]
async fn test_shorten_custom_code_too_short() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(post_json(
            "/api/shorten",
            json!({
                "url": "https://example.com",
                "custom_code": "ab"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_shorten_duplicate_custom_code() {
    let app = setup_test_app().await;

    // First request
    let response1 = app
        .clone()
        .oneshot(post_json(
            "/api/shorten",
            json!({
                "url": "https://example1.com",
                "custom_code": "mycode"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response1.status(), StatusCode::CREATED);

    
    let response2 = app
        .oneshot(post_json(
            "/api/shorten",
            json!({
                "url": "https://example2.com",
                "custom_code": "mycode"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::CONFLICT);

    let body = response_json(response2).await;
    assert_eq!(body["error"]["code"], "CODE_EXISTS");
}



#[tokio::test]
async fn test_redirect_valid_code() {
    let app = setup_test_app().await;

   
    let create_resp = app
        .clone()
        .oneshot(post_json(
            "/api/shorten",
            json!({
                "url": "https://www.google.com",
                "custom_code": "goog"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);

   
    let response = app
        .oneshot(get_request("/goog"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);

    let location = response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(location, "https://www.google.com/");
}

#[tokio::test]
async fn test_redirect_invalid_code() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(get_request("/nonexistent"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "CODE_NOT_FOUND");
}



#[tokio::test]
async fn test_stats_valid_code() {
    let app = setup_test_app().await;

    
    let create_resp = app
        .clone()
        .oneshot(post_json(
            "/api/shorten",
            json!({
                "url": "https://stats-test.com",
                "custom_code": "stats-test"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    
    let _ = app
        .clone()
        .oneshot(get_request("/stats-test"))
        .await
        .unwrap();

  
    let response = app
        .oneshot(get_request("/api/stats/stats-test"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["short_code"], "stats-test");
    assert_eq!(body["original_url"], "https://stats-test.com/");
    assert_eq!(body["click_count"], 1);
}

#[tokio::test]
async fn test_stats_not_found() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(get_request("/api/stats/doesnotexist"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}



#[tokio::test]
async fn test_delete_existing_url() {
    let app = setup_test_app().await;

   
    let create_resp = app
        .clone()
        .oneshot(post_json(
            "/api/shorten",
            json!({
                "url": "https://delete-me.com",
                "custom_code": "del-test"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);

   
    let response = app
        .clone()
        .oneshot(delete_request("/api/urls/del-test"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

   
    let response = app
        .oneshot(get_request("/api/stats/del-test"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_url() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(delete_request("/api/urls/ghost"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}



#[tokio::test]
async fn test_click_count_increments() {
    let app = setup_test_app().await;

    
    app.clone()
        .oneshot(post_json(
            "/api/shorten",
            json!({
                "url": "https://click-test.com",
                "custom_code": "clicks"
            }),
        ))
        .await
        .unwrap();

   
    for _ in 0..3 {
        let _ = app
            .clone()
            .oneshot(get_request("/clicks"))
            .await
            .unwrap();
    }

  
    let response = app
        .oneshot(get_request("/api/stats/clicks"))
        .await
        .unwrap();

    let body = response_json(response).await;
    assert_eq!(body["click_count"], 3);
}