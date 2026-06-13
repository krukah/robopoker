//! Subgame-policy extraction from a CFR profile, keyed on the *base* edge
//! type — the wrapper crates ([`rbp_depth`], [`rbp_world`],
//! [`rbp_subgame`]) implement this on their solver types so callers
//! never have to know about [`DepthEdge::Game`] unwrapping or per-world
//! info iteration.
use std::collections::BTreeMap;

use rbp_core::Probability;
use rbp_core::Utility;

/// Refined per-edge policy, visit counts, and total positive regret at the
/// harvested infoset. `regret` is `Σ_a max(0, R(I, a))` summed over choice
/// edges (and over partitions for world / subgame variants), matching how
/// `visits` aggregates. Divide by the sum of `visits` for per-iteration
/// regret at this decision.
pub struct Harvested<E> {
    pub refined: BTreeMap<E, Probability>,
    pub visits: BTreeMap<E, u32>,
    pub regret: Utility,
}

/// Extract refined policy + visit counts at a base infoset, keyed on the
/// base edge type. Each solver implementation handles its own
/// info-wrapping ([`DepthInfo::Game`], [`WorldInfo::new`]) and
/// edge-unwrapping ([`DepthEdge::Game`]) — the caller passes the
/// pre-wrap info and gets back maps over the base edge.
///
/// [`DepthInfo::Game`]: rbp_depth::DepthInfo::Game
/// [`DepthEdge::Game`]: rbp_depth::DepthEdge::Game
/// [`WorldInfo::new`]: rbp_world::WorldInfo::new
pub trait Harvest {
    /// Base infoset type (pre-wrapping). E.g. `NlheInfo`.
    type Base;
    /// Base edge type (pre-wrapping). E.g. `NlheEdge`.
    type Edge;

    fn harvest(&self, base: Self::Base) -> Harvested<Self::Edge>;
}
