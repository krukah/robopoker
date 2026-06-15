//! Property marker traits for [`CfrEncoder`] implementations.
//!
//! These traits carve up the space of CFR-compatible games along axes that
//! aren't otherwise visible in the type system:
//!
//! - [`EmbeddedHistory`] — the info set is a pure function of the game state,
//!   so `info(tree, leaf)` can blanket-delegate to `resume(&[], &leaf.1)`.
//! - [`PerfectRecall`] — `info(tree, leaf)` and `resume(past, head)` produce
//!   the same info for every reachable state. Implied by [`EmbeddedHistory`].
//! - [`PublicGame`] — no private information; the secret component is `()`.
//!
//! These are empty marker traits. Their only job is documentation and being
//! usable as trait bounds for downstream code (e.g. a debug_assert harness
//! that cross-checks `info` against `resume` when `PerfectRecall` holds).
//!
//! The actual derivation of behavior from [`EmbeddedHistory`] is baked into
//! [`CfrEncoder::info`]'s default implementation, not into a blanket impl —
//! Rust's coherence rules don't let us have `impl<H: EmbeddedHistory> CfrEncoder
//! for H` coexist with the existing `impl<N: CfrEncoder> CfrEncoder for &N` blanket.
//! So games get the derivation via the default, and opt into [`EmbeddedHistory`]
//! separately as an attestation.

use crate::*;

/// Attestation that the encoder's info set is a pure function of the game state.
///
/// Concretely: for every reachable [`Leaf`], the default
/// [`CfrEncoder::info`] implementation (`self.resume(&[], &leaf.1)`) produces
/// the correct info set. An encoder implementing this trait promises it
/// does not need tree context to reconstruct info.
///
/// # Implications
///
/// - [`PerfectRecall`] is implied (see blanket impl below).
/// - Games tagged with this can delete any hand-rolled `info` override and
///   rely on the [`CfrEncoder`] default.
pub trait EmbeddedHistory: CfrEncoder {}

/// Attestation that `info(tree, leaf)` and `resume(past, head)` agree for
/// every reachable state.
///
/// This is the weaker form of history stability: the encoder may use tree
/// context in `info`, but doing so must not disagree with a pure replay
/// from root. Games that satisfy [`EmbeddedHistory`] get this automatically.
///
/// # Use
///
/// Downstream code that reconstructs info via replay (AIVAT inference,
/// cross-session analysis, subgame seeding) can rely on this marker as a
/// type-level guarantee that replay and tree construction produce the same
/// strategy keys. Code that walks the tree during training can also use this
/// as a bound to enable debug-assertion harnesses that cross-check both
/// paths.
pub trait PerfectRecall: CfrEncoder {}

/// Attestation that the game has no private information.
///
/// Requires the info set's secret component to be `()`. Games tagged with
/// this can skip belief / world machinery entirely in subgame solving,
/// since the posterior over the opponent's private information is trivially
/// a Dirac at the empty world.
pub trait PublicGame: CfrEncoder
where
    <Self::I as CfrInfo>::Y: PublicSecret,
{
}

/// Helper trait: the single inhabited unit type, for constraining
/// [`PublicGame`]'s associated secret component. Rust's `where` clauses
/// can't directly express `Y = ()`, so we round-trip through a sealed
/// marker that only `()` implements.
pub trait PublicSecret: sealed::Sealed {}
impl PublicSecret for () {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for () {}
}

/// Every [`EmbeddedHistory`] encoder is [`PerfectRecall`]: if info is a pure
/// function of game state, the tree-walking and replay-walking paths produce
/// the same answer by construction.
impl<H> PerfectRecall for H where H: EmbeddedHistory {}
