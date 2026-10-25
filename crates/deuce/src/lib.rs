//! Card representation, hand evaluation, and strategic abstraction primitives.
//!
//! Every representation here is chosen for bijective encoding and fast bitwise
//! operations: [`Card`] is one byte, [`Hand`] is a 64-bit set, [`Evaluator`] is
//! table-driven, and [`Isomorphism`] canonicalizes an [`Observation`] under
//! suit [`Permutation`].
mod abstraction;
mod cards;
mod evaluation;
mod hands;
mod ordered;
mod street;

pub use abstraction::*;
pub use cards::*;
pub use evaluation::*;
pub use hands::*;
pub use ordered::*;
pub use street::*;
