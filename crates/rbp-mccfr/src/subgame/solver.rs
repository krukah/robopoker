//! Subgame solver for safe subgame solving.
//!
//! Wraps an inner solver/encoder and augments it with subgame handling
//! to enable safe subgame solving from arbitrary game states.
use super::*;
use crate::*;
use rbp_core::Probability;
use rbp_core::SUBGAME_ALTS;
use rbp_core::Utility;

/// Solver for safe subgame solving.
///
/// Wraps a base solver configuration and solves from an arbitrary subgame root
/// using the subgame construction for safety guarantees. The tree includes
/// the prefix history for correct reach probability calculations.
///
/// # Type Parameters
///
/// - `P`: Blueprint profile type
/// - `N`: Inner encoder type
/// - `I`: Number of subgame iterations
pub struct SubSolver<'blueprint, P, N, const I: usize>
where
    P: Profile,
    N: Encoder<T = P::T, E = P::E, G = P::G, I = P::I>,
{
    /// Encoder for the subgame-augmented game.
    encoder: SubEncoder<'blueprint, N>,
    /// Profile with blueprint and local storage.
    profile: SubProfile<'blueprint, P>,
    /// Root of the subgame being solved (starts at game root with prefix).
    subroot: SubGame<P::G>,
}
impl<'blueprint, P, N, const I: usize> SubSolver<'blueprint, P, N, I>
where
    P: Profile,
    N: Encoder<T = P::T, E = P::E, G = P::G, I = P::I>,
{
    /// Creates a new subgame solver.
    ///
    /// The tree starts from `P::G::root()` and replays the prefix history
    /// before entering the subgame gadget. This ensures reach probabilities
    /// include the full path for correct Bayesian weighting.
    ///
    /// # Arguments
    ///
    /// - `encoder`: The inner encoder for the base game
    /// - `profile`: Reference to the frozen blueprint profile
    /// - `villain`: The player who selects alternatives (non-traverser)
    /// - `prefix`: Sequence of actions from game root to subgame entry
    /// - `worlds`: K-world distribution for the subgame gadget
    pub fn new(
        encoder: &'blueprint N,
        profile: &'blueprint P,
        villain: P::T,
        prefix: Vec<P::E>,
        worlds: ManyWorlds<SUBGAME_ALTS>,
    ) -> Self {
        Self {
            subroot: SubGame::new(villain, prefix.len()),
            encoder: SubEncoder::new(encoder, prefix),
            profile: SubProfile::new(profile, worlds),
        }
    }
    /// Returns the solved profile (for extracting strategies).
    pub fn into_profile(self) -> SubProfile<'blueprint, P> {
        self.profile
    }
}
impl<'blueprint, P, N, const I: usize> Solver for SubSolver<'blueprint, P, N, I>
where
    P: Profile + Sync,
    N: Encoder<T = P::T, E = P::E, G = P::G, I = P::I> + Sync,
{
    type T = SubTurn<P::T>;
    type E = SubEdge<P::E>;
    type X = SubPublic<<P::I as CfrInfo>::X, P::E>;
    type Y = SubSecret<<P::I as CfrInfo>::Y>;
    type I = SubInfo<P::I, P::E>;
    type G = SubGame<P::G>;
    type S = ExternalSampling;
    type R = LinearRegret;
    type W = LinearWeight;
    type P = SubProfile<'blueprint, P>;
    type N = SubEncoder<'blueprint, N>;
    fn tree_count() -> usize {
        I
    }
    fn batch_size() -> usize {
        1
    }
    fn advance(&mut self) {
        self.profile.increment();
    }
    fn encoder(&self) -> &Self::N {
        &self.encoder
    }
    fn profile(&self) -> &Self::P {
        &self.profile
    }
    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut Probability {
        self.profile.mut_weight(info, edge)
    }
    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        self.profile.mut_regret(info, edge)
    }
    fn mut_evalue(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        self.profile.mut_evalue(info, edge)
    }
    fn mut_counts(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32 {
        self.profile.mut_counts(info, edge)
    }
    /// Override root to return the subgame at root.
    fn root(&self) -> Self::G {
        self.subroot
    }
}
