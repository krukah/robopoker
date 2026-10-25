use super::*;

pub(crate) const DEFAULT_OTLP_ENDPOINT: &str = "http://localhost:4317";

/// Default tracing-subscriber filter applied when `RUST_LOG` is unset.
///
/// Info-by-default: keeps lifecycle events while dropping the per-event debug
/// chatter that drove 99% of log ingestion cost. Hot-path logs are `debug!`/
/// `trace!` and suppressed here. Override with `RUST_LOG=debug` or
/// `RUST_LOG=spar=debug`.
pub(crate) const DEFAULT_FILTER: &str = "info,actix_web=info,tokio_postgres=info";

pub(crate) const METRICS_INTERVAL: Duration = Duration::from_secs(15);

pub(crate) static INITIALIZED: OnceLock<()> = OnceLock::new();
