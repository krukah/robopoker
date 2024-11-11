use super::bucket::Bucket;
use crate::mccfr::player::Player;
use crate::play::game::Game;

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
    // /// possible future edges emanating from this node
    // pub fn future(data: &Data, history: &[Edge]) -> Path {
    //     Path::from(Data::continuations(data, history))
    // }

    // /// possible edges emanating from this node after N-betting is cut off
    // pub fn continuations(data: &Data, history: &[Edge]) -> Vec<Edge> {
    //     let nraises = history
    //         .iter()
    //         .rev()
    //         .take_while(|e| e.is_choice())
    //         .filter(|e| e.is_aggro())
    //         .count();
    //     Data::expand(data, history)
    //         .into_iter()
    //         .map(|(e, _)| e)
    //         .filter(|e| !e.is_raise() || nraises < crate::MAX_N_BETS)
    //         .collect::<Vec<Edge>>()
    // }
}
