#![allow(dead_code)]

use super::action::Action;
use super::seat::Seat;
use super::seat::Status;
use super::Chips;
use crate::cards::board::Board;
use crate::cards::street::Street;

/// Rotation represents the memoryless state of the game in between actions.
///
/// It records both public and private data structs, and is responsible for managing the
/// rotation of players, the pot, and the board. Its immutable methods reveal
/// pure functions representing the rules of how the game may proceed.
#[derive(Debug, Clone)]
pub struct Rotation {
    // -- this is the real Rotation
    pub dealer: usize,
    pub counts: usize,
    pub action: usize,
    pub chairs: Vec<Seat>, // [Seat;10]
    // hoist this into Game
    pub pot: Chips,
    pub board: Board,
}

impl Rotation {
    pub fn new() -> Self {
        Rotation {
            chairs: Vec::with_capacity(10),
            board: Board::new(),
            pot: 0,
            dealer: 0,
            counts: 0,
            action: 0,
        }
    }

    pub fn has_more_hands(&self) -> bool {
        self.chairs.iter().filter(|s| s.stack() > 0).count() > 1
    }
    pub fn has_more_streets(&self) -> bool {
        !(self.are_all_folded() || (self.board.street() == Street::Show))
    }
    pub fn has_more_players(&self) -> bool {
        !(self.are_all_folded() || self.are_all_called() || self.are_all_shoved())
    }

    pub fn up(&self) -> &Seat {
        self.chairs.get(self.action).unwrap()
    }
    pub fn at(&self, index: usize) -> &Seat {
        self.chairs.iter().find(|s| s.position() == index).unwrap()
    }
    pub fn seat_at_position_mut(&mut self, index: usize) -> &mut Seat {
        self.chairs
            .iter_mut()
            .find(|s| s.position() == index)
            .unwrap()
    }
    pub fn after(&self, index: usize) -> usize {
        (index + 1) % self.chairs.len()
    }
    pub fn before(&self, index: usize) -> usize {
        (index + self.chairs.len() - 1) % self.chairs.len()
    }

    pub fn effective_stack(&self) -> Chips {
        let mut totals = self
            .chairs
            .iter()
            .map(|s| s.stack() + s.stake())
            .collect::<Vec<Chips>>();
        totals.sort();
        totals.pop().unwrap_or(0);
        totals.pop().unwrap_or(0)
    }
    pub fn effective_stake(&self) -> Chips {
        self.chairs.iter().map(|s| s.stake()).max().unwrap()
    }

    pub fn are_all_folded(&self) -> bool {
        // exactly one player has not folded
        self.chairs
            .iter()
            .filter(|s| s.status() != Status::Folding)
            .count()
            == 1
    }
    pub fn are_all_shoved(&self) -> bool {
        // everyone who isn't folded is all in
        self.chairs
            .iter()
            .filter(|s| s.status() != Status::Folding)
            .all(|s| s.status() == Status::Shoving)
    }
    pub fn are_all_called(&self) -> bool {
        // everyone who isn't folded has matched the bet
        // or all but one player is all in
        let stakes = self.effective_stake();
        let is_first_decision = self.counts == 0;
        let is_one_playing = self
            .chairs
            .iter()
            .filter(|s| s.status() == Status::Playing)
            .count()
            == 1;
        let has_no_decision = is_first_decision && is_one_playing;
        let has_all_decided = self.counts > self.chairs.len();
        let has_all_matched = self
            .chairs
            .iter()
            .filter(|s| s.status() == Status::Playing)
            .all(|s| s.stake() == stakes);
        (has_all_decided || has_no_decision) && has_all_matched
    }
}

// mutable methods are lowkey reserved for the node's owning Hand -- maybe some CFR engine
impl Rotation {
    pub fn apply(&mut self, action: Action) {
        let seat = self.chairs.get_mut(self.action).unwrap();
        // bets entail pot and stack change
        // folds and all-ins entail status change
        // choice actions entail rotation & logging, chance action entails board change
        match action {
            Action::Call(_, bet)
            | Action::Blind(_, bet)
            | Action::Raise(_, bet)
            | Action::Shove(_, bet) => {
                seat.bet(bet);
                self.pot += bet;
            }
            _ => (),
        }
        match action {
            Action::Fold(..) => seat.set(Status::Folding),
            Action::Shove(..) => seat.set(Status::Shoving),
            _ => (),
        }
        match action {
            Action::Draw(card) => self.board.add(card),
            _ => {
                self.rotate();
                println!("{action}");
            }
        }
    }
    pub fn begin_hand(&mut self) {
        for seat in self.chairs.iter_mut() {
            seat.set(Status::Playing);
            seat.clear();
        }
        self.pot = 0;
        self.counts = 0;
        self.board.clear();
        self.dealer = self.after(self.dealer);
        self.action = self.dealer;
        self.rotate();
    }
    pub fn begin_street(&mut self) {
        self.counts = 0;
        self.action = match self.board.street() {
            Street::Pref => self.after(self.after(self.dealer)),
            _ => self.dealer,
        };
        self.rotate();
    }
    pub fn end_street(&mut self) {
        for seat in self.chairs.iter_mut() {
            seat.clear();
        }
        self.board.advance();
    }
    fn rotate(&mut self) {
        'left: loop {
            if !self.has_more_players() {
                return;
            }
            self.counts += 1;
            self.action = self.after(self.action);
            match self.up().status() {
                Status::Playing => return,
                Status::Folding | Status::Shoving => continue 'left,
            }
        }
    }
    fn rewind(&mut self) {
        'right: loop {
            self.counts -= 1;
            self.action = self.before(self.action);
            match self.up().status() {
                Status::Playing => return,
                Status::Folding | Status::Shoving => continue 'right,
            }
        }
    }
    pub fn prune(&mut self) {
        self.chairs.retain(|s| s.stack() > 0);
        for (i, seat) in self.chairs.iter_mut().enumerate() {
            seat.assign(i);
        }
    }
    pub fn sit_down(&mut self, stack: Chips) {
        let position = self.chairs.len();
        let seat = Seat::new(stack, position);
        self.chairs.push(seat);
    }
    pub fn stand_up(&mut self, position: usize) {
        self.chairs.remove(position);
        for (i, seat) in self.chairs.iter_mut().enumerate() {
            seat.assign(i);
        }
    }
}

impl std::fmt::Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Pot:   {}", self.pot)?;
        writeln!(f, "Board: {}", self.board)?;
        for seat in &self.chairs {
            write!(f, "{}", seat)?;
        }
        Ok(())
    }
}
