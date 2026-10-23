//! Read-only view adapter for off-tree-augmented (nested) subgames.
//!
//! Mirrors [`crate::DepthView`]: a `Copy` lens over `&Blueprint` that answers `Profile` queries
//! across the wrapper edge types. Canonical [`NestEdge::Game`] edges delegate to the wrapped
//! blueprint; the synthetic [`NestEdge::Off`] edge has no blueprint analog, so it reports
//! defaults (the "cold on the augmented action" Modicum prior).
use crate::*;
use mccfr::*;
use pokerkit::*;

/// Read-only lens stripping the nest wrapping for blueprint lookups.
///
/// `Copy` (only field is `&P`) so a [`NestProfile`] can copy it out of
/// `&mut self` before mut-borrowing its local map.
pub struct NestView<'blueprint, P>
where
    P: Profile,
    GameOf<P>: Augmentable,
{
    inner: &'blueprint P,
}

impl<P> Copy for NestView<'_, P>
where
    P: Profile,
    GameOf<P>: Augmentable,
{
}
impl<P> Clone for NestView<'_, P>
where
    P: Profile,
    GameOf<P>: Augmentable,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'blueprint, P> NestView<'blueprint, P>
where
    P: Profile,
    GameOf<P>: Augmentable,
{
    pub fn new(inner: &'blueprint P) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &'blueprint P {
        self.inner
    }
}

impl<P> CfrSpec for NestView<'_, P>
where
    P: Profile,
    GameOf<P>: Augmentable,
{
    type F = NestInfo<InfoOf<P>, <GameOf<P> as Augmentable>::Off>;
}

impl<P> Profile for NestView<'_, P>
where
    P: Profile,
    GameOf<P>: Augmentable,
{
    fn t(&self) -> usize {
        self.inner().t()
    }

    fn cum_weight(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        match edge {
            NestEdge::Game(e) => self.inner().cum_weight(&info.inner(), e),
            NestEdge::Off(_) => 1.0,
        }
    }

    fn cum_regret(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        match edge {
            NestEdge::Game(e) => self.inner().cum_regret(&info.inner(), e),
            NestEdge::Off(_) => EPSILON,
        }
    }

    fn cum_payoff(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        match edge {
            NestEdge::Game(e) => self.inner().cum_payoff(&info.inner(), e),
            NestEdge::Off(_) => 0.0,
        }
    }

    fn cum_visits(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> u32 {
        match edge {
            NestEdge::Game(e) => self.inner().cum_visits(&info.inner(), e),
            NestEdge::Off(_) => 0,
        }
    }

    fn sum_regret(&self) -> Utility {
        self.inner().sum_regret()
    }

    fn warmstart(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Encounter {
        match edge {
            NestEdge::Off(_) => Encounter::default(),
            NestEdge::Game(e) => self.inner().warmstart(&info.inner(), e),
        }
    }
}

impl<P> Clock for NestView<'_, P>
where
    P: Profile + Clock,
    GameOf<P>: Augmentable,
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
