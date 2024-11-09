use super::bucket::Bucket;
use super::edge::Edge;
use super::path::Path;
use crate::cards::street::Street;
use crate::clustering::encoding::Odds;
use crate::mccfr::player::Player;
use crate::play::action::Action;
use crate::play::game::Game;
use crate::{Chips, Probability, Utility};

#[derive(Debug)]
pub struct Data {
    game: Game,
    info: Bucket,
}

impl From<(Game, Bucket)> for Data {
    fn from((game, info): (Game, Bucket)) -> Self {
        Self { game, info }
    }
}

impl Data {
    pub fn game(&self) -> &Game {
        &self.game
    }
    pub fn bucket(&self) -> &Bucket {
        &self.info
    }
    pub fn player(&self) -> Player {
        Player(self.game().player())
    }

    /// possible future edges emanating from this node
    pub fn future(&self, history: &[Edge]) -> Path {
        Path::from(self.continuations(history))
    }

    /// possible edges emanating from this node after N-betting is cut off
    pub fn continuations(&self, history: &[Edge]) -> Vec<Edge> {
        let nraises = history
            .iter()
            .rev()
            .take_while(|e| e.is_choice())
            .filter(|e| e.is_aggro())
            .count();
        self.expand(history)
            .into_iter()
            .map(|(e, _)| e)
            .filter(|e| !e.is_raise() || nraises < crate::MAX_N_BETS)
            .collect::<Vec<Edge>>()
    }

    /// all actions available to the player at this node
    pub fn expand(&self, history: &[Edge]) -> Vec<(Edge, Action)> {
        let mut options = self
            .game()
            .legal()
            .into_iter()
            .map(|a| (Edge::from(a), a))
            .collect::<Vec<(Edge, Action)>>();
        if let Some(raise) = options.iter().position(|(_, a)| a.is_raise()) {
            if let Some(shove) = options.iter().position(|(_, a)| a.is_shove()) {
                if let Action::Raise(min) = options.get(raise).unwrap().1 {
                    if let Action::Shove(max) = options.get(shove).unwrap().1 {
                        options.remove(raise);
                        options.splice(
                            raise..raise,
                            self.raises(history)
                                .into_iter()
                                .map(|odds| (Edge::Raises(odds), Probability::from(odds)))
                                .map(|(e, p)| (e, p * self.game().pot() as Utility))
                                .map(|(e, x)| (e, x as Chips))
                                .filter(|(_, x)| min <= *x && *x < max)
                                .map(|(e, a)| (e, Action::Raise(a)))
                                .collect::<Vec<(Edge, Action)>>(),
                        );
                        return options;
                    }
                }
            }
        }
        options
    }

    /// discretized raise sizes, conditional on street and betting history
    pub fn raises(&self, history: &[Edge]) -> Vec<Odds> {
        const PREF_RAISES: [Odds; 10] = [
            Odds(1, 4), // 0.25
            Odds(1, 3), // 0.33
            Odds(1, 2), // 0.50
            Odds(2, 3), // 0.66
            Odds(3, 4), // 0.75
            Odds(1, 1), // 1.00
            Odds(3, 2), // 1.50
            Odds(2, 1), // 2.00
            Odds(3, 1), // 3.00
            Odds(4, 1), // 4.00
        ];
        const FLOP_RAISES: [Odds; 5] = [
            Odds(1, 2), // 0.50
            Odds(3, 4), // 0.75
            Odds(1, 1), // 1.00
            Odds(3, 2), // 1.50
            Odds(2, 1), // 2.00
        ];
        const LATE_RAISES: [Odds; 2] = [
            Odds(1, 2), // 0.50
            Odds(1, 1), // 1.00
        ];
        const LAST_RAISES: [Odds; 1] = [
            Odds(1, 1), // 1.00
        ];
        match self.game().board().street() {
            Street::Pref => PREF_RAISES.to_vec(),
            Street::Flop => FLOP_RAISES.to_vec(),
            _ => match history
                .iter()
                .rev()
                .take_while(|e| e.is_choice())
                .filter(|e| e.is_aggro())
                .count() // this is basically node.is_not_first_raise
            {
                0 => LATE_RAISES.to_vec(),
                _ => LAST_RAISES.to_vec(),
            },
        }
    }
}
