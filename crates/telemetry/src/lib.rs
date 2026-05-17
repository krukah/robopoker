//! Telemetry init and metric registry for rbp binaries.
//!
//! One call — `let _guard = rbp_telemetry::init();` — installs a `tracing`
//! subscriber, routes `log::*` calls through it via the `tracing-log` shim,
//! configures OTLP exporters for metrics and traces, and pre-registers the
//! workspace's metric handles under `metrics::*`.
//!
//! Environment variables (all optional):
//!   OTEL_EXPORTER_OTLP_ENDPOINT  default http://localhost:4317 (gRPC)
//!   OTEL_SERVICE_NAME            default inferred from CARGO_BIN_NAME
//!   OTEL_RESOURCE_ATTRIBUTES     extra k=v pairs merged into resource
//!   RUST_LOG                     default `info,rbp=debug`
//!   RBP_TELEMETRY_DISABLED       if set and truthy, skip OTLP; fmt layer only
//!   RBP_TELEMETRY_TRACE_RATIO    head sampler ratio in [0.0, 1.0]; default 1.0

use std::sync::OnceLock;
use std::time::Duration;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
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
use opentelemetry_sdk::trace::BatchSpanProcessor;
use opentelemetry_sdk::trace::Sampler;
use opentelemetry_sdk::trace::TracerProvider;
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
    tracer: Option<TracerProvider>,
    meter: Option<SdkMeterProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(t) = self.tracer.take() {
            let _: Result<(), _> = t.shutdown();
        }
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
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_OTLP_ENDPOINT.to_string());
    let disabled = std::env::var("RBP_TELEMETRY_DISABLED")
        .ok()
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    let resource = build_resource(&service);
    let (tracer_provider, meter_provider) = if disabled {
        (None, None)
    } else {
        (
            install_tracer(&endpoint, resource.clone()),
            install_meter(&endpoint, resource.clone()),
        )
    };
    let _ = tracing_log::LogTracer::init();
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(false)
        .with_writer(std::io::stderr);
    let otel_layer = tracer_provider
        .as_ref()
        .map(|tp| tracing_opentelemetry::layer().with_tracer(tp.tracer(service.clone())));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .try_init();
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
    TelemetryGuard {
        tracer: tracer_provider,
        meter: meter_provider,
    }
}

fn install_tracer(endpoint: &str, resource: Resource) -> Option<TracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(3))
        .build()
        .ok()?;
    let batch = BatchSpanProcessor::builder(exporter, runtime::Tokio).build();
    let provider = TracerProvider::builder()
        .with_resource(resource)
        .with_sampler(sampler_from_env())
        .with_span_processor(batch)
        .build();
    global::set_tracer_provider(provider.clone());
    Some(provider)
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
    let mut builder = SdkMeterProvider::builder()
        .with_resource(resource)
        .with_reader(reader);
    for view in histogram_views() {
        builder = builder.with_view(view);
    }
    Some(builder.build())
}

/// Explicit bucket boundaries for histograms whose data ranges don't fit the
/// OpenTelemetry default of `[0, 5, 10, 25, 50, 75, 100, 250, 500, 750, 1000,
/// 2500, 5000, 7500, 10000, +Inf]`. Without these, data above 10000 collapses
/// into the `+Inf` bucket and quantile estimates become meaningless.
fn histogram_views() -> Vec<Box<dyn View>> {
    let views = [
        // Subgame CFR iteration count per decision — observed 5k–500k on
        // realtime variants.
        histogram_view(
            "rbp.subgame.iterations",
            &[
                100.0,
                500.0,
                1000.0,
                5000.0,
                10000.0,
                25000.0,
                50000.0,
                100000.0,
                250000.0,
                500000.0,
                1_000_000.0,
            ],
        ),
        // L1 distance between two probability distributions — bounded 0..=2.
        histogram_view(
            "rbp.subgame.policy_deviation",
            &[
                0.0, 0.05, 0.1, 0.2, 0.3, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0,
            ],
        ),
        // regret / pot — observed 0.5 to 2000+ in smokes. Log-ish buckets so
        // we have resolution at both the "converged" and "diverged" ends.
        histogram_view(
            "rbp.subgame.relative_regret",
            &[0.01, 0.1, 1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0],
        ),
        // bb/hand on Slumbot — signed, stack depth ~200 bb. Default OTel
        // buckets are positive-only; custom covers loss cases.
        histogram_view(
            "rbp.slumbot.hand_bb",
            &[
                -200.0, -150.0, -100.0, -50.0, -20.0, -10.0, -5.0, -2.0, -1.0, 0.0, 1.0, 2.0, 5.0,
                10.0, 20.0, 50.0, 100.0, 200.0,
            ],
        ),
        // Subgame decision wall-clock — budget-capped at SubgameHyperParams::timeout_ms (default 5000).
        histogram_view(
            "rbp.subgame.decision_ms",
            &[
                1.0, 10.0, 100.0, 500.0, 1000.0, 2000.0, 3000.0, 4000.0, 5000.0, 6000.0, 10000.0,
            ],
        ),
        // K-means iteration wall-clock — single iter typically <60s, full street caps ~30min.
        histogram_view(
            "rbp.kmeans.iteration_ms",
            &[
                100.0,
                500.0,
                1000.0,
                5000.0,
                10000.0,
                30000.0,
                60000.0,
                300000.0,
                1_800_000.0,
            ],
        ),
        // K-means phase wall-clock — same upper bound; init/iterate dominate, others are quick.
        histogram_view(
            "rbp.kmeans.phase_ms",
            &[
                100.0,
                1000.0,
                10000.0,
                60000.0,
                300000.0,
                1_800_000.0,
                7_200_000.0,
            ],
        ),
        // K-means cluster sizes — N varies wildly by street (preflop ~1k, turn ~1.3M).
        histogram_view(
            "rbp.kmeans.cluster_size",
            &[
                10.0,
                100.0,
                1000.0,
                10000.0,
                100000.0,
                1_000_000.0,
                10_000_000.0,
            ],
        ),
        // K-means per-cluster drift values — log-spaced; drift typically
        // descends from ~0.1 toward ~1e-5 over iterations.
        histogram_view(
            "rbp.kmeans.drift_dist",
            &[1e-6, 1e-5, 1e-4, 1e-3, 1e-2, 1e-1, 1.0, 10.0],
        ),
        // MCCFR flush duration — periodic DB-snapshot wall-clock.
        // Empirically a unimodal distribution centered at ~14s, all
        // observations in 10-30s (167K-infoset blueprint). Linear 5s
        // bins from 5s to 60s give a clean heatmap shape and ample
        // headroom; if flushes ever break 60s we'll see them in +Inf
        // and extend.
        histogram_view(
            "rbp.mccfr.flush_duration_ms",
            &[
                5_000.0, 10_000.0, 15_000.0, 20_000.0, 25_000.0, 30_000.0, 35_000.0, 40_000.0,
                45_000.0, 50_000.0, 55_000.0, 60_000.0,
            ],
        ),
        // MCCFR tree size — log-spaced; depends on game tree depth and
        // pruning. Hold'em batches typically produce trees of 1k-100k
        // nodes; spikes flag tree-depth pathologies.
        histogram_view(
            "rbp.mccfr.tree_size",
            &[10.0, 100.0, 1000.0, 10_000.0, 100_000.0, 1_000_000.0],
        ),
        // MCCFR infoset size — most infosets are small (1-5 nodes); long
        // tail of pathologically big infosets is the diagnostic of
        // interest.
        histogram_view(
            "rbp.mccfr.infoset_size",
            &[1.0, 2.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 10_000.0],
        ),
        // MCCFR infosets per tree — same magnitude as tree_size.
        histogram_view(
            "rbp.mccfr.infosets_per_tree",
            &[10.0, 100.0, 1000.0, 10_000.0, 100_000.0],
        ),
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

fn sampler_from_env() -> Sampler {
    let ratio = std::env::var("RBP_TELEMETRY_TRACE_RATIO")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    if ratio >= 1.0 {
        Sampler::AlwaysOn
    } else if ratio <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio)))
    }
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
