//! Instrumentation wrapper for SQL operations.
//!
//! Every `measure("name", future).await` bumps the `rbp.db.queries` counter
//! and records the duration in `rbp.db.query_ms` with the query name as a
//! label. Keep `name` a compile-time `&'static str` — cardinality stays
//! bounded to the set of trait-method names.
//!
//! Consumers in this crate wrap their SQL calls at the trait-impl level
//! (Ensure, Streamable, Stage, Check). Do not label with per-row or
//! per-request identifiers — those belong in trace attributes, not
//! metric labels.

use std::future::Future;
use std::time::Instant;
use tracing::Instrument;
use vitals::KeyValue;

/// Instruments a future that executes SQL. Non-async work inside the future
/// (polling, buffering) counts toward the measured duration; that's
/// intentional — we want wall-clock time for the DB interaction from the
/// caller's perspective.
///
/// Emits both the `rbp.db.*` metric pair *and* a `db.query` tracing span
/// so trace waterfalls show nested DB calls under the caller's span.
pub async fn measure<F, T>(name: &'static str, f: F) -> T
where
    F: Future<Output = T>,
{
    let handles = vitals::metrics::get();
    let labels = [KeyValue::new("query_name", name)];
    handles.db_queries.add(1, &labels);
    let span = tracing::info_span!("db.query", query_name = name);
    let t0 = Instant::now();
    let result = f.instrument(span).await;
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    handles.db_query_ms.record(ms, &labels);
    result
}
