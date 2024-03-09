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
        let results = self.settle();
        for result in results {
            let seat = self
                .head
                .seats
                .iter_mut()
                .find(|s| s.id == result.id)
                .unwrap();
            seat.stack += result.reward;
        }
        println!("{}\n---\n", self.head);
    }

    fn prune(&mut self) {
        // should do some shifting for positions
        self.head.seats.retain(|s| s.stack >= self.bblind);
        self.players
            .retain(|p| self.head.seats.iter().any(|s| s.id == p.id));
        match self.head.seats.len() {
            1 => panic!("Game over"),
            _ => (),
        }
    }

    fn status(&self, id: usize) -> BetStatus {
        self.head.seats.iter().find(|s| s.id == id).unwrap().status
    }
    fn staked(&self, id: usize) -> u32 {
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
    fn rank(&self, id: usize) -> u32 {
        rand::thread_rng().gen::<u32>() % 32
    }
    fn priority(&self, id: usize) -> u32 {
        (id.wrapping_sub(self.head.dealer).wrapping_sub(1) % self.head.seats.len()) as u32
    }

    fn settle(&self) -> Vec<HandResult> {
        let mut results = self.results();
        let mut curr_rank = u32::MAX;
        'winner: while let Some(next_rank) = self.next_rank(&results, curr_rank) {
            let mut curr_stake = u32::MIN;
            'pot: while let Some(next_stake) = self.next_stake(&results, curr_stake, next_rank) {
                self.distribute(&mut results, curr_stake, next_stake, next_rank);
                if self.is_complete(&results) {
                    return results;
                }
                curr_stake = next_stake;
                continue 'pot;
            }
            curr_rank = next_rank;
            continue 'winner;
        }
        unreachable!()
    }

    fn compare(&self, a: &HandResult, b: &HandResult) -> Ordering {
        let x = self.priority(a.id);
        let y = self.priority(b.id);
        x.cmp(&y)
    }

    fn is_complete(&self, results: &Vec<HandResult>) -> bool {
        // strict equality bc our logic is tiiiight
        let rewarded = results.iter().map(|p| p.reward).sum::<u32>();
        rewarded == self.head.pot
    }
    fn next_stake(
        &self,
        results: &Vec<HandResult>,
        curr_stake: u32,
        curr_rank: u32,
    ) -> Option<u32> {
        results
            .iter()
            .filter(|p| p.status != BetStatus::Folded)
            .filter(|p| p.staked > curr_stake)
            .filter(|p| p.rank == curr_rank)
            .map(|p| p.staked)
            .min()
    }
    fn next_rank(&self, results: &Vec<HandResult>, curr_rank: u32) -> Option<u32> {
        results
            .iter()
            .filter(|p| p.status != BetStatus::Folded)
            .filter(|p| p.rank < curr_rank)
            .map(|p| p.rank)
            .max()
    }

    fn results(&self) -> Vec<HandResult> {
        self.players
            .iter()
            .map(|p| HandResult {
                id: p.id,
                rank: self.rank(p.id),
                status: self.status(p.id),
                staked: self.staked(p.id),
                reward: 0,
            })
            .collect::<Vec<HandResult>>()
    }

    fn distribute(
        &self,
        results: &mut Vec<HandResult>,
        curr_stake: u32,
        next_stake: u32,
        next_rank: u32,
    ) {
        // get side pot
        // distribute to the winning HandResult(s) with the highest rank.
        // drop immut borrows. mutate bc end of the loop
        // single- and multi-way pots with division leftovers are handled generally at zero cost
        let winnable = results
            .iter()
            .map(|p| p.staked)
            .map(|s| std::cmp::min(s, next_stake))
            .map(|s| s.saturating_sub(curr_stake))
            .sum::<u32>();
        let mut winners = results
            .iter_mut()
            .filter(|p| p.status != BetStatus::Folded)
            .filter(|p| p.rank == next_rank)
            .filter(|p| p.staked > curr_stake)
            .collect::<Vec<&mut HandResult>>();
        let n = winners.len() as u32;
        let share = winnable / n;
        let leftover = winnable % n;
        for winner in winners.iter_mut() {
            winner.reward += share;
        }
        if leftover > 0 {
            winners.sort_by(|a, b| self.compare(a, b));
            for winner in winners.iter_mut().take(leftover as usize) {
                winner.reward += 1;
            }
        }
    }
}

use core::panic;
use std::cmp::Ordering;

use rand::Rng;

use super::{
    action::{Action, Player},
    node::Node,
    payoff::HandResult,
    player::RoboPlayer,
    seat::{BetStatus, Seat},
};
use crate::cards::{board::Street, deck::Deck};

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
