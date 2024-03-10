#[derive(Debug, Clone)]
pub struct HumanPlayer {
    pub id: usize,
}

impl Player for HumanPlayer {
    fn id(&self) -> usize {
        self.id
    }
    fn act(&self, _: &Hand) -> Action {
        Action::Fold(self.id())
    }
}
use super::{action::Action, game::Hand, player::Player};
