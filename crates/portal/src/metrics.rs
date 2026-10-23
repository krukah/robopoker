//! HTTP request metrics middleware.
//!
//! Instruments every request with `rbp.http.requests` (counter) and
//! `rbp.http.duration_ms` (histogram), labeled by `method`, `route`,
//! and `status`. Uses the matched route pattern (e.g., `/topology/hst-wrt-abs`)
//! rather than the raw path to keep label cardinality bounded.
use super::*;
use actix_web::Error;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use futures::future::Ready;
use futures::future::ready;

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
        ready(Ok(MetricsMiddleware::new(service)))
    }
}
