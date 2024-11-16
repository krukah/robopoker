use super::bucket::Bucket;
use crate::clustering::abstraction::Abstraction;
use crate::mccfr::player::Player;
use crate::play::game::Game;

#[derive(Debug)]
pub struct Data {
    game: Game,
    info: Abstraction,
    /// this gets populated on the second pass of tree generation
    /// because it requires global information as a
    /// rank-1 hypergraph quantity
    partition: Option<Bucket>,
}

impl From<(Game, Abstraction)> for Data {
    fn from((game, info): (Game, Abstraction)) -> Self {
        Self {
            game,
            info,
            partition: None,
        }
    }
}

impl Data {
    pub fn game(&self) -> &Game {
        &self.game
    }
    pub fn player(&self) -> Player {
        Player(self.game().player())
    }
    pub fn bucket(&self) -> &Bucket {
        self.partition.as_ref().expect("bucket assigned")
    }
    /// upstream of us, our resident Tree is partitioning
    /// the Data into buckets containing "global" higher rank
    /// information that we can't conveive of. so at compile
    /// time we tell ourselves that we will "fill in the blanks"
    /// later in the Tree generation and partitioning process.
    pub fn assign(&mut self, bucket: Bucket) {
        match self.partition {
            None => self.partition = Some(bucket),
            Some(_) => panic!("don't overwrite bucket"),
        }
    }
    pub fn abstraction(&self) -> &Abstraction {
        &self.info
    }
}
