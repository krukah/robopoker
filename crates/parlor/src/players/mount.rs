//! Uniform construction shape for every layer of the bot-config stack.
//!
//! Every layer (base [`Blueprint`](super::Blueprint), [`Brain`](super::Brain)
//! wrappers [`Depth`](super::Depth) / [`World`](super::World) /
//! [`Dirac`](super::Dirac), and the [`Agent`](super::Agent) on top)
//! takes the same `(Tag, &'static Flagship)` pair. Cascading via the
//! inner type's own [`Mount`] impl means [`zoo`](super::zoo) constructs
//! an arbitrarily-deep stack with one call.
use nlhe::Flagship;

use super::Tag;

pub trait Mount: Sized {
    fn mount(tag: Tag, model: &'static Flagship) -> Self;
}
