//! Pre-registered metric handles.
//!
//! Central ownership prevents typo-induced cardinality explosions and gives
//! us a single audit surface for what's emitted. Add new handles here rather
//! than minting instruments at call sites.
//!
//! Lazy against the global meter: when the OTLP provider is installed before
//! first access, emissions flow. When no provider is installed (tests, local
//! dev without a collector) the global meter is a no-op and emissions are
//! dropped silently. Either way, callers never panic.

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
    // ── MCCFR training ──────────────────────────────────────────────────
    pub mccfr_steps:                Counter   <u64>,
    pub mccfr_nodes:                Counter   <u64>,
    pub mccfr_infos:                Counter   <u64>,
    pub mccfr_sum_regret:           Gauge     <f64>,
    pub mccfr_flush_duration_ms:    Histogram <f64>,
    pub mccfr_tree_size:            Histogram <u64>,
    pub mccfr_infoset_size:         Histogram <u64>,
    pub mccfr_infosets_per_tree:    Histogram <u64>,
    // ── K-means clustering ──────────────────────────────────────────────
    pub kmeans_iterations:          Counter   <u64>,
    pub kmeans_early_terminated:    Counter   <u64>,
    pub kmeans_drift_max:           Gauge     <f64>,
    pub kmeans_reassignment:        Gauge     <f64>,
    pub kmeans_iteration_ms:        Histogram <f64>,
    pub kmeans_phase_ms:            Histogram <f64>,
    pub kmeans_cluster_size:        Histogram <u64>,
    pub kmeans_drift_dist:          Histogram <f64>,
    // ── Subgame solver internals ────────────────────────────────────────
    pub subgame_decisions:          Counter   <u64>,
    pub subgame_decision_ms:        Histogram <f64>,
    pub subgame_iterations:         Histogram <u64>,
    pub subgame_relative_regret:    Histogram <f64>,
    pub subgame_policy_deviation:   Histogram <f64>,
    // ── HTTP server ─────────────────────────────────────────────────────
    pub http_requests:              Counter   <u64>,
    pub http_duration_ms:           Histogram <f64>,
    // ── Database ────────────────────────────────────────────────────────
    pub db_queries:                 Counter   <u64>,
    pub db_query_ms:                Histogram <f64>,
    // ── Slumbot benchmark ───────────────────────────────────────────────
    pub slumbot_hands:              Counter   <u64>,
    pub slumbot_hand_bb_won:        Counter   <f64>,
    pub slumbot_hand_bb_lost:       Counter   <f64>,
    pub slumbot_errors:             Counter   <u64>,
    pub slumbot_hand_bb:            Histogram <f64>,
}

pub fn get() -> &'static Handles {
    HANDLES.get_or_init(|| build(&global::meter("rbp")))
}

pub(crate) fn install(meter: &Meter) {
    let _ = HANDLES.set(build(meter));
}

fn build(meter: &Meter) -> Handles {
    Handles {
        mccfr_steps: meter
            .u64_counter("rbp.mccfr.steps")
            .with_description(
                "MCCFR training steps completed (cumulative). Use rate() for steps/sec.",
            )
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
            .with_description(
                "K-means iterations completed (cumulative). Use rate() for iters/sec.",
            )
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
        slumbot_hands: meter
            .u64_counter("rbp.slumbot.hands")
            .with_description("Slumbot hands played")
            .build(),
        slumbot_hand_bb: meter
            .f64_histogram("rbp.slumbot.hand_bb")
            .with_description(
                "Winnings per hand in big blinds (signed; use _bucket for distribution, not _sum)",
            )
            .build(),
        slumbot_hand_bb_won: meter
            .f64_counter("rbp.slumbot.hand_bb_won")
            .with_description(
                "Cumulative big blinds won (positive side only). Paired with \
                 hand_bb_lost, the signed sum is `won - lost`; unlike a histogram's \
                 _sum this pair is monotonic so Prometheus rate()/increase() is safe.",
            )
            .build(),
        slumbot_hand_bb_lost: meter
            .f64_counter("rbp.slumbot.hand_bb_lost")
            .with_description(
                "Cumulative big blinds lost (absolute value of negative hand results). \
                 See hand_bb_won for mean-bb/100 computation.",
            )
            .build(),
        slumbot_errors: meter
            .u64_counter("rbp.slumbot.errors")
            .with_description("Slumbot session failures")
            .build(),
    }
}
