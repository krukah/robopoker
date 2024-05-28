use crate::cfr::traits::marker::action::Action;
use crate::cfr::traits::marker::player::Player;
use crate::cfr::traits::marker::bucket::Bucket;
use crate::cfr::traits::Utility;
use std::hash::Hash;

/// A node, history, game state, etc. is an omniscient, complete state of current game.
pub(crate) trait Node: Eq + Hash {
    // required
    fn parent(&self) -> &Option<&Self>;
    fn precedent(&self) -> &Option<&Self::NAction>;
    fn children(&self) -> &Vec<&Self>;
    fn available(&self) -> &Vec<&Self::NAction>;
    fn bucket(&self) -> Self::NBucket;
    fn player(&self) -> &Self::NPlayer;
    fn utility(&self, player: &Self::NPlayer) -> Utility;

    // provided
    fn follow(&self, action: &Self::NAction) -> &Self {
        self.children()
            .iter()
            .find(|child| action == child.precedent().unwrap())
            .unwrap()
    }
    fn descendants(&self) -> Vec<&Self> {
        match self.children().len() {
            0 => vec![&self],
            _ => self
                .children()
                .iter()
                .map(|child| child.descendants())
                .flatten()
                .collect(),
        }
    }

    type NPlayer: Player;
    type NAction: Action;
    type NBucket: Bucket;
}
