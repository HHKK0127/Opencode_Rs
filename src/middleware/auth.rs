use actix_web::{
    body::EitherBody, dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::rc::Rc;

use crate::{error::AppError, models::Claims};

const JWT_SECRET: &str = "test_secret_key_for_poc_verification";

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
    type Future = futures_util::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures_util::future::ok(AuthMiddlewareService {
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
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            if req.path().starts_with("/api/v1/auth/login")
                || req.path().starts_with("/api/v1/auth/register")
                || req.path().starts_with("/health")
            {
                return service.call(req).await.map(|res| res.map_into_left_body());
            }

            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        match verify_jwt(token) {
                            Ok(claims) => {
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

fn verify_jwt(token: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
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
            &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        )
        .expect("Failed to encode JWT");

        let verified = verify_jwt(&token);
        assert!(verified.is_ok());
        assert_eq!(verified.unwrap().sub, "test-user-id");
    }

    #[test]
    fn test_invalid_token() {
        let result = verify_jwt("invalid.token.here");
        assert!(result.is_err());
    }
}
