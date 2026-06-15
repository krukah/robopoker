//! [`Depth<B>`] — depth-limited subgame solve via `flagship.adapt_leaf`.
//!
//! Wraps an inner [`Brain`]. Only `Depth<Blueprint>` has a [`Brain`] impl;
//! non-canonical orderings (`Depth<Depth<…>>`, `Depth<World<…>>`)
//! intentionally don't compile.
use std::time::Duration;

use kicker::*;
use nlhe::*;

use super::Blueprint;
use super::Brain;
use super::Mount;
use super::Solved;
use super::Tag;

pub struct Depth<B>
where
    B: Brain,
{
    inner: B,
}

impl<B> Mount for Depth<B>
where
    B: Brain + Mount,
{
    fn mount(tag: Tag, model: &'static Flagship) -> Self {
        Self {
            inner: B::mount(tag, model),
        }
    }
}

impl Brain for Depth<Blueprint> {
    fn tag(&self) -> Tag {
        self.inner.tag()
    }

    fn model(&self) -> &'static Flagship {
        self.inner.model()
    }

    fn solve(&self, recall: &Witness, info: NlheInfo, deadline: Duration) -> Option<Solved> {
        Some(Solved::run(self.model().adapt_leaf(recall), info, deadline))
    }
}
