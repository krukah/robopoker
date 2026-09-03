//! Read-only view adapter for off-tree-augmented (nested) subgames.
//!
//! Mirrors [`crate::DepthView`]: a `Copy` lens over `&Blueprint` that answers
//! `RefProf` queries across the wrapper edge types. Canonical [`NestEdge::Game`]
//! edges delegate to the wrapped blueprint; the synthetic [`NestEdge::Off`]
//! edge has no blueprint analog, so it reports defaults (the "cold on the
//! augmented action" Modicum prior).
use super::*;
use mccfr::*;
use pokerkit::*;

/// Read-only lens stripping the nest wrapping for blueprint lookups.
///
/// `Copy` (only field is `&P`) so a [`NestProfile`] can copy it out of
/// `&mut self` before mut-borrowing its local map.
pub struct NestView<'blueprint, P>
where
    P: RefProf,
    P::G: Augmentable,
{
    inner: &'blueprint P,
}

impl<P> Copy for NestView<'_, P>
where
    P: RefProf,
    P::G: Augmentable,
{
}
impl<P> Clone for NestView<'_, P>
where
    P: RefProf,
    P::G: Augmentable,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'blueprint, P> NestView<'blueprint, P>
where
    P: RefProf,
    P::G: Augmentable,
{
    pub fn new(inner: &'blueprint P) -> Self {
        Self { inner }
    }
}

impl<P> CfrRule for NestView<'_, P>
where
    P: RefProf,
    P::G: Augmentable,
{
    type T = P::T;
    type E = NestEdge<P::E, <P::G as Augmentable>::Off>;
    type G = NestGame<P::G>;
    type I = NestInfo<P::I, <P::G as Augmentable>::Off>;
}

impl<P> RefProf for NestView<'_, P>
where
    P: RefProf,
    P::G: Augmentable,
{
    fn t(&self) -> usize {
        self.inner.t()
    }

    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        match edge {
            NestEdge::Game(e) => self.inner.cum_weight(&info.inner(), e),
            NestEdge::Off(_) => 1.0,
        }
    }

    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        match edge {
            NestEdge::Game(e) => self.inner.cum_regret(&info.inner(), e),
            NestEdge::Off(_) => EPSILON,
        }
    }

    fn cum_payoff(&self, info: &Self::I, edge: &Self::E) -> Utility {
        match edge {
            NestEdge::Game(e) => self.inner.cum_payoff(&info.inner(), e),
            NestEdge::Off(_) => 0.0,
        }
    }

    fn cum_visits(&self, info: &Self::I, edge: &Self::E) -> u32 {
        match edge {
            NestEdge::Game(e) => self.inner.cum_visits(&info.inner(), e),
            NestEdge::Off(_) => 0,
        }
    }

    fn sum_regret(&self) -> Utility {
        self.inner.sum_regret()
    }

    fn warmstart(&self, info: &Self::I, edge: &Self::E) -> Encounter {
        match edge {
            NestEdge::Off(_) => Encounter::default(),
            NestEdge::Game(e) => self.inner.warmstart(&info.inner(), e),
        }
    }
}

impl<P> CfrSampling for NestView<'_, P>
where
    P: RefProf + CfrSampling,
    P::G: Augmentable,
{
    fn increment(&mut self) {}
    fn walker(&self) -> Self::T {
        self.inner.walker()
    }

    fn temperature(&self) -> Entropy {
        self.inner.temperature()
    }

    fn smoothing(&self) -> Energy {
        self.inner.smoothing()
    }

    fn curiosity(&self) -> Probability {
        self.inner.curiosity()
    }
}
