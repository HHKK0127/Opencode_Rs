use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use prometheus::{Counter, CounterVec, Encoder, HistogramVec, IntGauge, Registry, TextEncoder};
use std::rc::Rc;
use std::time::Instant;

lazy_static::lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = CounterVec::new(
        prometheus::Opts::new("http_requests_total", "Total HTTP requests"),
        &["method", "endpoint", "status"],
    ).expect("Failed to create http_requests_total");

    pub static ref HTTP_REQUEST_DURATION_SECONDS: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        ),
        &["method", "endpoint"],
    ).expect("Failed to create http_request_duration_seconds");

    pub static ref HTTP_REQUEST_SIZE_BYTES: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "http_request_size_bytes",
            "HTTP request size in bytes"
        ),
        &["method", "endpoint"],
    ).expect("Failed to create http_request_size_bytes");

    pub static ref HTTP_RESPONSE_SIZE_BYTES: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "http_response_size_bytes",
            "HTTP response size in bytes"
        ),
        &["method", "endpoint"],
    ).expect("Failed to create http_response_size_bytes");

    pub static ref ACTIVE_CONNECTIONS: IntGauge = IntGauge::new(
        "active_connections",
        "Current number of active connections"
    ).expect("Failed to create active_connections");

    pub static ref FILE_UPLOAD_BYTES_TOTAL: Counter = Counter::new(
        "file_upload_bytes_total",
        "Total bytes uploaded"
    ).expect("Failed to create file_upload_bytes_total");
}

pub fn init_metrics() -> Result<(), Box<dyn std::error::Error>> {
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(HTTP_REQUEST_DURATION_SECONDS.clone()))?;
    REGISTRY.register(Box::new(HTTP_REQUEST_SIZE_BYTES.clone()))?;
    REGISTRY.register(Box::new(HTTP_RESPONSE_SIZE_BYTES.clone()))?;
    REGISTRY.register(Box::new(ACTIVE_CONNECTIONS.clone()))?;
    REGISTRY.register(Box::new(FILE_UPLOAD_BYTES_TOTAL.clone()))?;
    Ok(())
}

pub fn get_metrics() -> Result<String, prometheus::Error> {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

pub struct MetricsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MetricsMiddlewareService<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(MetricsMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct MetricsMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().to_string();
        let path = req.path().to_string();
        let request_size = req
            .headers()
            .get("content-length")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let service = self.service.clone();
        let method_clone = method.clone();
        let path_clone = path.clone();

        // Extract endpoint (simplified)
        let endpoint = if path_clone.contains("/health") {
            "health"
        } else if path_clone.contains("/api/v1/auth") {
            "auth"
        } else if path_clone.contains("/api/v1/files") {
            "files"
        } else if path_clone.contains("/api/v1/users") {
            "users"
        } else {
            "other"
        };

        ACTIVE_CONNECTIONS.inc();
        let start = Instant::now();

        Box::pin(async move {
            let res = service.call(req).await?;

            let status = res.status().as_u16().to_string();
            let duration = start.elapsed().as_secs_f64();
            let response_size = res
                .headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            HTTP_REQUESTS_TOTAL
                .with_label_values(&[&method_clone, endpoint, &status])
                .inc();

            HTTP_REQUEST_DURATION_SECONDS
                .with_label_values(&[&method_clone, endpoint])
                .observe(duration);

            if request_size > 0.0 {
                HTTP_REQUEST_SIZE_BYTES
                    .with_label_values(&[&method_clone, endpoint])
                    .observe(request_size);
            }

            if response_size > 0.0 {
                HTTP_RESPONSE_SIZE_BYTES
                    .with_label_values(&[&method_clone, endpoint])
                    .observe(response_size);
            }

            ACTIVE_CONNECTIONS.dec();

            Ok(res)
        })
    }
}
