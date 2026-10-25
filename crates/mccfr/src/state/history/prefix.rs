//! Immutable descent context fixed at subgame-solver construction time.
use crate::*;

/// An action sequence captured at subgame-solver construction time —
/// "context that predates the solver." A [`Trail`] tagged [`Fixed`], which
/// is why it has no `push`: anything the solver appends during its own
/// traversal belongs in a [`Story`], not here.
pub type Prefix<G> = Trail<G, Fixed>;
