//! Read-only view adapter for leaf-augmented games.
use crate::*;
use mccfr::*;
use pokerkit::*;

/// Read-only lens that strips leaf wrapping for blueprint lookups.
///
/// `Copy` because the only field is `&P`, which is trivially copyable. This matters at the
/// `DepthProfile` borrow-checker boundary: a mut-borrow of `self.local` inside
/// `or_insert_with(...)` needs a non-borrowed copy of `self.view` to avoid aliasing.
pub struct DepthView<'blueprint, P, const D: usize>
where
    P: Profile,
{
    inner: &'blueprint P,
}

impl<P, const D: usize> Copy for DepthView<'_, P, D> where P: Profile {}
impl<P, const D: usize> Clone for DepthView<'_, P, D>
where
    P: Profile,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'blueprint, P, const D: usize> DepthView<'blueprint, P, D>
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

impl<P, const D: usize> CfrSpec for DepthView<'_, P, D>
where
    P: Profile,
{
    type F = DepthInfo<InfoOf<P>, D>;
}

impl<P, const D: usize> Profile for DepthView<'_, P, D>
where
    P: Profile,
{
    fn t(&self) -> usize {
        self.inner().t()
    }

    fn cum_weight(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        match edge {
            DepthEdge::Game(e) => self.inner().cum_weight(&info.inner(), e),
            DepthEdge::Pick(_) => 1.0 / D as Probability,
        }
    }

    fn cum_regret(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        match edge {
            DepthEdge::Game(e) => self.inner().cum_regret(&info.inner(), e),
            DepthEdge::Pick(_) => EPSILON,
        }
    }

    fn cum_payoff(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        match edge {
            DepthEdge::Game(e) => self.inner().cum_payoff(&info.inner(), e),
            DepthEdge::Pick(_) => 0.0,
        }
    }

    fn cum_visits(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> u32 {
        match edge {
            DepthEdge::Game(e) => self.inner().cum_visits(&info.inner(), e),
            DepthEdge::Pick(_) => 0,
        }
    }

    fn sum_regret(&self) -> Utility {
        self.inner().sum_regret()
    }

    /// Override the default trait impl: frontier continuation-choice synthetic
    /// edges ([`DepthEdge::Pick`]) have no blueprint analog to inherit from,
    /// so seed them at [`Encounter::default`]. Real game edges delegate to
    /// the wrapped blueprint's warmstart.
    fn warmstart(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Encounter {
        match edge {
            DepthEdge::Pick(_) => Encounter::default(),
            DepthEdge::Game(e) => self.inner().warmstart(&info.inner(), e),
        }
    }
}

impl<P, const D: usize> Clock for DepthView<'_, P, D>
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
