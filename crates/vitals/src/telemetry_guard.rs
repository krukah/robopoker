use super::*;

/// Telemetry lifetime. Flushes exporters on drop. Hold this in `main()` for
/// the full binary lifetime.
pub struct TelemetryGuard {
    meter: Option<SdkMeterProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(m) = self.meter_mut().take() {
            let _: Result<(), _> = m.shutdown();
        }
    }
}

impl TelemetryGuard {
    /// Initialize the global telemetry stack. Returns a guard that flushes on drop.
    ///
    /// Must be called inside a tokio runtime (the OTLP exporters schedule background
    /// work on it). The four rbp binaries are `#[tokio::main]` so this holds.
    pub fn init() -> TelemetryGuard {
        let service = TelemetryGuard::service_name();
        let endpoint =
            std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_else(|_| DEFAULT_OTLP_ENDPOINT.to_string());
        let disabled = std::env::var("RBP_TELEMETRY_DISABLED")
            .is_ok_and(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"));
        let resource = TelemetryGuard::build_resource(&service);
        let meter_provider = if disabled {
            None
        } else {
            TelemetryGuard::install_meter(&endpoint, resource.clone())
        };
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
        metrics::Handles::install(&global::meter("rbp"));
        if INITIALIZED.set(()).is_ok() {
            if disabled {
                tracing::info!(service = %service, "telemetry initialized (otlp disabled)");
            } else {
                tracing::info!(service = %service, endpoint = %endpoint, "telemetry initialized");
            }
        }
        TelemetryGuard { meter: meter_provider }
    }

    fn meter_mut(&mut self) -> &mut Option<SdkMeterProvider> {
        &mut self.meter
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
        for view in TelemetryGuard::histogram_views() {
            builder = builder.with_view(view);
        }
        Some(builder.build())
    }

    /// Buckets for a signed bb/hand histogram. Every benchmark service deals the
    /// same 200-BB game, so they all share one boundary set — keeping it a
    /// function means a new opponent cannot quietly land on the default
    /// `[0..10000]` view and collapse every loss into the first bucket.
    fn signed_bb_view(metric: &str) -> Option<Box<dyn View>> {
        TelemetryGuard::histogram_view(
            metric,
            &[
                -200.0, -150.0, -100.0, -50.0, -20.0, -10.0, -5.0, -2.0, -1.0, 0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0,
                100.0, 200.0,
            ],
        )
    }

    /// Explicit bucket boundaries for histograms whose data ranges don't fit the OpenTelemetry
    /// default of `[0, 5, …, 10000, +Inf]`.
    fn histogram_views() -> Vec<Box<dyn View>> {
        let views = [
            // Subgame CFR iterations per decision — observed 5k–500k on realtime variants.
            TelemetryGuard::log2_view("rbp.subgame.iterations", 7, 20),
            // L1 distance between distributions — bounded 0..=2, so linear. The cliff of
            // interest is 0 (refined ≈ blueprint) vs 2 (fully disjoint).
            TelemetryGuard::histogram_view(
                "rbp.subgame.policy_deviation",
                &[0.0, 0.05, 0.1, 0.2, 0.3, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0],
            ),
            // regret / pot — observed 0.5 to 2000+ in smokes; spans converged and diverged.
            TelemetryGuard::log2_view("rbp.subgame.relative_regret", -7, 17),
            // bb/hand against an external opponent — signed (stack depth ~200 bb),
            // so log2 doesn't apply. One view per benchmark service.
            TelemetryGuard::signed_bb_view("rbp.slumbot.hand_bb"),
            TelemetryGuard::signed_bb_view("rbp.gtowizard.hand_bb"),
            // Subgame decision wall-clock — capped at SubgameHyperParams::timeout_ms.
            TelemetryGuard::log2_view("rbp.subgame.decision_ms", 0, 14),
            // K-means iteration wall-clock — typically <60s, full street caps ~30min.
            TelemetryGuard::log2_view("rbp.kmeans.iteration_ms", 7, 21),
            // K-means phase wall-clock — init/iterate dominate, up to ~2h.
            TelemetryGuard::log2_view("rbp.kmeans.phase_ms", 7, 23),
            // K-means cluster sizes — N varies by street (preflop ~1k, turn ~1.3M).
            TelemetryGuard::log2_view("rbp.kmeans.cluster_size", 4, 24),
            // K-means per-cluster drift — descends from ~0.1 toward ~1e-5 over iterations.
            TelemetryGuard::log2_view("rbp.kmeans.drift_dist", -20, 4),
            // Periodic DB-snapshot wall-clock — unimodal ~14s, observations 10-30s on a
            // 167K-infoset blueprint. Stay linear: log2 would compress the mode into one bucket.
            TelemetryGuard::histogram_view(
                "rbp.mccfr.flush_duration_ms",
                &[
                    5_000.0, 10_000.0, 15_000.0, 20_000.0, 25_000.0, 30_000.0, 35_000.0, 40_000.0, 45_000.0, 50_000.0,
                    55_000.0, 60_000.0,
                ],
            ),
            // MCCFR tree size — Hold'em batches typically produce 1k-100k nodes.
            TelemetryGuard::log2_view("rbp.mccfr.tree_size", 4, 20),
            // MCCFR infoset size — most are 1-5 nodes; the long tail is the diagnostic.
            TelemetryGuard::log2_view("rbp.mccfr.infoset_size", 0, 14),
            // MCCFR infosets per tree — same magnitude as tree_size.
            TelemetryGuard::log2_view("rbp.mccfr.infosets_per_tree", 4, 17),
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

    /// Explicit-bucket histogram view with power-of-two boundaries
    /// `[2^min_exp, …, 2^max_exp]`, inclusive on both ends.
    ///
    /// Each bucket pair covers a doubling, so a linear-y heatmap renders band
    /// heights proportional to bucket widths. Exponents are signed so sub-unit
    /// fractions (e.g. drift from 2^-20) are expressible.
    fn log2_view(name: &str, min_exp: i32, max_exp: i32) -> Option<Box<dyn View>> {
        let boundaries = (min_exp..=max_exp).map(|e| (e as f64).exp2()).collect::<Vec<_>>();
        TelemetryGuard::histogram_view(name, &boundaries)
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
            attrs.extend(raw.split(',').filter_map(TelemetryGuard::parse_kv));
        }
        Resource::new(attrs)
    }

    fn parse_kv(raw: &str) -> Option<KeyValue> {
        let mut it = raw.splitn(2, '=');
        let k = it.next()?.trim().to_string();
        let v = it.next()?.trim().to_string();
        (!k.is_empty() && !v.is_empty()).then(|| KeyValue::new(k, v))
    }
}
