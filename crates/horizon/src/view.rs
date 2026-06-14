//! Read-only view adapter for leaf-augmented games.
use crate::*;
use fulcrum::*;
use regret::*;

/// Read-only lens that strips leaf wrapping for blueprint lookups.
///
/// `Copy` because the only field is `&P`, which is trivially copyable.
/// This matters at the `DepthProfile` borrow-checker boundary: a mut-borrow
/// of `self.local` inside `or_insert_with(...)` needs a non-borrowed copy
/// of `self.view` to avoid aliasing. Copy/Clone are hand-implemented
/// rather than derived because `#[derive]` would require `P: Copy`,
/// which we don't actually need (only `&P` is copied).
pub struct DepthView<'blueprint, P, const D: usize>
where
    P: RefProf,
{
    inner: &'blueprint P,
}

impl<P, const D: usize> Copy for DepthView<'_, P, D> where P: RefProf {}
impl<P, const D: usize> Clone for DepthView<'_, P, D>
where
    P: RefProf,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'blueprint, P, const D: usize> DepthView<'blueprint, P, D>
where
    P: RefProf,
{
    pub fn new(inner: &'blueprint P) -> Self {
        Self { inner }
    }
}

impl<P, const D: usize> CfrRule for DepthView<'_, P, D>
where
    P: RefProf,
{
    type T = P::T;
    type E = DepthEdge<P::E, D>;
    type G = DepthGame<P::G, D>;
    type I = DepthInfo<P::I, D>;
}

impl<P, const D: usize> RefProf for DepthView<'_, P, D>
where
    P: RefProf,
{
    fn t(&self) -> usize {
        self.inner.t()
    }

    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        match edge {
            DepthEdge::Game(e) => self.inner.cum_weight(&info.inner(), e),
            DepthEdge::Pick(_) => 1.0 / D as Probability,
        }
    }

    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        match edge {
            DepthEdge::Game(e) => self.inner.cum_regret(&info.inner(), e),
            DepthEdge::Pick(_) => EPSILON,
        }
    }

    fn cum_payoff(&self, info: &Self::I, edge: &Self::E) -> Utility {
        match edge {
            DepthEdge::Game(e) => self.inner.cum_payoff(&info.inner(), e),
            DepthEdge::Pick(_) => 0.0,
        }
    }

    fn cum_visits(&self, info: &Self::I, edge: &Self::E) -> u32 {
        match edge {
            DepthEdge::Game(e) => self.inner.cum_visits(&info.inner(), e),
            DepthEdge::Pick(_) => 0,
        }
    }

    fn sum_regret(&self) -> Utility {
        self.inner.sum_regret()
    }

    /// Override the default trait impl: frontier continuation-choice synthetic
    /// edges ([`DepthEdge::Pick`]) have no blueprint analog to inherit from,
    /// so seed them at [`Encounter::default`]. Real game edges delegate to
    /// the wrapped blueprint's warmstart.
    fn warmstart(&self, info: &Self::I, edge: &Self::E) -> Encounter {
        match edge {
            DepthEdge::Pick(_) => Encounter::default(),
            DepthEdge::Game(e) => self.inner.warmstart(&info.inner(), e),
        }
    }
}

impl<P, const D: usize> CfrSampling for DepthView<'_, P, D>
where
    P: RefProf + CfrSampling,
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
