use actix_web::Error;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use futures::future::LocalBoxFuture;
use std::time::Instant;

pub struct MetricsMiddleware<S> {
    service: S,
}

impl<S> MetricsMiddleware<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }

    pub fn service(&self) -> &S {
        &self.service
    }
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
        let fut = self.service().call(req);
        Box::pin(async move {
            let res = fut.await?;
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let status = res.status().as_u16().to_string();
            let labels = [
                vitals::KeyValue::new("method", method),
                vitals::KeyValue::new("route", route),
                vitals::KeyValue::new("status", status),
            ];
            let metrics = vitals::metrics::Handles::get();
            metrics.http_requests().add(1, &labels);
            metrics.http_duration_ms().record(ms, &labels);
            Ok(res)
        })
    }
}
