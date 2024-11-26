use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::metric::Metric;
use super::pair::Pair;
use super::potential::Potential;
use crate::transport::coupling::Coupling;
use crate::transport::measure::Measure;
use crate::Probability;
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
        let ref mut pile = Potential::normalize(self.source);
        let ref mut sink = Potential::normalize(self.target);
        'cost: while pile.values().any(|&dx| dx > 0.) {
            'pile: for (x, dx) in pile
                .iter_mut()
                .filter(|(_, dx)| **dx > 0.)
                .map(|(&x, dx)| (x, dx))
                .collect::<Vec<_>>()
            {
                match sink
                    .iter_mut()
                    .filter(|(_, dy)| **dy > 0.)
                    .map(|(&y, dy)| ((y, dy), self.metric.distance(&x, &y)))
                    .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
                {
                    None => break 'cost,
                    Some(((y, dy), distance)) => {
                        let mass = Probability::min(*dx, *dy);
                        let pair = Pair::from((&x, &y));
                        *dx -= mass;
                        *dy -= mass;
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
