use mccfr::*;
use pokerkit::Probability;
use pokerkit::Utility;

/// A decision distribution at one information set — the client-facing form
/// of a solved policy, generic over the mccfr edge. Built from
/// [`Decision`]s (the solver's own currency) by an adapter at the fetch
/// boundary; consumed by the policy panel through [`Self::share`].
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict<E>
where
    E: CfrEdge,
{
    decisions: Vec<Decision<E>>,
    payoff: Utility,
}

impl<E> Verdict<E>
where
    E: CfrEdge,
{
    pub fn new(decisions: Vec<Decision<E>>, payoff: Utility) -> Self {
        Self { decisions, payoff }
    }

    pub fn decisions(&self) -> &[Decision<E>] {
        &self.decisions
    }

    /// The information set's expected value.
    pub fn payoff(&self) -> Utility {
        self.payoff
    }

    /// Total visits across all decisions.
    pub fn count(&self) -> u32 {
        self.decisions().iter().map(mccfr::Decision::visits).sum()
    }

    /// The normalized probability of one edge.
    pub fn share(&self, edge: E) -> Probability {
        let total = self.decisions().iter().map(mccfr::Decision::mass).sum::<Probability>();
        if total > 0.0 {
            self.decisions()
                .iter()
                .find(|decision| decision.edge() == &edge)
                .map_or(0.0, |decision| decision.mass() / total)
        } else {
            0.0
        }
    }

    /// The Dirac collapse: all mass onto the highest-mass edge.
    pub fn argmax(self) -> Self {
        let best = self
            .decisions()
            .iter()
            .max_by(|a, b| a.mass().total_cmp(&b.mass()))
            .map(|decision| *decision.edge());
        Self::new(
            self.decisions()
                .iter()
                .map(|decision| {
                    let mass = Probability::from(u8::from(Some(*decision.edge()) == best));
                    Decision::new(*decision.edge(), mass, decision.visits(), decision.payoff())
                })
                .collect(),
            self.payoff(),
        )
    }
}
