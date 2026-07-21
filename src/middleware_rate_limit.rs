#![allow(dead_code)]
use actix_web::dev::{Service, ServiceResponse, Transform};
use actix_web::{dev::ServiceRequest, Error};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::warn;

use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window: Duration,
    pub block_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            block_duration: Duration::from_secs(300),
        }
    }
}

#[derive(Clone)]
struct ClientState {
    requests: Vec<Instant>,
    blocked_until: Option<Instant>,
}

pub struct RateLimiter {
    clients: Arc<RwLock<HashMap<IpAddr, ClientState>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn check(&self, client_ip: IpAddr) -> Result<(), AppError> {
        let mut clients = self.clients.write().await;
        let now = Instant::now();

        let state = clients.entry(client_ip).or_insert_with(|| ClientState {
            requests: Vec::new(),
            blocked_until: None,
        });

        // ブロックチェック
        if let Some(until) = state.blocked_until {
            if now < until {
                return Err(AppError::BadRequest(
                    "Rate limit exceeded. Try again later.".to_string(),
                ));
            }
            state.blocked_until = None;
        }

        // 古いリクエストを削除
        state
            .requests
            .retain(|&t| now.duration_since(t) < self.config.window);

        // レート制限チェック
        if state.requests.len() >= self.config.max_requests as usize {
            state.blocked_until = Some(now + self.config.block_duration);
            warn!("Rate limit exceeded for IP: {}", client_ip);
            return Err(AppError::BadRequest(format!(
                "Rate limit exceeded. Blocked for {} seconds.",
                self.config.block_duration.as_secs()
            )));
        }

        state.requests.push(now);
        Ok(())
    }
}

// Actix-web Transform実装
pub struct RateLimitMiddleware {
    limiter: Arc<RateLimiter>,
}

impl RateLimitMiddleware {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new(config)),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddlewareService {
            service,
            limiter: self.limiter.clone(),
        })
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: S,
    limiter: Arc<RateLimiter>,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let client_ip = req
            .connection_info()
            .realip_remote_addr()
            .and_then(|ip| ip.parse().ok())
            .unwrap_or(IpAddr::from([127, 0, 0, 1]));

        let limiter = self.limiter.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            if let Err(_e) = limiter.check(client_ip).await {
                // Rate limit exceeded - return error as middleware error
                return Err(actix_web::error::ErrorTooManyRequests(
                    "Rate limit exceeded",
                ));
            }
            fut.await
        })
    }
}
