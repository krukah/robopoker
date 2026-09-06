//! Empty marker traits carving up CFR-compatible games along axes the type
//! system can't otherwise see, so downstream code can bound on them.
//!
//! The derivation implied by [`EmbeddedHistory`] lives in
//! [`CfrEncoder::info`]'s default, not a blanket impl: coherence forbids
//! `impl<H: EmbeddedHistory> CfrEncoder for H` coexisting with the existing
//! `impl<N: CfrEncoder> CfrEncoder for &N`. Games therefore get the behavior from
//! the default and opt into the marker separately as an attestation.

use crate::*;

/// Attestation that the info set is a pure function of the game state, i.e. the
/// default [`CfrEncoder::info`] (`self.resume(&[], &leaf.1)`) is correct for
/// every reachable [`Leaf`] and no tree context is needed.
///
/// Implies [`PerfectRecall`]; lets a game delete any hand-rolled `info` override.
pub trait EmbeddedHistory: CfrEncoder {}

/// Attestation that `info(tree, leaf)` and `resume(past, head)` agree for
/// every reachable state — the weaker form of history stability, where the
/// encoder may use tree context so long as it doesn't contradict pure replay.
///
/// Replay-based consumers (AIVAT inference, cross-session analysis, subgame
/// seeding) bound on this to guarantee both paths yield the same strategy keys.
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
