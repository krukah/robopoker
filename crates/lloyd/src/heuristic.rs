use super::*;
use monge::*;
use pokerkit::*;
use std::collections::BTreeMap;

/// Greedy heuristic for optimal transport (bipartite matching): repeatedly
/// move `min(source_mass, target_capacity)` from each source with mass left to
/// its nearest target with capacity left. O(N × M).
///
/// Can land arbitrarily far from optimal EMD, even in trivial 1D cases — this
/// exists to benchmark against Sinkhorn, not for production. The `BTreeMap`
/// transport plan also allocates, which a production path could not afford
/// across billions of EMD computations.
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
    type X = ClusterAbs;
    type Y = ClusterAbs;
    type P = Potential;
    type Q = Potential;
    type M = Metric;

    fn cost(&self) -> Probability {
        self.plan.values().sum()
    }

    fn flow(&self, x: &Self::X, y: &Self::Y) -> Probability {
        let ref index = Pair::from((&**x, &**y));
        self.plan.get(index).copied().expect("missing in transport plan")
    }

    fn minimize(mut self) -> Self {
        self.plan.clear();
        let ref mut pile = Potential::derive(self.source);
        let ref mut sink = Potential::derive(self.target);
        'cost: while pile.values().any(|v| v > 0.) {
            'pile: for x in Potential::support(pile)
                .filter(|x| Potential::density(pile, x) > 0.)
                .collect::<Vec<_>>()
            {
                match Potential::support(sink)
                    .filter(|y| Potential::density(sink, y) > 0.)
                    .map(|y| (y, self.metric.raw_distance(&x, &y)))
                    .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
                {
                    None => break 'cost,
                    Some((y, distance)) => {
                        let dx = Potential::density(pile, &x);
                        let dy = Potential::density(sink, &y);
                        let mass = Probability::min(dx, dy);
                        let pair = Pair::from((&x, &y));
                        Potential::increment(pile, &x, -mass);
                        Potential::increment(sink, &y, -mass);
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
