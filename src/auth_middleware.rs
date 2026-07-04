use actix_web::{
    body::EitherBody, dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, web,
};
use futures::future::{ok, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::rc::Rc;

use crate::{error::AppError, models::Claims, app_state::AppState, cache::session::SessionManager, config::JWT_SECRET};
use tracing::warn;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let path = req.path().to_string();

        Box::pin(async move {
            if path.starts_with("/api/v1/auth/login")
                || path.starts_with("/api/v1/auth/register")
                || path.starts_with("/api/v1/auth/refresh")
                || path.starts_with("/api/v1/metrics")
                || path.starts_with("/api/v1/health")
                || path.starts_with("/health")
                || path == "/metrics"
            {
                return service.call(req).await.map(|res| res.map_into_left_body());
            }

            // Get AppState from request extensions
            let app_state = req.app_data::<web::Data<AppState>>();

            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        // Use JWT_SECRET directly (consistent with auth.rs)
                        let jwt_secret = JWT_SECRET.as_str();

                        match verify_jwt(token, jwt_secret) {
                            Ok(claims) => {
                                // Validate session in Redis if cache available
                                if let Some(state) = app_state {
                                    if let Some(cache) = &state.cache {
                                        let session_mgr = SessionManager::new(cache.clone());

                                        match session_mgr.validate_session(token).await {
                                            Ok(session_data) => {
                                                // Session valid - extend activity timestamp
                                                let _ = session_mgr.extend_session(token).await;
                                            }
                                            Err(e) => {
                                                warn!("Session validation failed: {:?}", e);
                                                return Ok(req.into_response(
                                                    HttpResponse::Unauthorized()
                                                        .json(serde_json::json!({
                                                            "error": "Session expired or invalid",
                                                            "code": 401
                                                        }))
                                                        .map_into_right_body(),
                                                ));
                                            }
                                        }
                                    }
                                }

                                req.extensions_mut().insert(claims);
                                return service.call(req).await.map(|res| res.map_into_left_body());
                            }
                            Err(_) => {
                                return Ok(req.into_response(
                                    HttpResponse::Unauthorized()
                                        .json(serde_json::json!({
                                            "error": "Invalid token",
                                            "code": 401
                                        }))
                                        .map_into_right_body(),
                                ));
                            }
                        }
                    }
                }
            }

            Ok(req.into_response(
                HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "Missing or invalid authorization",
                        "code": 401
                    }))
                    .map_into_right_body(),
            ))
        })
    }
}

fn verify_jwt(token: &str, jwt_secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use jsonwebtoken::encode;

    #[test]
    fn test_verify_jwt() {
        let settings = crate::config::Settings::default();
        let now = Utc::now();
        let claims = Claims {
            sub: "test-user-id".to_string(),
            username: "testuser".to_string(),
            iat: now.timestamp(),
            exp: (now + chrono::Duration::hours(24)).timestamp(),
        };

        let token = encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(settings.auth.jwt_secret.as_bytes()),
        )
        .expect("Failed to encode JWT");

        let verified = verify_jwt(&token, &settings.auth.jwt_secret);
        assert!(verified.is_ok());
        assert_eq!(verified.unwrap().sub, "test-user-id");
    }

    #[test]
    fn test_invalid_token() {
        let settings = crate::config::Settings::default();
        let result = verify_jwt("invalid.token.here", &settings.auth.jwt_secret);
        assert!(result.is_err());
    }
}
