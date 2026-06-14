//! [`World<B>`] — world-partitioned subgame solve.
//!
//! Wraps an inner [`Brain`]. `World<Blueprint>` runs `flagship.adapt_safe`
//! (no depth limit); `World<Depth<Blueprint>>` runs `flagship.adapt_full`
//! (world-partitioned + depth-limited). Other orderings have no [`Brain`]
//! impl.
use std::time::Duration;

use croupier::*;
use holdem::*;

use super::Blueprint;
use super::Brain;
use super::Depth;
use super::Mount;
use super::Solved;
use super::Tag;

pub struct World<B>
where
    B: Brain,
{
    inner: B,
}

impl<B> Mount for World<B>
where
    B: Brain + Mount,
{
    fn mount(tag: Tag, model: &'static Flagship) -> Self {
        Self {
            inner: B::mount(tag, model),
        }
    }
}

impl Brain for World<Blueprint> {
    fn tag(&self) -> Tag {
        self.inner.tag()
    }

    fn model(&self) -> &'static Flagship {
        self.inner.model()
    }

    fn solve(&self, recall: &Witness, info: NlheInfo, deadline: Duration) -> Option<Solved> {
        Some(Solved::run(self.model().adapt_safe(recall), info, deadline))
    }
}

impl Brain for World<Depth<Blueprint>> {
    fn tag(&self) -> Tag {
        self.inner.tag()
    }

    fn model(&self) -> &'static Flagship {
        self.inner.model()
    }

    fn solve(&self, recall: &Witness, info: NlheInfo, deadline: Duration) -> Option<Solved> {
        Some(Solved::run(self.model().adapt_full(recall), info, deadline))
    }
}
