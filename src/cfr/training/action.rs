use super::player::Player;
use std::hash::Hash;

/// An element of the finite set of possible actions.
pub(crate) trait Action: Eq + Hash + Copy {
    // required
    fn player(&self) -> &Self::APlayer;

    type APlayer: Player;
}
