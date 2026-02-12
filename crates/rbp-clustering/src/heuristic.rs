use super::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_transport::*;
use std::collections::BTreeMap;

/// Greedy heuristic for optimal transport (bipartite matching).
///
/// Iteratively pairs each source to its nearest available target, moving
/// as much mass as possible. This is O(N × M) but produces a suboptimal
/// coupling in many cases.
///
/// # Algorithm
///
/// For each source with remaining mass:
/// 1. Find nearest target with remaining capacity
/// 2. Transfer min(source_mass, target_capacity)
/// 3. Update remaining masses and continue
///
/// # Limitations
///
/// This greedy approach can be arbitrarily far from optimal EMD, even in
/// trivial 1D cases. It's provided primarily for benchmarking against
/// Sinkhorn, not for production use.
///
/// # Allocation
///
/// Currently uses `BTreeMap` for the transport plan, which incurs allocation
/// overhead. A zero-allocation version would improve performance for the
/// billions of EMD computations during clustering.
pub struct Heuristic<'a> {
    /// Transport plan mapping pairs to flow amount.
    plan: BTreeMap<Pair, Probability>,
    /// Ground metric for distances.
    metric: &'a Metric,
    /// Source distribution.
    source: &'a Histogram,
    /// Target distribution.
    target: &'a Histogram,
}

impl Coupling for Heuristic<'_> {
    type X = Abstraction;
    type Y = Abstraction;
    type P = Potential;
    type Q = Potential;
    type M = Metric;

    fn cost(&self) -> Probability {
        self.plan.values().sum()
    }
    fn flow(&self, x: &Self::X, y: &Self::Y) -> Probability {
        let ref index = Pair::from((x, y));
        self.plan
            .get(index)
            .copied()
            .expect("missing in transport plan")
    }
    fn minimize(mut self) -> Self {
        self.plan.clear();
        let ref mut pile = Potential::derive(self.source);
        let ref mut sink = Potential::derive(self.target);
        'cost: while pile.values().any(|v| v > 0.) {
            'pile: for x in pile
                .support()
                .filter(|x| pile.density(x) > 0.)
                .collect::<Vec<_>>()
            {
                match sink
                    .support()
                    .filter(|y| sink.density(y) > 0.)
                    .map(|y| (y, self.metric.distance(&x, &y)))
                    .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
                {
                    None => break 'cost,
                    Some((y, distance)) => {
                        let dx = pile.density(&x);
                        let dy = sink.density(&y);
                        let mass = Probability::min(dx, dy);
                        let pair = Pair::from((&x, &y));
                        pile.increment(&x, -mass);
                        sink.increment(&y, -mass);
                        *self.plan.entry(pair).or_default() += mass * distance;
                        continue 'pile;
                    }
                }
            }
        }
        self
    }
}

impl<'a> From<(&'a Histogram, &'a Histogram, &'a Metric)> for Heuristic<'a> {
    fn from((source, target, metric): (&'a Histogram, &'a Histogram, &'a Metric)) -> Self {
        Self {
            plan: BTreeMap::default(),
            metric,
            source,
            target,
        }
    }
}
