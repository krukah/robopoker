#![allow(dead_code)]

use super::action::Action;
use super::seat::Seat;
use super::seat::Status;
use super::Chips;
use crate::cards::board::Board;
use crate::cards::deck::Deck;
use crate::cards::hand::Hand;
use crate::cards::street::Street;

/// Rotation represents the memoryless state of the game in between actions.
///
/// It records both public and private data structs, and is responsible for managing the
/// rotation of players, the pot, and the board. Its immutable methods reveal
/// pure functions representing the rules of how the game may proceed.
/// This full game state will also be our CFR node representation.
#[derive(Debug, Clone, Copy)]
pub struct Spot {
    chips: Chips,
    board: Board,
    seats: [Seat; 2],
    button: usize,
    nturns: usize,
}

use crate::cfr::player::Player;
impl Spot {
    /// apply an Action to the game state.
    /// rotate if it's a decision == not a Card Draw.
    pub fn apply(&mut self, ref action: Action) {
        self.update_stacks(action);
        self.update_status(action);
        self.update_boards(action);
        self.update_rotate(action);
    }

    fn from_seats() -> Self {
        Self {
            seats: [Seat::new(0); 2],
            board: Board::empty(),
            chips: 0,
            button: 0,
            nturns: 0,
        }
    }

    fn action(&self) -> usize {
        (self.button + self.nturns) % self.seats.len()
    }
    fn player(&self) -> &Player {
        if self.has_no_more_decisions() {
            &Player::Chance
        } else if self.button == self.action() {
            &Player::P1
        } else {
            &Player::P2
        }
    }

    //  actor getters
    //  actor getters
    //  actor getters

    /// who is currently acting?
    fn actor_mut(&mut self) -> &mut Seat {
        let index = self.action();
        self.seats
            .get_mut(index)
            .expect("index should be in bounds bc modulo")
    }
    /// who is currently acting?
    fn actor_ref(&self) -> &Seat {
        let index = self.action();
        self.seats
            .get(index)
            .expect("index should be in bounds bc modulo")
    }

    // lazy deck generation
    // lazy deck generation
    // lazy deck generation

    /// generate a deck from the board and the seats
    /// using removal of cards
    fn remaining(&self) -> Deck {
        let board = Hand::from(self.board);
        let mut removed = Hand::add(Hand::empty(), board);
        for seat in self.seats.iter() {
            let hole = seat.hole_ref().to_owned();
            let hole = Hand::from(hole);
            removed = Hand::add(removed, hole);
        }
        Deck::from(removed.complement())
    }

    // effective chip calcs
    // effective chip calcs
    // effective chip calcs

    fn effective_stack(&self) -> Chips {
        let mut totals = self
            .seats
            .iter()
            .map(|s| s.stack() + s.stake())
            .collect::<Vec<Chips>>();
        totals.sort_unstable();
        totals.pop().unwrap_or(0);
        totals.pop().unwrap_or(0)
    }
    fn effective_stake(&self) -> Chips {
        self.seats
            .iter()
            .map(|s| s.stake())
            .max()
            .expect("non-empty seats")
    }
    const fn bblind() -> Chips {
        20
    }
    const fn sblind() -> Chips {
        10
    }

    // action calcs
    // action calcs
    // action calcs

    fn actions(&self) -> Vec<Action> {
        //? TODO
        // nothing in here about Action::Blind() being possible,
        // presumably we won't care about this
        // when we construct our MCCFR tree
        let mut actions = Vec::with_capacity(5);
        if self.player() == &Player::Chance {
            actions.push(Action::Draw(self.remaining().draw()));
            return actions;
        }
        if self.can_call() {
            actions.push(Action::Call(self.to_call()));
        }
        if self.can_raise() {
            actions.push(Action::Raise(self.to_raise()));
        }
        if self.can_shove() {
            actions.push(Action::Shove(self.to_shove()));
        }
        if self.can_fold() {
            actions.push(Action::Fold);
        }
        if self.can_check() {
            actions.push(Action::Check);
        }
        actions
    }

    // state machine methods
    // state machine methods
    // state machine methods

    /// apply an Action to update pot and
    /// stacks for each Seat.
    fn update_stacks(&mut self, action: &Action) {
        match action {
            Action::Call(bet) | Action::Blind(bet) | Action::Raise(bet) | Action::Shove(bet) => {
                self.chips += bet;
                self.actor_mut().bet(bet);
            }
            _ => {}
        }
    }
    /// apply an Action to update Seat status
    /// for folds and all-ins.
    fn update_status(&mut self, action: &Action) {
        match action {
            Action::Fold => self.actor_mut().set_sttus(Status::Folding),
            Action::Shove(_) => self.actor_mut().set_sttus(Status::Shoving),
            _ => {}
        }
    }
    /// apply an Action to update the Board,
    /// iff it's a Card Draw. could be joined with
    /// update_button
    fn update_boards(&mut self, action: &Action) {
        match action {
            Action::Draw(card) => self.board.add(card.clone()),
            _ => {}
        }
    }
    /// orient the table position
    /// to uphold the rotation invariant. could be joined with
    /// update_boards
    fn update_rotate(&mut self, action: &Action) {
        match action {
            Action::Draw(_) => {}
            _ => 'left: loop {
                if self.has_no_more_decisions() {
                    break 'left;
                }
                self.nturns += 1;
                match self.actor_ref().status() {
                    Status::Playing => return,
                    Status::Folding => continue 'left,
                    Status::Shoving => continue 'left,
                }
            },
        }
    }

    // state transitions
    // state transitions
    // state transitions

    //? check rotation dynamics
    /// reset the game state
    /// for a new hand
    fn next_hand(&mut self) {
        self.next_hand_table();
        self.next_hand_button();
        self.next_hand_players();
        self.post(Self::sblind());
        self.post(Self::bblind());
    }
    fn next_hand_table(&mut self) {
        self.chips = 0;
        self.board.clear();
        assert!(self.board.street() == Street::Pref);
    }
    fn next_hand_button(&mut self) {
        assert!(self.board.street() == Street::Pref);
        self.nturns = 0;
        self.button += 1;
        self.button %= self.seats.len();
    }
    fn next_hand_players(&mut self) {
        assert!(self.board.street() == Street::Pref);
        let mut deck = Deck::new();
        for seat in self.seats.iter_mut() {
            seat.set_sttus(Status::Playing);
            seat.set_cards(deck.hole());
            seat.set_stake();
        }
    }
    fn post(&mut self, blind: Chips) {
        assert!(self.board.street() == Street::Pref);
        let stack = self.actor_ref().stack();
        if blind < stack {
            self.apply(Action::Blind(blind))
        } else {
            self.apply(Action::Shove(stack))
        }
    }

    //? check rotation dynamics
    /// reset the game state for a new street.
    /// zero the Rotation and
    /// start action with SB/UTßG
    fn next_street(&mut self) {
        self.next_street_board();
        self.next_street_seats();
        self.nturns = 0;
    }
    fn next_street_board(&mut self) {
        assert!(self.board.street() != Street::Rive);
        assert!(self.board.street() != Street::Show);
        let mut deck = self.remaining();
        match self.board.street() {
            Street::Rive | Street::Show => {}
            Street::Flop | Street::Turn => {
                self.apply(Action::Draw(deck.draw()));
            }
            Street::Pref => {
                self.apply(Action::Draw(deck.draw()));
                self.apply(Action::Draw(deck.draw()));
                self.apply(Action::Draw(deck.draw()));
            }
        }
    }
    fn next_street_seats(&mut self) {
        for seat in self.seats.iter_mut() {
            seat.set_stake();
        }
    }

    // end-of-street indicators
    // end-of-street indicators
    // end-of-street indicators

    fn has_no_more_decisions(&self) -> bool {
        self.are_all_called() || self.are_all_folded() || self.are_all_shoved()
    }

    fn are_all_folded(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.status() != Status::Folding)
            .count()
            == 1
    }
    fn are_all_shoved(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.status() != Status::Folding)
            .all(|s| s.status() == Status::Shoving)
    }
    fn are_all_called(&self) -> bool {
        let is_first_decision = self.is_first_decision();
        let is_only_one_playing = self.is_only_one_playing();
        let have_all_players_acted = self.is_after_rotation();
        let have_all_players_called = self.is_stake_matched();
        let has_no_actions_to_take = is_first_decision && is_only_one_playing;
        let has_every_player_taken_turn = have_all_players_acted || has_no_actions_to_take;
        have_all_players_called && has_every_player_taken_turn
    }

    fn is_first_decision(&self) -> bool {
        self.nturns
            == match self.board.street() {
                Street::Pref => 2,
                _ => 0,
            }
    }
    fn is_after_rotation(&self) -> bool {
        self.nturns
            >= self.seats.len()
                + match self.board.street() {
                    Street::Pref => 2,
                    _ => 0,
                }
    }
    fn is_only_one_playing(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.status() == Status::Playing)
            .count()
            == 1
    }
    fn is_stake_matched(&self) -> bool {
        let call = self.effective_stake();
        self.seats
            .iter()
            .filter(|s| s.status() == Status::Playing)
            .all(|s| s.stake() == call)
    }

    // action constraints
    // action constraints
    // action constraints

    fn can_fold(&self) -> bool {
        self.to_call() > 0
    }
    fn can_call(&self) -> bool {
        self.can_fold() && self.to_call() <= self.actor_ref().stack()
    }
    fn can_check(&self) -> bool {
        self.effective_stake() == self.actor_ref().stake()
    }
    fn can_raise(&self) -> bool {
        self.to_shove() >= self.to_raise()
    }
    fn can_shove(&self) -> bool {
        self.to_shove() > 0
    }

    // bet constraints
    // bet constraints
    // bet constraints

    fn to_call(&self) -> Chips {
        self.effective_stake() - self.actor_ref().stake()
    }
    fn to_shove(&self) -> Chips {
        std::cmp::max(self.actor_ref().stack(), self.to_call())
    }
    fn to_raise(&self) -> Chips {
        let mut stakes = self
            .seats
            .iter()
            .filter(|s| s.status() != Status::Folding)
            .map(|s| s.stake())
            .collect::<Vec<Chips>>();
        stakes.sort_unstable();
        let most = stakes.pop().unwrap_or(0);
        let next = stakes.pop().unwrap_or(0);
        let diff = most - next;
        std::cmp::max(most + diff, most + Self::bblind())
    }
}

impl std::fmt::Display for Spot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:>8}   {}", self.chips, self.board)?;
        for seat in &self.seats {
            write!(f, "{}", seat)?;
        }
        Ok(())
    }
}
