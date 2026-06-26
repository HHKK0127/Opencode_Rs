use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;

#[derive(Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

impl BasicAuth {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn check(&self, header: &str) -> bool {
        let expected = format!("{}:{}", self.username, self.password);
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            expected.as_bytes(),
        );
        let expected_header = format!("Basic {}", encoded);
        header == expected_header
    }
}

pub struct BasicAuthMiddleware {
    pub auth: BasicAuth,
}

impl<S, B> Transform<S, ServiceRequest> for BasicAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Transform = BasicAuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BasicAuthMiddlewareService {
            service: Rc::new(service),
            auth: self.auth.clone(),
        }))
    }
}

pub struct BasicAuthMiddlewareService<S> {
    service: Rc<S>,
    auth: BasicAuth,
}

impl<S, B> Service<ServiceRequest> for BasicAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();

        if path == "/global/health" || path == "/api/health" {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        let auth_header = req
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        let is_valid = match &auth_header {
            Some(header) => self.auth.check(header),
            None => false,
        };

        if !is_valid {
            return Box::pin(async move {
                let res = actix_web::HttpResponse::Unauthorized()
                    .insert_header(("www-authenticate", "Basic realm=\"opencode\""))
                    .finish();
                Ok(req.into_response(res).map_into_right_body())
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}
