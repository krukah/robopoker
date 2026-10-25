//! Read-only view lifting a blueprint into world-tagged info sets.
use crate::*;
use mccfr::*;
use pokerkit::*;

/// Lifts a blueprint into the world spec: every lookup strips the world tag and reads the
/// blueprint. No policy perturbation; worlds differ structurally, through [`WorldRestrict`]
/// dealing different opponent cards per world (Brown & Sandholm 2017).
pub struct WorldView<'blueprint, P>
where
    P: Profile,
{
    inner: &'blueprint P,
}

impl<P> Copy for WorldView<'_, P> where P: Profile {}

impl<P> Clone for WorldView<'_, P>
where
    P: Profile,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> CfrSpec for WorldView<'_, P>
where
    P: Profile,
{
    type F = WorldInfo<InfoOf<P>>;
}

impl<P> Profile for WorldView<'_, P>
where
    P: Profile,
{
    fn t(&self) -> usize {
        self.inner().t()
    }

    fn cum_weight(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        self.inner().cum_weight(&info.inner(), edge)
    }

    fn cum_regret(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.inner().cum_regret(&info.inner(), edge)
    }

    fn cum_payoff(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.inner().cum_payoff(&info.inner(), edge)
    }

    fn cum_visits(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> u32 {
        self.inner().cum_visits(&info.inner(), edge)
    }

    fn sum_regret(&self) -> Utility {
        self.inner().sum_regret()
    }

    fn warmstart(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Encounter {
        self.inner().warmstart(&info.inner(), edge)
    }
}

impl<P> Clock for WorldView<'_, P>
where
    P: Profile + Clock,
{
    fn increment(&mut self) {}

    fn walker(&self) -> TurnOf<Self> {
        self.inner().walker()
    }

    fn temperature(&self) -> Entropy {
        self.inner().temperature()
    }

    fn smoothing(&self) -> Energy {
        self.inner().smoothing()
    }

    fn curiosity(&self) -> Probability {
        self.inner().curiosity()
    }
}

impl<'blueprint, P> WorldView<'blueprint, P>
where
    P: Profile,
{
    pub fn new(inner: &'blueprint P) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &'blueprint P {
        self.inner
    }
}
