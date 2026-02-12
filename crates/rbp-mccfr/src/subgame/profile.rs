//! Subgame profile that routes lookups between blueprint and local storage.
//!
//! During safe subgame solving, we maintain:
//! - A frozen blueprint profile for computing reach probabilities
//! - Fresh local regrets/policies for the subgame being solved
//!
//! The profile routes lookups based on whether we're querying
//! world-phase information (from Worlds) or real-game information (local).
use super::*;
use crate::*;
use rbp_core::Probability;
use rbp_core::SUBGAME_ALTS;
use rbp_core::Utility;
use std::collections::BTreeMap;

/// Profile wrapper for safe subgame solving.
///
/// Routes strategy lookups between a frozen blueprint and fresh local storage.
/// The worlds distribution provides reach probabilities for world selection,
/// while local storage accumulates new regrets/policies within the subgame.
///
/// # Type Parameters
///
/// - `P`: The blueprint profile type
pub struct SubProfile<'blueprint, P>
where
    P: Profile,
{
    /// Fresh values accumulated during subgame solving.
    local: BTreeMap<SubInfo<P::I, P::E>, BTreeMap<SubEdge<P::E>, Encounter>>,
    /// Frozen blueprint strategies (immutable during subgame solve).
    global: &'blueprint P,
    /// K-world distribution for opponent range buckets.
    worlds: ManyWorlds<SUBGAME_ALTS>,
    /// Current iteration within subgame solving.
    iterations: usize,
}

impl<'blueprint, P> SubProfile<'blueprint, P>
where
    P: Profile,
{
    /// Creates a new subgame profile from a blueprint reference.
    pub fn new(blueprint: &'blueprint P, worlds: ManyWorlds<SUBGAME_ALTS>) -> Self {
        Self {
            local: BTreeMap::new(),
            global: blueprint,
            iterations: 0,
            worlds,
        }
    }
    /// Returns the blueprint profile.
    pub fn blueprint(&self) -> &P {
        self.global
    }
    /// Returns the worlds distribution.
    pub fn worlds(&self) -> &ManyWorlds<SUBGAME_ALTS> {
        &self.worlds
    }
    /// Returns a mutable reference to the local regret for an info/edge pair.
    pub fn mut_regret(&mut self, info: &SubInfo<P::I, P::E>, edge: &SubEdge<P::E>) -> &mut Utility {
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_default()
            .regret
    }
    /// Returns a mutable reference to the local weight for an info/edge pair.
    pub fn mut_weight(
        &mut self,
        info: &SubInfo<P::I, P::E>,
        edge: &SubEdge<P::E>,
    ) -> &mut Probability {
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_default()
            .weight
    }
    /// Returns a mutable reference to the local EV for an info/edge pair.
    pub fn mut_evalue(&mut self, info: &SubInfo<P::I, P::E>, edge: &SubEdge<P::E>) -> &mut Utility {
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_default()
            .evalue
    }
    /// Returns a mutable reference to the local counts for an info/edge pair.
    pub fn mut_counts(&mut self, info: &SubInfo<P::I, P::E>, edge: &SubEdge<P::E>) -> &mut u32 {
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_default()
            .counts
    }
}

impl<P> Profile for SubProfile<'_, P>
where
    P: Profile,
{
    type T = SubTurn<P::T>;
    type E = SubEdge<P::E>;
    type G = SubGame<P::G>;
    type I = SubInfo<P::I, P::E>;
    fn increment(&mut self) {
        self.iterations += 1;
    }
    fn walker(&self) -> Self::T {
        Self::T::from(self.iterations % 2)
    }
    fn epochs(&self) -> usize {
        self.iterations
    }
    /// Lookup cumulative weight for this info/edge pair.
    ///
    /// For world-phase (world selection), returns the world weight.
    /// For prefix-phase, returns 1.0 (forced edge).
    /// For real-game phase, returns accumulated local weight.
    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        match (info, edge) {
            (SubInfo::Prefix(_, _), SubEdge::Inner(_)) => 1.0,
            (SubInfo::Root, SubEdge::World(i)) => self.worlds.weight(*i),
            (SubInfo::Info(i), SubEdge::Inner(e)) => self
                .local
                .get(info)
                .and_then(|m| m.get(edge))
                .map(|e| e.weight)
                .unwrap_or_else(|| self.global.cum_weight(i, e)),
            _ => panic!("mismatched info/edge phases"),
        }
    }
    /// Lookup cumulative regret for this info/edge pair.
    ///
    /// For world-phase (world selection), returns 0 (no regret tracking).
    /// For prefix-phase, returns 0 (no regret tracking for forced edge).
    /// For real-game phase, returns accumulated local regret.
    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        match (info, edge) {
            (SubInfo::Prefix(_, _), SubEdge::Inner(_)) => 0.0,
            (SubInfo::Root, SubEdge::World(_)) => 0.0,
            (SubInfo::Info(i), SubEdge::Inner(e)) => self
                .local
                .get(info)
                .and_then(|m| m.get(edge))
                .map(|e| e.regret)
                .unwrap_or_else(|| self.global.cum_regret(i, e)),
            _ => panic!("mismatched info/edge phases"),
        }
    }
    /// Lookup cumulative EV for this info/edge pair.
    ///
    /// For world-phase (world selection), returns 0 (no EV tracking).
    /// For prefix-phase, returns 0 (no EV tracking for forced edge).
    /// For real-game phase, returns accumulated local EV.
    fn cum_evalue(&self, info: &Self::I, edge: &Self::E) -> Utility {
        match (info, edge) {
            (SubInfo::Prefix(_, _), SubEdge::Inner(_)) => 0.0,
            (SubInfo::Root, SubEdge::World(_)) => 0.0,
            (SubInfo::Info(i), SubEdge::Inner(e)) => self
                .local
                .get(info)
                .and_then(|m| m.get(edge))
                .map(|e| e.evalue)
                .unwrap_or_else(|| self.global.cum_evalue(i, e)),
            _ => panic!("mismatched info/edge phases"),
        }
    }
    /// Lookup cumulative counts for this info/edge pair.
    ///
    /// For world-phase (world selection), returns 0 (no counts tracking).
    /// For prefix-phase, returns 0 (no counts tracking for forced edge).
    /// For real-game phase, returns accumulated local counts.
    fn cum_counts(&self, info: &Self::I, edge: &Self::E) -> u32 {
        match (info, edge) {
            (SubInfo::Prefix(_, _), SubEdge::Inner(_)) => 0,
            (SubInfo::Root, SubEdge::World(_)) => 0,
            (SubInfo::Info(i), SubEdge::Inner(e)) => self
                .local
                .get(info)
                .and_then(|m| m.get(edge))
                .map(|e| e.counts)
                .unwrap_or_else(|| self.global.cum_counts(i, e)),
            _ => panic!("mismatched info/edge phases"),
        }
    }
    fn temperature(&self) -> rbp_core::Entropy {
        self.global.temperature()
    }
    fn smoothing(&self) -> rbp_core::Energy {
        self.global.smoothing()
    }
    fn curiosity(&self) -> rbp_core::Probability {
        self.global.curiosity()
    }
}
