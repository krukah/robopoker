use super::{action::Action, game::Hand};

pub trait Player {
    fn id(&self) -> usize;
    fn act(&self, game: &Hand) -> Action;
}
