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

    fn get_seat<'a>(&self, game: &'a Game) -> &'a Seat {
        game.head.seats.iter().find(|s| s.id == self.id).unwrap()
    }
    fn get_sunk(&self, game: &Game) -> u32 {
        let seat = self.get_seat(game);
        seat.sunk
    }
    fn get_stack(&self, game: &Game) -> u32 {
        let seat = self.get_seat(game);
        seat.stack
    }

    fn to_call(&self, game: &Game) -> u32 {
        game.head.get_table_sunk() - self.get_sunk(game)
    }
    fn to_shove(&self, game: &Game) -> u32 {
        min(game.head.get_table_stack(), self.get_stack(game)) - self.get_sunk(game)
    }
    fn to_raise(&self, game: &Game) -> u32 {
        let mut rng = thread_rng();
        let min = self.to_call(game) + 1;
        let max = self.to_shove(game);
        let max = min + (max - min) / 4;
        rng.gen_range(min..max)
    }

    fn can_check(&self, game: &Game) -> bool {
        self.get_sunk(game) >= game.head.get_table_sunk()
    }
    // min bet is min(stack, big blind)
    // max bet is min(stack, effective stack)
}

impl Actor for RoboPlayer {
    fn act(&self, game: &Game) -> Action {
        sleep(Duration::from_secs(1));
        let rand = thread_rng().gen_range(0..=99);
        if rand < 20 && self.can_check(game) {
            Action::Check
        } else if rand < 25 && !self.can_check(game) {
            Action::Fold
        } else if rand < 80 {
            Action::Call(self.to_call(game))
        } else if rand < 95 {
            Action::Raise(self.to_raise(game))
        } else {
            Action::Shove(self.to_shove(game))
        }
    }
}
use super::{
    action::{Action, Actor},
    game::Game,
    seat::Seat,
};
use crate::cards::hole::Hole;
use rand::{thread_rng, Rng};
use std::{cmp::min, thread::sleep, time::Duration};
