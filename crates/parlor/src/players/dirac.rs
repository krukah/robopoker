//! [`Dirac<B>`] — sharpens an inner [`Brain`]'s distribution to a Dirac
//! delta on its mode. Stacks alongside [`Depth`](super::Depth) and
//! [`World`](super::World) at parity: all three are `Brain` wrappers
//! that take a `Brain` and produce a new `Brain` with one axis modified.
//!
//! Canonical position is outermost (`Dirac<World<Depth<Blueprint>>>`)
//! because depth/world layers compute a refined distribution and Dirac
//! then commits to its mode. Sampling from a Dirac always returns the
//! same action, so `Agent::decide` doesn't need a separate "argmax"
//! code path — the structure of the distribution carries the semantics.
use std::collections::BTreeMap;
use std::time::Duration;

use croupier::*;
use fulcrum::Probability;
use holdem::*;

use super::Brain;
use super::Mount;
use super::Solved;
use super::Tag;

pub struct Dirac<B>
where
    B: Brain,
{
    inner: B,
}

impl<B> Mount for Dirac<B>
where
    B: Brain + Mount,
{
    fn mount(tag: Tag, model: &'static Flagship) -> Self {
        Self {
            inner: B::mount(tag, model),
        }
    }
}

impl<B> Brain for Dirac<B>
where
    B: Brain,
{
    fn tag(&self) -> Tag {
        self.inner.tag()
    }

    fn model(&self) -> &'static Flagship {
        self.inner.model()
    }

    /// Dirac doesn't run its own subgame solver — it transforms what
    /// the inner brain produces. Delegate `solve` so any subgame layers
    /// underneath still take the deadline.
    fn solve(&self, recall: &Witness, info: NlheInfo, deadline: Duration) -> Option<Solved> {
        self.inner.solve(recall, info, deadline)
    }

    /// Override the default postflop pipeline: take the inner's full
    /// distribution and collapse to a Dirac delta on its mode. When
    /// `Agent::decide` samples from this it deterministically returns
    /// the mode action. Shares [`holdem::argmax`] with
    /// [`holdem::Strategy::argmax`] so the analysis-side dirac
    /// post-process and this gameplay brain agree.
    fn distrib(&self, recall: &Witness) -> BTreeMap<Edge, Probability> {
        holdem::argmax(&self.inner.distrib(recall))
    }
}
