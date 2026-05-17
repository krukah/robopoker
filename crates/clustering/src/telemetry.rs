//! Telemetry helpers for the clustering pipeline.
//!
//! Encapsulates the OTLP recording done by [`Layer::cluster`] so the
//! algorithm body stays readable. All recorders share a single
//! `{street}` label set built once at construction.

use rbp_cards::Street;
use rbp_telemetry::KeyValue;
use rbp_telemetry::metrics::Handles;
use std::time::Duration;
use std::time::Instant;

/// Single source of truth for phase names. Each constant is used as
/// both a `tracing` field value and a metric label value; aliasing
/// them here prevents the two from drifting.
pub(crate) mod phase {
    pub const HYDRATE: &str = "hydrate";
    pub const INIT: &str = "init";
    pub const BOUND: &str = "bound";
    pub const ITERATE: &str = "iterate";
    pub const LOOKUP: &str = "lookup";
    pub const METRIC: &str = "metric";
    pub const FUTURE: &str = "future";
}

/// Per-clustering-run telemetry recorder. Holds the street label and
/// dispatches every emit through pre-cached `Handles`.
pub(crate) struct ClusterTelemetry {
    handles: &'static Handles,
    labels: [KeyValue; 1],
}

impl ClusterTelemetry {
    pub(crate) fn new(street: Street) -> Self {
        Self {
            handles: rbp_telemetry::metrics::get(),
            labels: [KeyValue::new("street", format!("{}", street))],
        }
    }

    /// Records `phase_ms` for one named clustering phase.
    pub(crate) fn phase(&self, t0: Instant, name: &'static str) {
        self.handles.kmeans_phase_ms.record(
            t0.elapsed().as_secs_f64() * 1000.0,
            &[self.labels[0].clone(), KeyValue::new("phase", name)],
        );
    }

    /// Records iteration time, increments the iteration counter,
    /// records `drift_max`, and emits the full drift distribution
    /// (one histogram observation per cluster). `drifts` is folded
    /// from the values already produced by `step_elkan` for the
    /// bound update — recording is K sub-µs OTLP records, free
    /// relative to step_elkan's EMD work.
    pub(crate) fn iteration<const K: usize>(&self, elapsed: Duration, drift: &crate::Drift<K>) {
        self.handles
            .kmeans_iteration_ms
            .record(elapsed.as_secs_f64() * 1000.0, &self.labels);
        self.handles.kmeans_iterations.add(1, &self.labels);
        self.handles
            .kmeans_drift_max
            .record(drift.max() as f64, &self.labels);
        drift.as_array().iter().for_each(|&d| {
            self.handles
                .kmeans_drift_dist
                .record(d as f64, &self.labels)
        });
    }

    /// Records the per-iter reassignment fraction (fraction of points
    /// whose cluster assignment changed since the previous step).
    pub(crate) fn reassignment(&self, fraction: f64) {
        self.handles
            .kmeans_reassignment
            .record(fraction, &self.labels);
    }

    /// Increments the early-termination counter. Called when the loop
    /// breaks before exhausting `Street::t()` because drift fell below
    /// `KmeansHyperParams::drift_threshold`.
    pub(crate) fn early_terminated(&self) {
        self.handles.kmeans_early_terminated.add(1, &self.labels);
    }

    /// Records one `cluster_size` per cluster. Empty clusters are
    /// derivable from the histogram's `le=0` bucket, so no separate
    /// gauge is emitted.
    pub(crate) fn cluster_sizes(&self, sizes: &[u64]) {
        sizes
            .iter()
            .for_each(|&s| self.handles.kmeans_cluster_size.record(s, &self.labels));
    }
}
