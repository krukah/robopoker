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
use std::sync::Arc;
use std::sync::Mutex;

/// Computes terminal-equivalent values for depth-limited frontier leaves.
pub trait FrontierEvaluator<P>: Send + Sync
where
    P: Profile,
{
    fn evaluate(
        &self,
        blueprint: &P,
        info: &P::I,
        game: &P::G,
        payoff_turn: P::T,
        continuation: Continuation,
    ) -> Option<Utility>;
}

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
    /// Optional rollout/value evaluator for depth-limited pseudo-terminals.
    frontier: Option<Arc<dyn FrontierEvaluator<P> + 'blueprint>>,
    /// Per-solve cache. It is intentionally local to avoid stale cross-hand EVs.
    frontier_cache: Mutex<BTreeMap<(P::I, Continuation), Utility>>,
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
            frontier: None,
            frontier_cache: Mutex::new(BTreeMap::new()),
        }
    }
    /// Creates a subgame profile with a frontier evaluator.
    pub fn with_frontier_evaluator(
        blueprint: &'blueprint P,
        worlds: ManyWorlds<SUBGAME_ALTS>,
        frontier: Option<Arc<dyn FrontierEvaluator<P> + 'blueprint>>,
    ) -> Self {
        Self {
            local: BTreeMap::new(),
            global: blueprint,
            iterations: 0,
            worlds,
            frontier,
            frontier_cache: Mutex::new(BTreeMap::new()),
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
            (SubInfo::Frontier(_), SubEdge::Continuation(_)) => self
                .local
                .get(info)
                .and_then(|m| m.get(edge))
                .map(|e| e.weight)
                .unwrap_or_default(),
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
            (SubInfo::Frontier(_), SubEdge::Continuation(_)) => self
                .local
                .get(info)
                .and_then(|m| m.get(edge))
                .map(|e| e.regret)
                .unwrap_or_default(),
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
            (SubInfo::Frontier(_), SubEdge::Continuation(_)) => self
                .local
                .get(info)
                .and_then(|m| m.get(edge))
                .map(|e| e.evalue)
                .unwrap_or_default(),
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
            (SubInfo::Frontier(_), SubEdge::Continuation(_)) => self
                .local
                .get(info)
                .and_then(|m| m.get(edge))
                .map(|e| e.counts)
                .unwrap_or_default(),
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
    fn relative_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        leaf: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> Utility {
        if let Some(continuation) = leaf.game().continuation() {
            if let SubInfo::Frontier(info) = leaf.info() {
                let payoff_turn = match root.game().turn() {
                    SubTurn::Natural(turn) | SubTurn::Adverse(turn) => turn,
                };
                return self.frontier_continuation_evalue(
                    info,
                    &leaf.game().inner(),
                    payoff_turn,
                    continuation,
                ) * self.relative_reach(root, leaf)
                    / self.sampling_reach(leaf);
            }
        }
        (if leaf.game().turn() == Self::T::terminal() {
            leaf.game().payoff(root.game().turn())
        } else {
            self.frontier_evalue(leaf.info())
        }) * self.relative_reach(root, leaf)
            / self.sampling_reach(leaf)
    }
}

impl<P> SubProfile<'_, P>
where
    P: Profile,
{
    fn frontier_continuation_evalue(
        &self,
        info: &P::I,
        game: &P::G,
        payoff_turn: P::T,
        continuation: Continuation,
    ) -> Utility {
        let key = (*info, continuation);
        if let Some(value) = self
            .frontier_cache
            .lock()
            .expect("frontier cache")
            .get(&key)
            .copied()
        {
            return value;
        }
        let value = self
            .frontier
            .as_ref()
            .and_then(|evaluator| {
                evaluator.evaluate(self.global, info, game, payoff_turn, continuation)
            })
            // Rollout can fail when the runtime observation is outside the
            // blueprint abstraction. In that case, preserve liveness by using
            // the blueprint frontier EV rather than blocking action selection.
            .unwrap_or_else(|| self.continuation_evalue(info, continuation));
        self.frontier_cache
            .lock()
            .expect("frontier cache")
            .insert(key, value);
        value
    }

    fn continuation_evalue(&self, info: &P::I, continuation: Continuation) -> Utility {
        let choices = info.choices();
        let denom = choices
            .iter()
            .map(|edge| {
                self.global.cum_weight(info, edge).max(rbp_core::POLICY_MIN)
                    * continuation.multiplier(edge)
            })
            .sum::<Probability>();
        if denom <= 0.0 {
            return self.global.frontier_evalue(info);
        }
        choices
            .into_iter()
            .map(|edge| {
                let weight = self
                    .global
                    .cum_weight(info, &edge)
                    .max(rbp_core::POLICY_MIN)
                    * continuation.multiplier(&edge);
                weight * self.global.cum_evalue(info, &edge)
            })
            .sum::<Utility>()
            / denom
    }
}
