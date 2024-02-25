pub struct RoboPlayer {
    pub id: usize,
    pub hole: Hole,
}

impl RoboPlayer {
    pub fn new(seat: &Seat) -> RoboPlayer {
        RoboPlayer {
            hole: Hole::new(),
            id: seat.id,
        }
    }

    fn seat<'a>(&self, game: &'a Game) -> &'a Seat {
        game.head.seats.iter().find(|s| s.id == self.id).unwrap()
    }
    fn stuck(&self, game: &Game) -> u32 {
        self.seat(game).stuck
    }
    fn stack(&self, game: &Game) -> u32 {
        self.seat(game).stack
    }

    pub fn to_call(&self, game: &Game) -> u32 {
        game.head.table_stuck() - self.stuck(game)
    }
    pub fn to_shove(&self, game: &Game) -> u32 {
        min(self.stack(game), game.head.table_stack())
    }

    fn can_call(&self, game: &Game) -> bool {
        self.stuck(game) < game.head.table_stuck() && self.stack(game) >= self.to_call(game)
    }

    fn get_random(&self) -> u32 {
        thread_rng().gen_range(0..100)
    }
}

impl Player for RoboPlayer {
    fn act(&self, game: &Game) -> Action {
        todo!()
    }
}
use super::{
    action::{Action, Player},
    game::Game,
    seat::Seat,
};
use crate::cards::hole::Hole;
use rand::{thread_rng, Rng};
use std::cmp::min;
