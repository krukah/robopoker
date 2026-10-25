use kicker::*;
use pokerkit::*;
use rand::Rng;

/// Edge weights at one information set.
///
/// Weights are kept exactly as the strategy handed them over — the HTTP API returns
/// unnormalized accumulated regret-matching weights, an in-memory profile returns normalized
/// ones — and normalized on read, so no caller has to remember which it is holding. Non-finite
/// and non-positive weights are ignored throughout.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Policy(Vec<(Edge, Probability)>);
impl Policy {
    /// Every edge carrying usable mass, in the order the strategy offered them.
    pub fn edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.0
            .iter()
            .filter(|(_, w)| w.is_finite() && *w > 0.0)
            .map(|(edge, _)| *edge)
    }

    /// How many edges carry a weight, sound or not.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// True when the strategy had nothing to say — an unreached information set.
    pub fn is_empty(&self) -> bool {
        self.total() <= 0.0
    }

    /// Sum of the usable weights: the normalizer for [`mass`](Self::mass).
    pub fn total(&self) -> Probability {
        self.0
            .iter()
            .map(|(_, w)| *w)
            .filter(|w| w.is_finite() && *w > 0.0)
            .sum()
    }

    /// Normalized weight on `edge`; zero when absent or when nothing has mass.
    pub fn mass(&self, edge: Edge) -> Probability {
        match self.total() {
            total if total > 0.0 => self.0.iter().find(|(e, _)| *e == edge).map_or(0.0, |(_, w)| w / total),
            _ => 0.0,
        }
    }

    /// The heaviest edge — what the strategy would do if it never mixed.
    pub fn favorite(&self) -> Option<Edge> {
        self.0
            .iter()
            .filter(|(_, w)| w.is_finite() && *w > 0.0)
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(edge, _)| *edge)
    }

    /// One edge drawn in proportion to its weight — what the strategy actually does.
    ///
    /// [`favorite`](Self::favorite) overstates a mixed strategy by playing its top
    /// action purely, so a rollout draws rather than taking the argmax.
    pub fn draw(&self, rng: &mut impl Rng) -> Option<Edge> {
        let mut cursor = match self.total() {
            total if total > 0.0 => rng.random_range(0.0..total),
            _ => return None,
        };
        self.0
            .iter()
            .filter(|(_, w)| w.is_finite() && *w > 0.0)
            .find(|(_, w)| {
                cursor -= *w;
                cursor <= 0.0
            })
            .map(|(edge, _)| *edge)
            .or_else(|| self.favorite())
    }
}
impl FromIterator<(Edge, Probability)> for Policy {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Edge, Probability)>,
    {
        Self(iter.into_iter().collect())
    }
}
