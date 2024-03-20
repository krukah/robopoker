// Node represents the memoryless state of the game in between actions. it records both public and private data structs, and is responsible for managing the rotation of players, the pot, and the board. it's immutable methods reveal pure functions representing the rules of how the game may proceed.
#[derive(Debug, Clone)]
pub struct Node {
    pub pot: u32,
    pub dealer: usize,
    pub counter: usize,
    pub pointer: usize,
    pub board: Board,
    pub seats: Vec<Seat>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            seats: Vec::with_capacity(10),
            board: Board::new(),
            pot: 0,
            dealer: 0,
            counter: 0,
            pointer: 0,
        }
    }

    pub fn has_more_hands(&self) -> bool {
        self.seats.iter().filter(|s| s.stack > 0).count() > 1
    }
    pub fn has_more_streets(&self) -> bool {
        !(self.are_all_folded() || (self.board.street == Street::Showdown))
    }
    pub fn has_more_players(&self) -> bool {
        !(self.are_all_folded() || self.are_all_called() || self.are_all_shoved())
    }

    pub fn next(&self) -> &Seat {
        self.seats.get(self.pointer).unwrap()
    }
    pub fn seat(&self, index: usize) -> &Seat {
        self.seats.iter().find(|s| s.position == index).unwrap()
    }
    pub fn seat_mut(&mut self, index: usize) -> &mut Seat {
        self.seats.iter_mut().find(|s| s.position == index).unwrap()
    }
    pub fn after(&self, index: usize) -> usize {
        (index + 1) % self.seats.len()
    }
    pub fn before(&self, index: usize) -> usize {
        (index + self.seats.len() - 1) % self.seats.len()
    }

    pub fn effective_stack(&self) -> u32 {
        let mut totals = self
            .seats
            .iter()
            .map(|s| s.stack + s.stake)
            .collect::<Vec<u32>>();
        totals.sort();
        totals.pop().unwrap_or(0);
        totals.pop().unwrap_or(0)
    }
    pub fn effective_stake(&self) -> u32 {
        self.seats.iter().map(|s| s.stake).max().unwrap()
    }

    pub fn are_all_folded(&self) -> bool {
        // exactly one player has not folded
        self.seats
            .iter()
            .filter(|s| s.status != BetStatus::Folded)
            .count()
            == 1
    }
    pub fn are_all_shoved(&self) -> bool {
        // everyone who isn't folded is all in
        self.seats
            .iter()
            .filter(|s| s.status != BetStatus::Folded)
            .all(|s| s.status == BetStatus::Shoved)
    }
    pub fn are_all_called(&self) -> bool {
        // everyone who isn't folded has matched the bet
        // or all but one player is all in
        let stakes = self.effective_stake();
        let is_first_decision = self.counter == 0;
        let is_one_playing = self
            .seats
            .iter()
            .filter(|s| s.status == BetStatus::Playing)
            .count()
            == 1;
        let has_no_decision = is_first_decision && is_one_playing;
        let has_all_decided = self.counter > self.seats.len();
        let has_all_matched = self
            .seats
            .iter()
            .filter(|s| s.status == BetStatus::Playing)
            .all(|s| s.stake == stakes);
        (has_all_decided || has_no_decision) && has_all_matched
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Pot:   {}\n", self.pot)?;
        write!(f, "Board: {}", self.board)?;
        for seat in &self.seats {
            write!(f, "{}", seat)?;
        }
        write!(f, "")
    }
}

// mutable methods are lowkey reserved for the node's owning Hand -- maybe some CFR engine

impl Node {
    pub fn apply(&mut self, action: Action) {
        let seat = self.seats.get_mut(self.pointer).unwrap();
        // bets entail pot and stack change
        match action {
            Action::Call(_, bet)
            | Action::Blind(_, bet)
            | Action::Raise(_, bet)
            | Action::Shove(_, bet) => {
                self.pot += bet;
                seat.stake += bet;
                seat.stack -= bet;
            }
            _ => (),
        }
        // folds and all-ins entail status change
        match action {
            Action::Fold(..) => seat.status = BetStatus::Folded,
            Action::Shove(..) => seat.status = BetStatus::Shoved,
            _ => (),
        }
        // player actions entail rotation
        match action {
            Action::Draw(card) => self.board.push(card.clone()),
            _ => self.rotate(),
        }
    }
    pub fn start_hand(&mut self) {
        self.prune();
        for seat in self.seats.iter_mut() {
            seat.status = BetStatus::Playing;
            seat.stake = 0;
        }
        self.pot = 0;
        self.counter = 0;
        self.board.cards.clear();
        self.board.street = Street::Pre;
        self.dealer = self.after(self.dealer);
        self.pointer = self.dealer;
        self.rotate();
    }
    pub fn start_street(&mut self) {
        self.counter = 0;
        self.pointer = match self.board.street {
            Street::Pre => self.after(self.after(self.dealer)),
            _ => self.dealer,
        };
        self.rotate();
    }
    pub fn end_street(&mut self) {
        for seat in self.seats.iter_mut() {
            seat.stake = 0;
        }
        self.board.street = match self.board.street {
            Street::Pre => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::River,
            Street::River => Street::Showdown,
            Street::Showdown => unreachable!(),
        }
    }
    fn rotate(&mut self) {
        'left: loop {
            if !self.has_more_players() {
                return;
            }
            self.counter += 1;
            self.pointer = self.after(self.pointer);
            match self.next().status {
                BetStatus::Playing => return,
                BetStatus::Folded | BetStatus::Shoved => continue 'left,
            }
        }
    }
    fn _rewind(&mut self) {
        'right: loop {
            self.counter -= 1;
            self.pointer = self.before(self.pointer);
            match self.next().status {
                BetStatus::Playing => return,
                BetStatus::Folded | BetStatus::Shoved => continue 'right,
            }
        }
    }
    pub fn prune(&mut self) {
        if self.seats.iter().any(|s| s.stack == 0) {
            for seat in self.seats.iter().filter(|s| s.stack == 0) {
                println!("DROP {}", seat);
            }
            self.seats.retain(|s| s.stack > 0);
            for (i, seat) in self.seats.iter_mut().enumerate() {
                seat.position = i;
            }
        }
    }
    pub fn add(&mut self, stack: u32, player: Rc<dyn Player>) {
        let position = self.seats.len();
        let seat = Seat::new(stack, position, player);
        println!("ADD  {}", &seat);
        self.seats.push(seat);
    }
    pub fn drop(&mut self, position: usize) {
        println!("DROP {}", self.seat(position));
        self.seats.remove(position);
        for (i, seat) in self.seats.iter_mut().enumerate() {
            seat.position = i;
        }
    }
}

use super::{
    action::Action,
    player::Player,
    seat::{BetStatus, Seat},
};
use crate::cards::board::{Board, Street};
use std::{
    fmt::{Display, Formatter, Result},
    rc::Rc,
};
