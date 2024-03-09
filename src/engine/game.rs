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
        self.allocate();
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

    fn allocate(&mut self) {
        let outcomes = self.settle();
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
                Action::Call(id_, _)
                | Action::Blind(id_, _)
                | Action::Raise(id_, _)
                | Action::Shove(id_, _) => *id_ == id,
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

    fn priority(&self, i: usize) -> u32 {
        (i.wrapping_sub(self.head.dealer).wrapping_sub(1) % self.head.seats.len()) as u32
    }
}
impl Game {
    fn settle(&self) -> Vec<HandResult> {
        // to keep track of the winner of the hand result vector
        // to keep track of the most immediate side pot of the hand result vector
        // the actual hand result vector
        let mut curr_winning_rank: u32 = u32::MAX;
        let mut curr_highest_stake: u32 = u32::MIN;
        let mut results = self
            .players
            .iter()
            .map(|p| HandResult {
                id: p.id,
                reward: 0,
                staked: self.risked(p.id),
                status: self.status(p.id),
                rank: self.priority(p.id),
            })
            .collect::<Vec<HandResult>>();
        // select the winner(s) of the hand
        'winner: while let Some(next_winning_rank) = results
            .iter()
            .filter(|p| p.status != BetStatus::Folded)
            .filter(|p| p.rank < curr_winning_rank)
            .map(|p| p.rank)
            .max()
        {
            // select the smallest winnable amount to distribute to winner(s), given side pot constraints
            'pot: while let Some(next_highest_stake) = results
                .iter()
                .filter(|p| p.status != BetStatus::Folded)
                .filter(|p| p.rank == next_winning_rank)
                .filter(|p| p.staked > curr_highest_stake)
                .map(|p| p.staked)
                .min()
            {
                // get side pot
                // distribute to the winning HandResult(s) with the highest rank.
                // drop immut borrows. mutate bc end of the loop
                // single- and multi-way pots with division leftovers are handled generally at zero cost
                let winnable = results
                    .iter()
                    .map(|p| p.staked)
                    .map(|s| min(s, next_highest_stake))
                    .map(|s| s.saturating_sub(curr_highest_stake))
                    .sum::<u32>();
                let winners = results
                    .iter_mut()
                    .filter(|p| p.status != BetStatus::Folded)
                    .filter(|p| p.rank == next_winning_rank)
                    .filter(|p| p.staked > curr_highest_stake)
                    .collect::<Vec<&mut HandResult>>();
                let share = winnable / winners.len() as u32;
                let leftover = winnable % winners.len() as u32;
                for winner in winners {
                    winner.reward += share;
                    if leftover > 0 && leftover > self.priority(winner.id) {
                        winner.reward += 1;
                    }
                }
                // lower the bar for score of the next winner
                // raise the bar for the stakes of the next side pot
                // in 99% of cases, these loops will exit after 1 iteration, with 1 winner and 1 main pot
                // but the  abstraction generalizes with zero cost to handle multi-way all-in tie-breaking pots!
                // i spent so fucking long trying to achieve this
                curr_winning_rank = next_winning_rank;
                curr_highest_stake = next_highest_stake;
                match results.iter().map(|p| p.reward).sum::<u32>() < self.head.pot {
                    true => continue 'pot,
                    false => return results,
                }
            }
        }
        panic!()
    }
}

use super::{
    action::{Action, Player},
    node::Node,
    payoff::{self, HandResult},
    player::RoboPlayer,
    seat::{BetStatus, Seat},
};
use crate::cards::{board::Street, deck::Deck};
use std::cmp::{min, Ordering};

pub struct Showdown<'a> {
    pub payoffs: &'a mut Vec<HandResult>,
    pub curr_score: u32,
    pub curr_stake: u32,
    pub prev_stake: u32,
}

impl<'a> Showdown<'a> {
    pub fn new<'b>(payoffs: &'b mut Vec<HandResult>) -> Showdown<'b> {
        Showdown {
            payoffs,
            curr_score: u32::MAX,
            curr_stake: u32::MIN,
            prev_stake: u32::MIN,
        }
    }
}
