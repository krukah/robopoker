//! Global cap on simultaneous requests to one external API.

use std::sync::Arc;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::Semaphore;

/// Shared concurrency limiter for a benchmark service's HTTP surface.
///
/// When one process runs several variant benchmarks at once they each hold a clone of the same
/// throttle and serialize through it, so the aggregate in-flight count is capped globally
/// rather than per-variant. The external APIs are research endpoints we don't control, and
/// nothing gets a benchmark revoked faster than hammering one.
#[derive(Clone)]
pub struct Throttle(Arc<Semaphore>);

impl Throttle {
    /// Cap the in-flight requests across every holder of this clone.
    pub fn new(inflight: usize) -> Self {
        Self(Arc::new(Semaphore::new(inflight.max(1))))
    }

    /// Wait for a slot. Hold the permit for the duration of the request.
    pub async fn acquire(&self) -> Option<OwnedSemaphorePermit> {
        self.0.clone().acquire_owned().await.ok()
    }
}
