/// Represents a player's turn
pub trait Turn: Clone + Copy + PartialEq + Eq {
    fn chance() -> Self;
}

impl Turn for crate::gameplay::ply::Turn {
    fn chance() -> Self {
        crate::gameplay::ply::Turn::Chance
    }
}
