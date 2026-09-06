//! Card representation, hand evaluation, and strategic abstraction primitives.
//!
//! Every representation here is chosen for bijective encoding and fast bitwise
//! operations: [`Card`] is one byte, [`Hand`] is a 64-bit set, [`Evaluator`] is
//! table-driven, and [`Isomorphism`] canonicalizes an [`Observation`] under
//! suit [`Permutation`].
mod board;
mod card;
mod card_seq;
mod deck;
mod evaluator;
mod hand;
mod hand_iter;
mod hand_seq;
mod hole;
mod isomorphism;
mod isomorphism_iter;
mod kicks;
mod lehmer;
mod observation;
mod observation_iter;
mod observation_seq;
mod permutation;
mod rank;
mod ranking;
mod street;
mod strength;
mod suit;

pub use board::*;
pub use card::*;
pub use card_seq::*;
pub use deck::*;
pub use evaluator::*;
pub use hand::*;
pub use hand_iter::*;
pub use hand_seq::*;
pub use hole::*;
pub use isomorphism::*;
pub use isomorphism_iter::*;
pub use kicks::*;
pub use lehmer::*;
pub use observation::*;
pub use observation_iter::*;
pub use observation_seq::*;
pub use permutation::*;
pub use rank::*;
pub use ranking::*;
pub use street::*;
pub use strength::*;
pub use suit::*;
