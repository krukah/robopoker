use super::*;
use crate::gameplay::*;
use crate::transport::*;
use crate::*;
use std::collections::BTreeMap;

/// greedy algorithm for optimimal transport.
/// my favorite interpretation of this is in the formalization
/// of bipartite matching. we have a Left set of sources and a
/// Right set of targets. we want to find a way to pair each source
/// to a target under the constraint of conserving probability mass.
///
/// for each step of the algorithm, we pair the next source to its nearest target.
/// we move as much mass as we can. we then continue to an arbitrary next source, and repeat.
///
/// this is O(N * M) in (sources, targets) size. for us these are both
/// the K number of clusters at different layers of the hierarchy.
///
/// if anything, the most expensive part about this is the double BTreeMap
/// allocation. considering that we do this a few billion times it is
/// probably worth optimizing into a 0-alloc implementation.
///
/// also, it turns out this algorithm sucks in worst case. like it's just not at all
/// a reasonable heuristic, even in pathological 1D trivial cases.
pub struct Heuristic<'a> {
    plan: BTreeMap<Pair, Probability>,
    metric: &'a Metric,
    source: &'a Histogram,
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
