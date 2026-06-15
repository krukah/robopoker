//! One subgame solve's output — refined policy and per-edge visit counts,
//! plus the (iterations / elapsed / regret) triple used for telemetry.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::time::Duration;

use cowboys::Edge;
use endgame::SubgameHyperParams;
use holdem::NlheEdge;
use holdem::NlheInfo;
use kicker::Street;
use mccfr::Harvest;
use mccfr::Solver;
use pokerkit::Chips;
use pokerkit::Probability;
use pokerkit::Utility;
use vitals::KeyValue;

use super::Tag;

pub struct Solved {
    iterations: usize,
    elapsed: Duration,
    /// `Σ_a max(0, R(info, a))` at the decision infoset (partition-summed
    /// for `world` / full variants). Chips. Divide by total visits at info
    /// for per-iteration regret — `Brain::distrib` does that and further
    /// divides by pot for the pot-relative settledness signal recorded as
    /// `subgame_relative_regret`.
    regret: Utility,
    policy: BTreeMap<Edge, Probability>,
    visits: BTreeMap<Edge, u32>,
}

impl Solved {
    /// Run the full solve pipeline — `spend` the deadline, `harvest` the
    /// result at `info`, choice-filter, and assemble. The 3 subgame
    /// `Brain::solve` overrides differ only in which `adapt_*` they pass
    /// in; everything else collapses to one call here.
    pub fn run<S>(mut solver: S, info: NlheInfo, deadline: Duration) -> Self
    where
        S: Solver + Harvest<Base = NlheInfo, Edge = NlheEdge>,
    {
        let (iterations, elapsed) = solver.spend(deadline);
        let harvest = solver.harvest(info);
        let policy = harvest
            .refined
            .into_iter()
            .filter(|(e, _)| e.is_choice())
            .map(|(e, p)| (Edge::from(e), p))
            .collect();
        let visits = harvest
            .visits
            .into_iter()
            .filter(|(e, _)| e.is_choice())
            .map(|(e, v)| (Edge::from(e), v))
            .collect();
        Self {
            iterations,
            elapsed,
            regret: harvest.regret,
            policy,
            visits,
        }
    }

    pub fn iterations(&self) -> usize {
        self.iterations
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn regret(&self) -> Utility {
        self.regret
    }

    pub fn policy(&self) -> &BTreeMap<Edge, Probability> {
        &self.policy
    }

    pub fn visits(&self) -> &BTreeMap<Edge, u32> {
        &self.visits
    }

    /// Emit the postflop solver-cell metrics + tracing. Called once per
    /// postflop decision after `solve` returns.
    pub fn emit_postflop(&self, tag: Tag, street: Street, pot: Chips, relative: Utility) {
        let labels = with_street(tag, street);
        let metrics = vitals::metrics::get();
        metrics.subgame_decisions.add(1, &labels);
        metrics.subgame_iterations.record(self.iterations as u64, &labels);
        metrics
            .subgame_decision_ms
            .record(self.elapsed.as_secs_f64() * 1000.0, &labels);
        metrics.subgame_relative_regret.record(relative as f64, &labels);
        tracing::debug!(
            variant = tag.label,
            street = %street,
            iterations = self.iterations,
            elapsed_ms = self.elapsed.as_millis() as u64,
            regret = self.regret as f64,
            pot = pot as i64,
            relative = relative as f64,
            "subgame decision",
        );
    }

    /// Visits-blend `self.refined` against a `blueprint` prior, emit the
    /// extraction-side telemetry, and return the final policy. Wrapping
    /// the whole pipeline here keeps `Brain::distrib` a thin orchestrator.
    pub fn extract(
        &self,
        policy: &BTreeMap<Edge, Probability>,
        tag: Tag,
        street: Street,
    ) -> BTreeMap<Edge, Probability> {
        let total: u64 = self.visits.values().map(|&v| v as u64).sum();
        tracing::info_span!("subgame.extract", variant = tag.label, total_visits = total).in_scope(|| {
            let blend = self.blend(policy);
            self.emit_verify(tag, policy, &blend);
            self.emit_extraction_stats(tag, street, policy, &blend, total);
            blend
        })
    }

    /// Per-edge visits-weighted convex mix of `self.refined` and `blueprint`:
    /// `w(a) = v(a) / (v(a) + threshold)`,
    /// `π(a) = w·refined + (1-w)·blueprint`, then renormalised. High-visit
    /// edges trust the subgame; low-visit edges fall back to blueprint. The
    /// only blend strategy — pure-blueprint = use [`Blueprint`](super::Blueprint)
    /// directly without a subgame layer.
    fn blend(&self, policy: &BTreeMap<Edge, Probability>) -> BTreeMap<Edge, Probability> {
        let threshold = SubgameHyperParams::get().visit_threshold() as Probability;
        let edges = policy
            .keys()
            .chain(self.policy.keys())
            .copied()
            .collect::<BTreeSet<_>>();
        let raw = edges
            .into_iter()
            .map(|e| {
                let v = self.visits.get(&e).copied().unwrap_or(0) as Probability;
                let w = v / (v + threshold);
                let sg = self.policy.get(&e).copied().unwrap_or(0.0);
                let bp = policy.get(&e).copied().unwrap_or(0.0);
                (e, w * sg + (1.0 - w) * bp)
            })
            .collect::<Vec<_>>();
        let total = raw.iter().map(|(_, p)| p).sum::<Probability>().max(pokerkit::EPSILON);
        raw.into_iter().map(|(e, p)| (e, p / total)).collect()
    }

    /// Diagnostic: log blueprint and subgame-refined policy side-by-side
    /// for a single postflop decision. Emitted at `trace!` — workspace
    /// default filter is `info,rbp=debug`, so this stays off in production
    /// until enabled with `RUST_LOG=parlor=trace`.
    fn emit_verify(&self, tag: Tag, blueprint: &BTreeMap<Edge, Probability>, refined: &BTreeMap<Edge, Probability>) {
        let fmt = |m: &BTreeMap<Edge, Probability>| -> String {
            m.iter()
                .map(|(e, p)| format!("{e}={p:.3}"))
                .collect::<Vec<_>>()
                .join(" ")
        };
        tracing::trace!(
            variant = tag.label,
            blueprint = %fmt(blueprint),
            refined = %fmt(refined),
            "VERIFY refined vs blueprint",
        );
    }

    /// Emit `subgame_policy_deviation` (L1 distance refined↔blueprint).
    fn emit_extraction_stats(
        &self,
        tag: Tag,
        street: Street,
        blueprint: &BTreeMap<Edge, Probability>,
        refined: &BTreeMap<Edge, Probability>,
        _total_visits: u64,
    ) {
        let labels = with_street(tag, street);
        let metrics = vitals::metrics::get();
        let edges = blueprint.keys().chain(refined.keys()).copied().collect::<BTreeSet<_>>();
        let l1 = edges
            .iter()
            .map(|e| {
                let b = blueprint.get(e).copied().unwrap_or(0.0);
                let r = refined.get(e).copied().unwrap_or(0.0);
                (b - r).abs()
            })
            .sum::<Probability>();
        metrics.subgame_policy_deviation.record(l1 as f64, &labels);
    }
}

/// Subgame metric label set: `variant` + `street`. The cube axes
/// (`depth`/`world`/`dirac`) are intentionally omitted — they are
/// determined by `variant`, so emitting both triples Prometheus
/// series count for no PromQL recoverability gain.
fn with_street(tag: Tag, street: Street) -> [KeyValue; 2] {
    [
        KeyValue::new("variant", tag.label),
        KeyValue::new("street", street.to_string()),
    ]
}
