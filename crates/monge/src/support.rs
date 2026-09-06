/// Marker for types that can be elements of a distribution's support. `Clone`
/// so transport plans and distribution iteration can copy them freely.
pub trait Support: Clone {}

/// usize implements Support for use as world indices in subgame solving.
impl Support for usize {}
/// Unit type as trivial support for games with no private information.
impl Support for () {}
