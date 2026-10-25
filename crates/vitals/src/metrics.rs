//! Pre-registered metric handles.
//!
//! Central ownership prevents typo-induced cardinality explosions and gives us a single audit
//! surface for what's emitted. Add new handles here rather than minting instruments at call
//! sites.
use super::*;

use std::sync::OnceLock;

use opentelemetry::global;
use opentelemetry::metrics::Counter;
use opentelemetry::metrics::Gauge;
use opentelemetry::metrics::Histogram;
use opentelemetry::metrics::Meter;

static HANDLES: OnceLock<Handles> = OnceLock::new();

/// Each row is `<namespace>_<metric>: <Kind><Type>`. Namespace prefix
/// groups vertically; the `Counter` / `Gauge` / `Histogram` column tells
/// you the PromQL shape at a glance. Within each namespace, fields are
/// sorted by kind (Counter → Gauge → Histogram). `#[rustfmt::skip]`
/// preserves the alignment.
#[rustfmt::skip]
pub struct Handles {
    mccfr_steps: Counter   <u64>,
    mccfr_nodes: Counter   <u64>,
    mccfr_infos: Counter   <u64>,
    mccfr_sum_regret: Gauge     <f64>,
    mccfr_flush_duration_ms: Histogram <f64>,
    mccfr_tree_size: Histogram <u64>,
    mccfr_infoset_size: Histogram <u64>,
    mccfr_infosets_per_tree: Histogram <u64>,
    kmeans_iterations: Counter   <u64>,
    kmeans_early_terminated: Counter   <u64>,
    kmeans_drift_max: Gauge     <f64>,
    kmeans_reassignment: Gauge     <f64>,
    kmeans_iteration_ms: Histogram <f64>,
    kmeans_phase_ms: Histogram <f64>,
    kmeans_cluster_size: Histogram <u64>,
    kmeans_drift_dist: Histogram <f64>,
    subgame_decisions: Counter   <u64>,
    subgame_decision_ms: Histogram <f64>,
    subgame_iterations: Histogram <u64>,
    subgame_relative_regret: Histogram <f64>,
    subgame_policy_deviation: Histogram <f64>,
    http_requests: Counter   <u64>,
    http_duration_ms: Histogram <f64>,
    db_queries: Counter   <u64>,
    db_query_ms: Histogram <f64>,
    slumbot: Bench,
    gtowizard: Bench,
}

impl Handles {
    pub fn mccfr_steps(&self) -> &Counter<u64> {
        &self.mccfr_steps
    }

    pub fn mccfr_nodes(&self) -> &Counter<u64> {
        &self.mccfr_nodes
    }

    pub fn mccfr_infos(&self) -> &Counter<u64> {
        &self.mccfr_infos
    }

    pub fn mccfr_sum_regret(&self) -> &Gauge<f64> {
        &self.mccfr_sum_regret
    }

    pub fn mccfr_flush_duration_ms(&self) -> &Histogram<f64> {
        &self.mccfr_flush_duration_ms
    }

    pub fn mccfr_tree_size(&self) -> &Histogram<u64> {
        &self.mccfr_tree_size
    }

    pub fn mccfr_infoset_size(&self) -> &Histogram<u64> {
        &self.mccfr_infoset_size
    }

    pub fn mccfr_infosets_per_tree(&self) -> &Histogram<u64> {
        &self.mccfr_infosets_per_tree
    }

    pub fn kmeans_iterations(&self) -> &Counter<u64> {
        &self.kmeans_iterations
    }

    pub fn kmeans_early_terminated(&self) -> &Counter<u64> {
        &self.kmeans_early_terminated
    }

    pub fn kmeans_drift_max(&self) -> &Gauge<f64> {
        &self.kmeans_drift_max
    }

    pub fn kmeans_reassignment(&self) -> &Gauge<f64> {
        &self.kmeans_reassignment
    }

    pub fn kmeans_iteration_ms(&self) -> &Histogram<f64> {
        &self.kmeans_iteration_ms
    }

    pub fn kmeans_phase_ms(&self) -> &Histogram<f64> {
        &self.kmeans_phase_ms
    }

    pub fn kmeans_cluster_size(&self) -> &Histogram<u64> {
        &self.kmeans_cluster_size
    }

    pub fn kmeans_drift_dist(&self) -> &Histogram<f64> {
        &self.kmeans_drift_dist
    }

    pub fn subgame_decisions(&self) -> &Counter<u64> {
        &self.subgame_decisions
    }

    pub fn subgame_decision_ms(&self) -> &Histogram<f64> {
        &self.subgame_decision_ms
    }

    pub fn subgame_iterations(&self) -> &Histogram<u64> {
        &self.subgame_iterations
    }

    pub fn subgame_relative_regret(&self) -> &Histogram<f64> {
        &self.subgame_relative_regret
    }

    pub fn subgame_policy_deviation(&self) -> &Histogram<f64> {
        &self.subgame_policy_deviation
    }

    pub fn http_requests(&self) -> &Counter<u64> {
        &self.http_requests
    }

    pub fn http_duration_ms(&self) -> &Histogram<f64> {
        &self.http_duration_ms
    }

    pub fn db_queries(&self) -> &Counter<u64> {
        &self.db_queries
    }

    pub fn db_query_ms(&self) -> &Histogram<f64> {
        &self.db_query_ms
    }

    pub fn slumbot(&self) -> &Bench {
        &self.slumbot
    }

    pub fn gtowizard(&self) -> &Bench {
        &self.gtowizard
    }
}

impl Handles {
    pub fn get() -> &'static Handles {
        HANDLES.get_or_init(|| Handles::build(&global::meter("rbp")))
    }

    pub(crate) fn install(meter: &Meter) {
        let _ = HANDLES.set(Handles::build(meter));
    }

    fn build(meter: &Meter) -> Handles {
        Handles {
            mccfr_steps: meter
                .u64_counter("rbp.mccfr.steps")
                .with_description("MCCFR training steps completed (cumulative). Use rate() for steps/sec.")
                .build(),
            mccfr_nodes: meter
                .u64_counter("rbp.mccfr.nodes")
                .with_description("Game tree nodes explored (cumulative). Use rate() for nodes/sec.")
                .build(),
            mccfr_infos: meter
                .u64_counter("rbp.mccfr.infos")
                .with_description("Information sets visited (cumulative). Use rate() for infos/sec.")
                .build(),
            mccfr_sum_regret: meter
                .f64_gauge("rbp.mccfr.sum_regret")
                .with_description("Sum of accumulated regret (convergence signal, lower is better)")
                .build(),
            mccfr_flush_duration_ms: meter
                .f64_histogram("rbp.mccfr.flush_duration_ms")
                .with_description("Wall time of a periodic training-to-DB snapshot")
                .build(),
            mccfr_tree_size: meter
                .u64_histogram("rbp.mccfr.tree_size")
                .with_description(
                    "Number of nodes in each sampled training tree. One \
                     observation per tree per batch.",
                )
                .build(),
            mccfr_infoset_size: meter
                .u64_histogram("rbp.mccfr.infoset_size")
                .with_description(
                    "Number of nodes per infoset within a sampled tree (post \
                     walker filter). One observation per infoset per batch — \
                     the heavy tail flags pathological infosets driving \
                     sum_regret.",
                )
                .build(),
            mccfr_infosets_per_tree: meter
                .u64_histogram("rbp.mccfr.infosets_per_tree")
                .with_description(
                    "Distinct infosets per sampled tree (post walker filter). \
                     One observation per tree per batch.",
                )
                .build(),
            kmeans_drift_max: meter
                .f64_gauge("rbp.kmeans.drift_max")
                .with_description(
                    "Largest centroid movement during the last Elkan iteration. \
                     Approaches zero at convergence; spikes mean a cluster reseeded.",
                )
                .build(),
            kmeans_iteration_ms: meter
                .f64_histogram("rbp.kmeans.iteration_ms")
                .with_description("Wall time of a single Elkan iteration (labeled by street)")
                .build(),
            kmeans_phase_ms: meter
                .f64_histogram("rbp.kmeans.phase_ms")
                .with_description(
                    "Wall time per clustering phase. Phases: hydrate, init, bound, \
                     iterate, lookup, metric, future.",
                )
                .build(),
            kmeans_cluster_size: meter
                .u64_histogram("rbp.kmeans.cluster_size")
                .with_description(
                    "Points-per-cluster at convergence (one record per cluster). \
                     Distribution skew flags imbalanced abstractions.",
                )
                .build(),
            kmeans_iterations: meter
                .u64_counter("rbp.kmeans.iterations")
                .with_description("K-means iterations completed (cumulative). Use rate() for iters/sec.")
                .build(),
            kmeans_early_terminated: meter
                .u64_counter("rbp.kmeans.early_terminated")
                .with_description(
                    "Clustering runs that hit KmeansHyperParams::drift_threshold before \
                     exhausting Street::t() iterations. Increment per street that triggered \
                     early stop.",
                )
                .build(),
            kmeans_reassignment: meter
                .f64_gauge("rbp.kmeans.reassignment")
                .with_description(
                    "Fraction of points whose assigned cluster changed since the \
                     previous iteration. 0.0 = fully stable, 1.0 = every point moved. \
                     Complementary convergence signal to drift_max (centroid motion).",
                )
                .build(),
            kmeans_drift_dist: meter
                .f64_histogram("rbp.kmeans.drift_dist")
                .with_description(
                    "Distribution of per-cluster drift values within one iteration \
                     (K=128/144 records per iter). Heatmap exposes which clusters \
                     are still moving vs which froze; drift_max only tells you the \
                     worst offender.",
                )
                .build(),
            subgame_decisions: meter
                .u64_counter("rbp.subgame.decisions")
                .with_description("Subgame decisions taken")
                .build(),
            subgame_decision_ms: meter
                .f64_histogram("rbp.subgame.decision_ms")
                .with_description("Wall time per subgame decision")
                .build(),
            subgame_iterations: meter
                .u64_histogram("rbp.subgame.iterations")
                .with_description("Solver iterations per subgame decision")
                .build(),
            subgame_relative_regret: meter
                .f64_histogram("rbp.subgame.relative_regret")
                .with_description("regret/pot at subgame decision time")
                .build(),
            subgame_policy_deviation: meter
                .f64_histogram("rbp.subgame.policy_deviation")
                .with_description(
                    "L1 distance between refined and blueprint policies per decision. \
                     Zero = subgame agreed with blueprint; 2 = fully disjoint.",
                )
                .build(),
            http_requests: meter
                .u64_counter("rbp.http.requests")
                .with_description("HTTP requests served")
                .build(),
            http_duration_ms: meter
                .f64_histogram("rbp.http.duration_ms")
                .with_description("HTTP request duration")
                .build(),
            db_queries: meter
                .u64_counter("rbp.db.queries")
                .with_description("Database queries executed")
                .build(),
            db_query_ms: meter
                .f64_histogram("rbp.db.query_ms")
                .with_description("Database query duration")
                .build(),
            slumbot: Bench::build(meter, "slumbot"),
            gtowizard: Bench::build(meter, "gtowizard"),
        }
    }
}
