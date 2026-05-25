//! HTTP request metrics middleware.
//!
//! Instruments every request with `rbp.http.requests` (counter) and
//! `rbp.http.duration_ms` (histogram), labeled by `method`, `route`,
//! and `status`. Uses the matched route pattern (e.g., `/topology/hst-wrt-abs`)
//! rather than the raw path to keep label cardinality bounded.
use actix_web::Error;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use futures::future::LocalBoxFuture;
use futures::future::Ready;
use futures::future::ready;
use std::time::Instant;

/// Wrap-this type for `App::wrap(Metrics)` — emits `rbp.http.*` per request.
pub struct Metrics;

impl<S, B> Transform<S, ServiceRequest> for Metrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddleware { service }))
    }
}

pub struct MetricsMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let t0 = Instant::now();
        let method = req.method().as_str().to_owned();
        let route = req.match_pattern().unwrap_or_else(|| "unmatched".to_string());
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let status = res.status().as_u16().to_string();
            let labels = [
                rbp_telemetry::KeyValue::new("method", method),
                rbp_telemetry::KeyValue::new("route", route),
                rbp_telemetry::KeyValue::new("status", status),
            ];
            let metrics = rbp_telemetry::metrics::get();
            metrics.http_requests.add(1, &labels);
            metrics.http_duration_ms.record(ms, &labels);
            Ok(res)
        })
    }
}
