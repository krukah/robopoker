#[derive(Debug, Clone)]
pub struct RoboPlayer {
    pub id: usize,
    pub hole: Hole,
}

impl RoboPlayer {
    pub fn new(seat: &Seat) -> RoboPlayer {
        RoboPlayer {
            id: seat.id,
            hole: Hole::new(),
        }
    }

    pub fn valid_actions(&self, game: &Game) -> Vec<Action> {
        let mut actions = vec![];
        if self.can_check(game) {
            actions.push(Action::Check(self.id));
        }
        if self.can_fold(game) {
            actions.push(Action::Fold(self.id));
        }
        if self.can_call(game) {
            actions.push(Action::Call(self.id, self.to_call(game)));
        }
        if self.can_shove(game) {
            actions.push(Action::Shove(self.id, self.to_shove(game)));
        }
        if self.can_raise(game) {
            actions.push(Action::Raise(self.id, self.to_raise(game)));
        }
        actions
    }

    pub fn to_call(&self, game: &Game) -> u32 {
        game.head.table_stuck() - self.stuck(game)
    }
    pub fn to_shove(&self, game: &Game) -> u32 {
        std::cmp::min(self.stack(game), game.head.table_stack())
    }
    pub fn to_raise(&self, game: &Game) -> u32 {
        let min = self.to_call(game);
        let max = self.to_shove(game);
        min + (max - min) / 2
    }

    fn seat<'a>(&self, game: &'a Game) -> &'a Seat {
        game.head.seats.iter().find(|s| s.id == self.id).unwrap()
    }
    fn stuck<'a>(&self, game: &'a Game) -> u32 {
        self.seat(game).stuck
    }
    fn stack<'a>(&self, game: &'a Game) -> u32 {
        self.seat(game).stack
    }

    fn can_check(&self, game: &Game) -> bool {
        self.stuck(game) == game.head.table_stuck()
    }
    fn can_shove(&self, game: &Game) -> bool {
        self.to_shove(game) > 0
    }
    fn can_fold(&self, game: &Game) -> bool {
        self.to_call(game) > 0
    }
    fn can_raise(&self, game: &Game) -> bool {
        self.to_call(game) < self.to_shove(game)
    }
    fn can_call(&self, game: &Game) -> bool {
        self.can_fold(game) && self.can_raise(game)
    }

    fn weight(&self, action: Action) -> u32 {
        match action {
            Action::Fold(_) => 15,
            Action::Check(_) => 10,
            Action::Call(..) => 40,
            Action::Raise(..) => 5,
            Action::Shove(..) => 2,
            _ => 0,
        }
    }

    fn policies(&self, game: &Game) -> Vec<Policy> {
        self.valid_actions(game)
            .iter()
            .map(|a| Policy {
                action: a.clone(),
                weight: self.weight(a.clone()),
            })
            .collect()
    }

    fn choose(&self, policies: Vec<Policy>) -> Action {
        let total = policies.iter().map(|p| p.weight).sum();
        let roll = thread_rng().gen_range(0..total);
        let mut sum = 0;
        for policy in policies.iter() {
            sum += policy.weight;
            if roll < sum {
                return policy.action.clone();
            }
        }
        Action::Fold(self.id)
    }
}

impl Player for RoboPlayer {
    fn act(&self, game: &Game) -> Action {
        self.choose(self.policies(game))
    }
}
use super::{
    action::{Action, Player},
    game::Game,
    seat::Seat,
};
use crate::{cards::hole::Hole, solver::policy::Policy};
use rand::{thread_rng, Rng};
