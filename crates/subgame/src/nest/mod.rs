//! Off-tree action nesting (Modicum-style) — generic wrapper family.
//!
//! When an opponent takes an action outside the training abstraction (an
//! off-tree bet size), the nesting solver augments the menu at that one node
//! with the literal action and re-solves the subgame. These wrappers carry the
//! off-tree action through the CFR framework without touching the base game's
//! canonical edge type (which typically bit-packs and can't hold the payload).
//!
//! Generic over any [`Augmentable`] game. That trait is the *entire*
//! domain-specific surface: its single method, [`Augmentable::augment`], says
//! how to apply the off-tree action to the base game. Everything else is
//! game-agnostic and unit-tested here on a toy game (no blueprint/DB).
//!
//! ```mermaid
//! flowchart TD
//!   AUG["Augmentable::augment(off) -> Self<br/>the ONE game-specific method"]
//!   NG["NestGame&lt;G&gt;"] -->|"apply(Off) delegates to"| AUG
//!   NG -->|CfrGame| E["NestEdge = Game(E) | Off(G::Off)"]
//!   NI["NestInfo = Entry | Augmented | Game"] -->|public| NP["NestPublic = Game | Entry(_, Off)"]
//!   NP -->|Entry.choices| SPLICE["canonical ∪ {Off(off)}"]
//! ```
//!
//! The concrete `Nlhe` side is now just `impl Augmentable for NlheGame` (a few
//! lines) plus `Nlhe::adapt_nested`. See `docs/active/off-tree-nesting.md`.

mod augmentable;
mod edge;
mod encoder;
mod game;
mod info;
mod profile;
mod public;
#[cfg(test)]
mod tests;
mod view;

pub use augmentable::*;
pub use edge::*;
pub use encoder::*;
pub use game::*;
pub use info::*;
pub use profile::*;
pub use public::*;
pub use view::*;
