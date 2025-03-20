/// Represents a player's turn
pub trait Playee: Clone + Copy + PartialEq + Eq {
    fn chance() -> Self;
}

impl Playee for crate::gameplay::ply::Turn {
    fn chance() -> Self {
        crate::gameplay::ply::Turn::Chance
    }
}
