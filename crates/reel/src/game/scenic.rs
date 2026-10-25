use super::*;

/// The drawing bridge: everything the reel needs that the solver traits
/// don't carry. Implemented per game inside this crate, so solver crates
/// stay ignorant of rendering and mccfr stays untouched.
pub trait Scenic: CfrGame {
    /// The authored demonstration hand — concrete states along the chosen
    /// line, rigged deterministically by the game (fixed cards, real menus).
    fn demo() -> Demo<Self>;

    /// What chance revealed entering this state's phase, from the hero's
    /// point of view, as drawable tokens.
    fn reveals(&self, hero: Self::T) -> Vec<Token>;

    /// The current phase label (street for NLHE, deal for Kuhn).
    fn phase(&self) -> String;

    /// The seat label of a turn under the demo's table convention.
    fn badge(turn: Self::T) -> String;

    /// The semantic action class of an edge — the paint vocabulary.
    fn deed(edge: &Self::E) -> Deed;

    /// The compact tile glyph of an edge.
    fn glyph(edge: &Self::E) -> String;
}
