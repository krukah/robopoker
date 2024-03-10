#[derive(Debug, Clone)]
pub struct RoboPlayer {
    pub id: usize,
}

impl RoboPlayer {
    pub fn new(seat: &Seat) -> RoboPlayer {
        RoboPlayer { id: seat.id }
    }

    pub fn valid_actions(&self, hand: &Hand) -> Vec<Action> {
        let mut actions = vec![];
        if self.can_check(hand) {
            actions.push(Action::Check(self.id));
        }
        if self.can_fold(hand) {
            actions.push(Action::Fold(self.id));
        }
        if self.can_call(hand) {
            actions.push(Action::Call(self.id, self.to_call(hand)));
        }
        if self.can_shove(hand) {
            actions.push(Action::Shove(self.id, self.to_shove(hand)));
        }
        if self.can_raise(hand) {
            actions.push(Action::Raise(self.id, self.to_raise(hand)));
        }
        actions
    }

    pub fn to_call(&self, hand: &Hand) -> u32 {
        hand.head.table_stuck() - self.stuck(hand)
    }
    pub fn to_shove(&self, hand: &Hand) -> u32 {
        std::cmp::min(self.stack(hand), hand.head.table_stack())
    }
    pub fn to_raise(&self, _: &Hand) -> u32 {
        3
    }

    fn seat<'a>(&self, hand: &'a Hand) -> &'a Seat {
        hand.head.seats.iter().find(|s| s.id == self.id).unwrap()
    }
    fn stuck<'a>(&self, hand: &'a Hand) -> u32 {
        self.seat(hand).stuck
    }
    fn stack<'a>(&self, hand: &'a Hand) -> u32 {
        self.seat(hand).stack
    }

    fn can_check(&self, hand: &Hand) -> bool {
        self.stuck(hand) == hand.head.table_stuck()
    }
    fn can_shove(&self, hand: &Hand) -> bool {
        self.to_shove(hand) > 0
    }
    fn can_fold(&self, hand: &Hand) -> bool {
        self.to_call(hand) > 0
    }
    fn can_raise(&self, hand: &Hand) -> bool {
        self.to_call(hand) < self.to_shove(hand)
    }
    fn can_call(&self, hand: &Hand) -> bool {
        self.can_fold(hand) && self.can_raise(hand)
    }

    fn weight(&self, action: Action) -> u32 {
        match action {
            Action::Fold(_) => 15,
            Action::Check(_) => 10,
            Action::Call(..) => 40,
            Action::Raise(..) => 5,
            Action::Shove(..) => 0,
            _ => 0,
        }
    }

    fn policies(&self, hand: &Hand) -> Vec<Policy> {
        self.valid_actions(hand)
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
    fn id(&self) -> usize {
        self.id
    }
    fn act(&self, hand: &Hand) -> Action {
        self.choose(self.policies(hand))
    }
}
use super::{action::Action, game::Hand, player::Player, seat::Seat};
use crate::solver::policy::Policy;
use rand::{thread_rng, Rng};
