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
    fn get_stuck(&self, game: &Game) -> u32 {
        let seat = self.get_seat(game);
        seat.stuck
    }
    fn get_stack(&self, game: &Game) -> u32 {
        let seat = self.get_seat(game);
        seat.stack
    }

    fn to_call(&self, game: &Game) -> u32 {
        game.head.get_table_stuck() - self.get_stuck(game)
    }
    fn to_shove(&self, game: &Game) -> u32 {
        let max = min(game.head.get_table_stack(), self.get_stack(game));
        max - self.get_stuck(game)
    }
    fn to_raise(&self, game: &Game) -> u32 {
        let mut rng = thread_rng();
        let min = self.to_call(game) + 1;
        let max = self.to_shove(game) + 1;
        let max = std::cmp::min(max, 50);
        rng.gen_range(min..max)
    }

    fn can_check(&self, game: &Game) -> bool {
        self.get_stuck(game) >= game.head.get_table_stuck()
    }
    fn can_call(&self, game: &Game) -> bool {
        self.get_stuck(game) < game.head.get_table_stuck()
            && self.get_stack(game) >= self.to_call(game)
    }
    fn can_raise(&self, game: &Game) -> bool {
        self.get_stack(game) > self.to_call(game)
    }
    // min bet is min(stack, big blind)
    // max bet is min(stack, effective stack)
}

impl Actor for RoboPlayer {
    fn act(&self, game: &Game) -> Action {
        // sleep(Duration::from_millis(400));
        let rand = thread_rng().gen_range(0..=99);

        if self.can_check(game) && rand < 60 {
            Action::Check
        } else if self.can_call(game) && rand < 90 {
            Action::Call(self.to_call(game))
        } else if self.can_raise(game) && rand < 10 {
            Action::Raise(self.to_raise(game))
        } else {
            Action::Fold
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
