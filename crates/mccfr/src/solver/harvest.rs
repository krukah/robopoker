//! Subgame-policy extraction from a CFR profile, keyed on the *base* edge
//! type — the wrapper crates ([`horizon`], [`worldview`],
//! [`subgame`]) implement this on their solver types so callers
//! never have to know about `DepthEdge::Game` unwrapping or per-world
//! info iteration.
use crate::*;

/// Extract refined policy + visit counts at a base infoset, keyed on the base
/// edge type.
///
/// Each solver handles its own info-wrapping (`subgame::DepthInfo::Game`,
/// `subgame::WorldInfo::new`) and edge-unwrapping (`subgame::DepthEdge::Game`),
/// so the caller passes pre-wrap info and gets back maps over the base edge.
pub trait Harvest {
    /// The base info type, pre-wrapping, which also names its edges.
    type Base: CfrInfo;

    fn harvest(&self, base: Self::Base) -> Harvested<EdgeOf<Self::Base>>;
}
