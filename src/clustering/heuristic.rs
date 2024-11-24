use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::metric::Metric;
use super::potential::Potential;
use super::xor::Pair;
use crate::transport::coupling::Coupling;
use crate::transport::measure::Measure;
use crate::Distance;
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
    greedy: Metric,
    metric: &'a Metric,
    source: &'a Histogram,
    target: &'a Histogram,
}

impl Heuristic<'_> {
    fn minimize(mut self) -> Self {
        let mut plan = BTreeMap::<Pair, Distance>::default();
        let ref mut pile = self.source.normalize();
        let ref mut sink = self.target.normalize();
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
                        let pair = Pair::from((&x, &y));
                        let flow = Distance::min(*dx, *dy);
                        let cost = flow * distance;
                        *plan.entry(pair).or_default() += cost;
                        *dx -= flow;
                        *dy -= flow;
                        continue 'pile;
                    }
                }
            }
        }
        self.greedy = Metric::from(plan);
        self
    }
    fn flow(&self, x: &Abstraction, y: &Abstraction) -> Distance {
        self.greedy.distance(x, y) // interpretation as distance even though it's just useful bc typing
    }
    fn cost(&self) -> Distance {
        self.source
            .support()
            .map(|x| self.target.support().map(move |y| (x, y)))
            .flatten()
            .map(|(x, y)| self.flow(&x, &y))
            .sum()
    }
}

impl Coupling for Heuristic<'_> {
    type X = Abstraction;
    type Y = Abstraction;
    type P = Potential;
    type Q = Potential;
    type M = Metric;

    fn minimize(self) -> Self {
        self.minimize()
    }
    fn flow(&self, x: &Self::X, y: &Self::Y) -> Distance {
        self.flow(x, y)
    }
    fn cost(&self) -> Distance {
        self.cost()
    }
}

impl<'a> From<(&'a Histogram, &'a Histogram, &'a Metric)> for Heuristic<'a> {
    fn from((source, target, metric): (&'a Histogram, &'a Histogram, &'a Metric)) -> Self {
        Self {
            greedy: Metric::default(),
            metric,
            source,
            target,
        }
    }
}
