#[derive(Debug, Clone)]
pub struct Game {
    pub bblind: u32,
    pub sblind: u32,
    pub deck: Deck,
    pub tail: Node, // is this useful?
    pub head: Node,
    pub outcomes: Vec<HandResult>,
    pub players: Vec<RoboPlayer>,
    pub actions: Vec<Action>,
}
impl Game {
    pub fn new(seats: Vec<Seat>) -> Self {
        let players: Vec<RoboPlayer> = seats.iter().map(|s| RoboPlayer::new(s)).collect();
        let node = Node::new(seats);
        Game {
            sblind: 1,
            bblind: 2,
            actions: Vec::new(),
            outcomes: Vec::new(),
            deck: Deck::new(),
            tail: node.clone(),
            head: node,
            players,
        }
    }

    pub fn begin_hand(&mut self) {
        self.head.begin_hand();
        self.deal_players();
        self.post_blinds();
    }
    pub fn begin_street(&mut self) {
        self.head.begin_street();
    }
    pub fn apply(&mut self, action: Action) {
        self.head.apply(action.clone());
        self.actions.push(action.clone());
        match action {
            Action::Draw(_) => (),
            _ => println!("{action}"),
        }
    }

    pub fn to_next_hand(&mut self) {
        self.settle();
        self.prune();
        self.reset_hand();
    }
    pub fn to_next_street(&mut self) {
        self.deal_board();
        self.reset_street();
    }
    pub fn to_next_player(&mut self) {
        let seat = self.head.next();
        let player = self.players.iter().find(|p| p.id == seat.id).unwrap();
        let action = player.act(&self);
        self.apply(action);
    }

    fn reset_hand(&mut self) {
        println!("{}\n---\n", self.head);
        for seat in &mut self.head.seats {
            seat.status = BetStatus::Playing;
            seat.stuck = 0;
        }
        self.tail = self.head.clone();
        self.deck = Deck::new();
        self.actions.clear();
        self.outcomes.clear();
    }
    fn reset_street(&mut self) {
        for seat in &mut self.head.seats {
            seat.stuck = 0;
        }
    }

    fn post_blinds(&mut self) {
        // todo!() handle all in case. check if stack > blind ? Post : Shove
        self.apply(Action::Blind(self.head.next().id, self.sblind));
        self.apply(Action::Blind(self.head.next().id, self.bblind));
        self.head.counter = 0;
    }
    fn deal_players(&mut self) {
        // engine
        for player in self.players.iter_mut() {
            let card1 = self.deck.draw().unwrap();
            let card2 = self.deck.draw().unwrap();
            player.hole.cards.clear();
            player.hole.cards.push(card1);
            player.hole.cards.push(card2);
        }
    }
    fn deal_board(&mut self) {
        match self.head.board.street {
            Street::Pre => {
                let card1 = self.deck.draw().unwrap();
                let card2 = self.deck.draw().unwrap();
                let card3 = self.deck.draw().unwrap();
                self.head.board.street = Street::Flop;
                self.apply(Action::Draw(card1));
                self.apply(Action::Draw(card2));
                self.apply(Action::Draw(card3));
                println!("FLOP   {} {} {}", card1, card2, card3);
            }
            Street::Flop => {
                let card = self.deck.draw().unwrap();
                self.head.board.street = Street::Turn;
                self.apply(Action::Draw(card));
                println!("TURN   {}", card)
            }
            Street::Turn => {
                let card = self.deck.draw().unwrap();
                self.head.board.street = Street::River;
                self.apply(Action::Draw(card));
                println!("RIVER  {}", card)
            }
            _ => (),
        }
    }

    fn settle(&mut self) {
        let outcomes = self.payoffs();
        for payoff in outcomes {
            let seat = self.seat_mut(payoff.id);
            seat.stack += payoff.reward;
        }
    }

    fn status(&self, id: usize) -> BetStatus {
        self.head.seats.iter().find(|s| s.id == id).unwrap().status
    }
    fn seat_mut(&mut self, id: usize) -> &mut Seat {
        self.head.seats.iter_mut().find(|s| s.id == id).unwrap()
    }
    fn prune(&mut self) {
        // self.head.seats.retain(|s| s.stack > 0);
    }
    fn risked(&self, id: usize) -> u32 {
        self.actions
            .iter()
            .filter(|a| match a {
                Action::Call(x, _)
                | Action::Blind(x, _)
                | Action::Raise(x, _)
                | Action::Shove(x, _) => *x == id,
                _ => false,
            })
            .map(|a| match a {
                Action::Call(_, bet)
                | Action::Blind(_, bet)
                | Action::Raise(_, bet)
                | Action::Shove(_, bet) => *bet,
                _ => 0,
            })
            .sum()
        // O(n) in actions
    }
    fn evaluate(&self, id: usize) -> u32 {
        ((id - self.head.dealer - 1) % self.head.seats.len()) as u32
    }
}
impl Game {
    fn payoffs(&self) -> Vec<HandResult> {
        let payoffs = self
            .players
            .iter()
            .map(|p| HandResult {
                id: p.id,
                reward: 0,
                staked: self.risked(p.id),
                status: self.status(p.id),
                score: self.evaluate(p.id),
            })
            .collect::<Vec<HandResult>>();
        self.distribute(payoffs)
    }

    fn distribute(&self, mut payoffs: Vec<HandResult>) -> Vec<HandResult> {
        payoffs.sort_by(|a, b| self.prioritize(a, b));
        let cloned = payoffs.clone();
        let mut stake_watermark = u32::MIN;
        let mut score_watermark = u32::MAX;
        'scores: while payoffs.iter().map(|p| p.reward).sum::<u32>() < self.head.pot {
            score_watermark = payoffs
                .iter()
                .filter(|p| p.status != BetStatus::Folded)
                .filter(|p| p.score < score_watermark)
                .map(|p| p.score)
                .max()
                .unwrap();
            let mut winners = payoffs
                .iter_mut()
                .filter(|r| r.status != BetStatus::Folded)
                .filter(|r| r.score == score_watermark)
                .collect::<Vec<&mut HandResult>>();
            'stakes: while winners.len() > 0 {
                // split side pot
                let stakes = winners
                    .iter()
                    .map(|p| p.staked.saturating_sub(stake_watermark))
                    .min()
                    .unwrap();
                let pot = cloned.iter().map(|p| min(p.staked, stakes)).sum::<u32>();
                let share = pot / winners.len() as u32;
                let mut leftover = pot % winners.len() as u32;
                for winner in &mut winners {
                    winner.reward += share;
                    if leftover > 0 {
                        winner.reward += 1;
                        leftover -= 1;
                    }
                }
                // remove winners who have been paid their full share
                stake_watermark += winners
                    .iter()
                    .map(|p| p.staked.saturating_sub(stake_watermark))
                    .min()
                    .unwrap();
                winners.retain(|p| p.staked > stake_watermark);
                continue 'stakes;
            }
            continue 'scores;
        }
        payoffs
    }

    fn prioritize(&self, a: &HandResult, b: &HandResult) -> Ordering {
        let x = (a.id - self.head.dealer - 1) % self.head.seats.len();
        let y = (b.id - self.head.dealer - 1) % self.head.seats.len();
        x.cmp(&y)
    }
}

use super::{
    action::{Action, Player},
    node::Node,
    payoff::HandResult,
    player::RoboPlayer,
    seat::{BetStatus, Seat},
};
use crate::cards::{board::Street, deck::Deck};
use std::cmp::{min, Ordering};

pub struct Showdown<'a> {
    pub seat: &'a mut Seat,
    pub player: &'a mut RoboPlayer,
}
