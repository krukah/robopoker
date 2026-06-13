//! Telemetry init and metric registry for rbp binaries.
//!
//! One call — `let _guard = rbp_telemetry::init();` — installs a `tracing`
//! subscriber, routes `log::*` calls through it via the `tracing-log` shim,
//! configures the OTLP metrics exporter, and pre-registers the workspace's
//! metric handles under `metrics::*`.
//!
//! Environment variables (all optional):
//!   OTEL_EXPORTER_OTLP_ENDPOINT  default http://localhost:4317 (gRPC)
//!   OTEL_SERVICE_NAME            default inferred from CARGO_BIN_NAME
//!   OTEL_RESOURCE_ATTRIBUTES     extra k=v pairs merged into resource
//!   RUST_LOG                     default `info,rbp=debug`
//!   RBP_TELEMETRY_DISABLED       if set and truthy, skip OTLP; fmt layer only

use std::sync::OnceLock;
use std::time::Duration;

use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::Aggregation;
use opentelemetry_sdk::metrics::Instrument;
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::metrics::Stream;
use opentelemetry_sdk::metrics::View;
use opentelemetry_sdk::metrics::new_view;
use opentelemetry_sdk::runtime;
use opentelemetry_semantic_conventions as semcov;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub mod metrics;

pub use opentelemetry::KeyValue;

const DEFAULT_OTLP_ENDPOINT: &str = "http://localhost:4317";
/// Default tracing-subscriber filter applied when `RUST_LOG` is unset.
///
/// Info-by-default for everything. This keeps lifecycle events (task
/// start/stop, hand-complete, metric handle registration, exploit
/// checkpoints, etc.) while dropping per-event debug chatter that used
/// to drive 99% of log ingestion cost. Per-hand session detail
/// lines in slumbot, per-node visit traces, and similar hot-path logs
/// are `debug!` or `trace!` and are now suppressed by default.
///
/// Override at runtime via `RUST_LOG=debug` (all crates debug) or
/// `RUST_LOG=rbp_slumbot=debug` (just one crate) when investigating a
/// specific issue.
const DEFAULT_FILTER: &str = "info,actix_web=info,tokio_postgres=info";
const METRICS_INTERVAL: Duration = Duration::from_secs(15);

/// Telemetry lifetime. Flushes exporters on drop. Hold this in `main()` for
/// the full binary lifetime.
pub struct TelemetryGuard {
    meter: Option<SdkMeterProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(m) = self.meter.take() {
            let _: Result<(), _> = m.shutdown();
        }
    }
}

/// Initialize the global telemetry stack. Returns a guard that flushes on drop.
///
/// Must be called inside a tokio runtime (the OTLP exporters schedule background
/// work on it). The four rbp binaries are `#[tokio::main]` so this holds.
pub fn init() -> TelemetryGuard {
    let service = service_name();
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_else(|_| DEFAULT_OTLP_ENDPOINT.to_string());
    let disabled = std::env::var("RBP_TELEMETRY_DISABLED")
        .ok()
        .is_some_and(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"));
    let resource = build_resource(&service);
    let meter_provider = if disabled { None } else { install_meter(&endpoint, resource.clone()) };
    let _ = tracing_log::LogTracer::init();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(false)
        .with_writer(std::io::stderr);
    let _ = tracing_subscriber::registry().with(filter).with(fmt_layer).try_init();
    if let Some(ref mp) = meter_provider {
        global::set_meter_provider(mp.clone());
    }
    metrics::install(&global::meter("rbp"));
    if INITIALIZED.set(()).is_ok() {
        if disabled {
            tracing::info!(service = %service, "telemetry initialized (otlp disabled)");
        } else {
            tracing::info!(service = %service, endpoint = %endpoint, "telemetry initialized");
        }
    }
    TelemetryGuard { meter: meter_provider }
}

fn install_meter(endpoint: &str, resource: Resource) -> Option<SdkMeterProvider> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(3))
        .build()
        .ok()?;
    let reader = PeriodicReader::builder(exporter, runtime::Tokio)
        .with_interval(METRICS_INTERVAL)
        .build();
    let mut builder = SdkMeterProvider::builder().with_resource(resource).with_reader(reader);
    for view in histogram_views() {
        builder = builder.with_view(view);
    }
    Some(builder.build())
}

/// Explicit bucket boundaries for histograms whose data ranges don't fit the
/// OpenTelemetry default of `[0, 5, 10, 25, 50, 75, 100, 250, 500, 750, 1000,
/// 2500, 5000, 7500, 10000, +Inf]`. Without these, data above 10000 collapses
/// into the `+Inf` bucket and quantile estimates become meaningless.
///
/// Log-style histograms (anything spanning more than ~1 decade of magnitude)
/// use power-of-two boundaries via [`log2_view`]. Each adjacent pair of
/// buckets covers a doubling, so the visual band heights on a linear y-axis
/// are proportional to the bucket widths — no arbitrary "1, 2, 5, 10" cutoffs
/// to defend.
///
/// Two classes stay on hand-picked boundaries: signed data (Slumbot hand_bb)
/// and bounded data (subgame.policy_deviation, mccfr.flush_duration_ms).
fn histogram_views() -> Vec<Box<dyn View>> {
    let views = [
        // Subgame CFR iteration count per decision — observed 5k–500k on
        // realtime variants. 2^7 (128) → 2^20 (~1M).
        log2_view("rbp.subgame.iterations", 7, 20),
        // L1 distance between two probability distributions — bounded 0..=2.
        // Linear because the range is small and the interesting cliff is
        // near 0 (refined ≈ blueprint) vs ≈2 (fully disjoint).
        histogram_view(
            "rbp.subgame.policy_deviation",
            &[0.0, 0.05, 0.1, 0.2, 0.3, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0],
        ),
        // regret / pot — observed 0.5 to 2000+ in smokes. 2^-7 (~0.008) →
        // 2^17 (~131k) covers converged and diverged ends.
        log2_view("rbp.subgame.relative_regret", -7, 17),
        // bb/hand on Slumbot — signed, stack depth ~200 bb. Log2 doesn't
        // apply to signed data; the linear bins are intentional.
        histogram_view(
            "rbp.slumbot.hand_bb",
            &[
                -200.0, -150.0, -100.0, -50.0, -20.0, -10.0, -5.0, -2.0, -1.0, 0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0,
                100.0, 200.0,
            ],
        ),
        // Subgame decision wall-clock — budget-capped at
        // SubgameHyperParams::timeout_ms (default 5000). 2^0 (1ms) → 2^14
        // (~16s).
        log2_view("rbp.subgame.decision_ms", 0, 14),
        // K-means iteration wall-clock — single iter typically <60s, full
        // street caps ~30min. 2^7 (128ms) → 2^21 (~2M = ~35min).
        log2_view("rbp.kmeans.iteration_ms", 7, 21),
        // K-means phase wall-clock — init/iterate dominate, others quick.
        // 2^7 (128ms) → 2^23 (~8M = ~2h).
        log2_view("rbp.kmeans.phase_ms", 7, 23),
        // K-means cluster sizes — N varies wildly by street (preflop ~1k,
        // turn ~1.3M). 2^4 (16) → 2^24 (~16M).
        log2_view("rbp.kmeans.cluster_size", 4, 24),
        // K-means per-cluster drift values — drift typically descends
        // from ~0.1 toward ~1e-5 over iterations. 2^-20 (~1e-6) → 2^4 (16).
        log2_view("rbp.kmeans.drift_dist", -20, 4),
        // MCCFR flush duration — periodic DB-snapshot wall-clock.
        // Empirically unimodal centered ~14s, all observations 10-30s
        // (167K-infoset blueprint). Linear 5s bins from 5s to 60s give a
        // clean heatmap shape; log2 would compress the visible mode into
        // one bucket. Stay linear.
        histogram_view(
            "rbp.mccfr.flush_duration_ms",
            &[
                5_000.0, 10_000.0, 15_000.0, 20_000.0, 25_000.0, 30_000.0, 35_000.0, 40_000.0, 45_000.0, 50_000.0,
                55_000.0, 60_000.0,
            ],
        ),
        // MCCFR tree size — depends on game tree depth and pruning. Hold'em
        // batches typically produce trees of 1k-100k nodes. 2^4 (16) →
        // 2^20 (~1M).
        log2_view("rbp.mccfr.tree_size", 4, 20),
        // MCCFR infoset size — most infosets are small (1-5 nodes); long
        // tail of pathologically big infosets is the diagnostic. 2^0 (1) →
        // 2^14 (~16k).
        log2_view("rbp.mccfr.infoset_size", 0, 14),
        // MCCFR infosets per tree — same magnitude as tree_size.
        log2_view("rbp.mccfr.infosets_per_tree", 4, 17),
    ];
    views.into_iter().flatten().collect()
}

fn histogram_view(name: &str, boundaries: &[f64]) -> Option<Box<dyn View>> {
    new_view(
        Instrument::new().name(name.to_owned()),
        Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
            boundaries: boundaries.to_vec(),
            record_min_max: true,
        }),
    )
    .ok()
}

/// Builds an explicit-bucket histogram view with power-of-two boundaries
/// `[2^min_exp, 2^(min_exp+1), …, 2^max_exp]` (inclusive on both ends).
///
/// Use this for any quantity that spans more than ~1 decade of magnitude.
/// Each adjacent pair of buckets covers a doubling, so plotting the
/// resulting heatmap on a linear y-axis renders band heights proportional
/// to bucket widths — no arbitrary "1, 2, 5, 10" cutoffs to justify, and
/// the eye reads density correctly without a log-scale axis.
///
/// `min_exp` and `max_exp` are i32 so negative exponents (sub-unit
/// fractions, e.g. drift distance from 2^-20) are expressible.
fn log2_view(name: &str, min_exp: i32, max_exp: i32) -> Option<Box<dyn View>> {
    let boundaries = (min_exp..=max_exp).map(|e| (e as f64).exp2()).collect::<Vec<_>>();
    histogram_view(name, &boundaries)
}

fn service_name() -> String {
    std::env::var("OTEL_SERVICE_NAME")
        .ok()
        .or_else(|| std::env::var("CARGO_BIN_NAME").ok())
        .unwrap_or_else(|| "rbp".to_string())
}

fn build_resource(service: &str) -> Resource {
    let mut attrs = vec![
        KeyValue::new(semcov::resource::SERVICE_NAME, service.to_string()),
        KeyValue::new(semcov::resource::SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
    ];
    if let Ok(raw) = std::env::var("OTEL_RESOURCE_ATTRIBUTES") {
        attrs.extend(raw.split(',').filter_map(parse_kv));
    }
    Resource::new(attrs)
}

fn parse_kv(raw: &str) -> Option<KeyValue> {
    let mut it = raw.splitn(2, '=');
    let k = it.next()?.trim().to_string();
    let v = it.next()?.trim().to_string();
    (!k.is_empty() && !v.is_empty()).then(|| KeyValue::new(k, v))
}

static INITIALIZED: OnceLock<()> = OnceLock::new();
