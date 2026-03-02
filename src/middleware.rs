use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;


#[derive(Debug, Clone)]
pub struct RateLimiter {
   
    requests: Arc<Mutex<HashMap<String, (u64, Instant)>>>,
    max_requests: u64,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u64, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

   
    async fn check(&self, ip: &str) -> Result<u64, ()> {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();

        let entry = requests
            .entry(ip.to_string())
            .or_insert((0, now));

        
        if now.duration_since(entry.1) > self.window {
            *entry = (0, now);
        }

        entry.0 += 1;

        if entry.0 > self.max_requests {
            Err(())
        } else {
            Ok(self.max_requests - entry.0)
        }
    }

   
    pub fn start_cleanup_task(&self) {
        let requests = self.requests.clone();
        let window = self.window;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(window).await;
                let mut map = requests.lock().await;
                let now = Instant::now();
                map.retain(|_, (_, start)| now.duration_since(*start) <= window);
                tracing::debug!("Rate limiter cleanup: {} entries remaining", map.len());
            }
        });
    }
}


pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    
    let limiter = request
        .extensions()
        .get::<RateLimiter>()
        .cloned();

    if let Some(limiter) = limiter {
        let ip = addr.ip().to_string();

        match limiter.check(&ip).await {
            Ok(remaining) => {
                let mut response = next.run(request).await;
               
                response.headers_mut().insert(
                    "X-RateLimit-Remaining",
                    remaining.to_string().parse().unwrap(),
                );
                response
            }
            Err(_) => {
                tracing::warn!("Rate limit exceeded for IP: {}", ip);
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({
                        "error": {
                            "code": "RATE_LIMIT_EXCEEDED",
                            "message": "Too many requests. Please try again later."
                        }
                    })),
                )
                    .into_response()
            }
        }
    } else {
        next.run(request).await
    }
}