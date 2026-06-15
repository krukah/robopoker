//! [`Blueprint`] — the base brain. In-memory blueprint lookup at every
//! decision; no subgame solver. Wrap with [`Sample`](super::Sample) or
//! [`Dirac`](super::Dirac) to get the `blueprint` / `zerotemp`
//! production presets.
use nlhe::Flagship;

use super::Brain;
use super::Mount;
use super::Tag;

pub struct Blueprint {
    model: &'static Flagship,
    tag: Tag,
}

impl Mount for Blueprint {
    fn mount(tag: Tag, model: &'static Flagship) -> Self {
        Self { model, tag }
    }
}

impl Brain for Blueprint {
    fn tag(&self) -> Tag {
        self.tag
    }

    fn model(&self) -> &'static Flagship {
        self.model
    }
    // solve() = default (None) — blueprint base never solves
    // distrib() = default — preflop and postflop both fall back to blueprint
}
