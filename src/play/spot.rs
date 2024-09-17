#![allow(dead_code)]

use super::action::Action;
use super::payout::Payout;
use super::seat::Seat;
use super::seat::State;
use super::Chips;
use super::N;
use crate::cards::board::Board;
use crate::cards::deck::Deck;
use crate::cards::hand::Hand;
use crate::cards::street::Street;
use crate::cards::strength::Strength;
use crate::play::showdown::Showdown;

/// Rotation represents the memoryless state of the game in between actions.
///
/// It records both public and private data structs, and is responsible for managing the
/// rotation of players, the pot, and the board. Its immutable methods reveal
/// pure functions representing the rules of how the game may proceed.
/// This full game state will also be our CFR node representation.
#[derive(Debug, Clone, Copy)]
pub struct Spot {
    seats: [Seat; N],
    chips: Chips,
    board: Board,
    dealer: usize,
    nturns: usize,
}

impl Spot {
    /// apply an Action to the game state.
    /// rotate if it's a decision == not a Card Draw.
    pub fn apply(&mut self, ref action: Action) {
        // assert!(self.options().contains(action));
        self.update_stacks(action);
        self.update_states(action);
        self.update_boards(action);
        self.update_rotate(action);
    }

    pub fn actor(&self) -> &Seat {
        self.actor_ref()
    }
    pub fn pot(&self) -> Chips {
        self.chips
    }
    pub fn board(&self) -> Board {
        self.board
    }
    pub fn options(&self) -> Vec<Action> {
        let mut options = Vec::new();
        if self.is_terminal() {
            return options;
        }
        if self.is_awaiting() {
            options.push(Action::Draw(self.deck().draw()));
            return options;
        }
        if self.can_call() {
            options.push(Action::Call(self.to_call()));
        }
        if self.can_raise() {
            options.push(Action::Raise(self.to_raise()));
        }
        if self.can_shove() {
            options.push(Action::Shove(self.to_shove()));
        }
        if self.can_fold() {
            options.push(Action::Fold);
        }
        if self.can_check() {
            options.push(Action::Check);
        }
        options
        //? TODO
        // nothing in here about Action::Blind() being possible,
        // presumably we won't care about this
        // when we construct our MCCFR tree
    }

    fn deck(&self) -> Deck {
        let board = Hand::from(self.board);
        let mut removed = Hand::add(Hand::empty(), board);
        for seat in self.seats.iter() {
            let hole = seat.cards().to_owned();
            let hole = Hand::from(hole);
            removed = Hand::add(removed, hole);
        }
        Deck::from(removed.complement())
    }

    fn actor_idx(&self) -> usize {
        assert!(self.seats.len() == N);
        (self.dealer + self.nturns) % N
    }
    fn actor_ref(&self) -> &Seat {
        let index = self.actor_idx();
        self.seats
            .get(index)
            .expect("index should be in bounds bc modulo")
    }
    fn actor_mut(&mut self) -> &mut Seat {
        let index = self.actor_idx();
        self.seats
            .get_mut(index)
            .expect("index should be in bounds bc modulo")
    }

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

    fn update_stacks(&mut self, action: &Action) {
        match action {
            Action::Call(bet) | Action::Blind(bet) | Action::Raise(bet) | Action::Shove(bet) => {
                self.chips += bet;
                self.actor_mut().bet(bet);
            }
            _ => {}
        }
    }
    fn update_states(&mut self, action: &Action) {
        match action {
            Action::Shove(_) => self.actor_mut().set_state(State::Shoving),
            Action::Fold => self.actor_mut().set_state(State::Folding),
            _ => {}
        }
    }
    fn update_boards(&mut self, action: &Action) {
        match action {
            Action::Draw(card) => self.board.add(card.clone()),
            _ => {}
        }
    }
    fn update_rotate(&mut self, action: &Action) {
        match action {
            Action::Draw(_) => {}
            _ => 'left: loop {
                if self.is_awaiting() {
                    break 'left;
                }
                self.nturns += 1;
                match self.actor_ref().state() {
                    State::Playing => break 'left,
                    State::Folding => continue 'left,
                    State::Shoving => continue 'left,
                }
            },
        }
    }

    fn next_hand(&mut self) {
        self.next_hand_public();
        self.next_hand_rotate();
        self.next_hand_stacks();
        self.next_hand_blinds(Self::sblind());
        self.next_hand_blinds(Self::bblind());
    }
    fn next_hand_public(&mut self) {
        self.chips = 0;
        self.board.clear();
        assert!(self.board.street() == Street::Pref);
    }
    fn next_hand_rotate(&mut self) {
        assert!(self.seats.len() == N);
        assert!(self.board.street() == Street::Pref);
        self.nturns = 0;
        self.dealer += 1;
        self.dealer %= N;
    }
    fn next_hand_stacks(&mut self) {
        assert!(self.board.street() == Street::Pref);
        let mut deck = Deck::new();
        for seat in self.seats.iter_mut() {
            seat.set_state(State::Playing);
            seat.set_cards(deck.hole());
            seat.set_stake();
            seat.set_spent();
        }
    }
    fn next_hand_blinds(&mut self, blind: Chips) {
        assert!(self.board.street() == Street::Pref);
        let stack = self.actor_ref().stack();
        if blind < stack {
            self.apply(Action::Blind(blind))
        } else {
            self.apply(Action::Shove(stack))
        }
    }

    fn next_street(&mut self) {
        self.next_street_public();
        self.next_street_stacks();
        self.nturns = 0;
    }
    fn next_street_public(&mut self) {
        assert!(self.board.street() != Street::Rive);
        assert!(self.board.street() != Street::Show);
        let mut deck = self.deck();
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
    fn next_street_stacks(&mut self) {
        for seat in self.seats.iter_mut() {
            seat.set_stake();
        }
    }

    fn is_awaiting(&self) -> bool {
        self.are_all_folded() || self.is_awaiting_revealed()
    }
    fn is_terminal(&self) -> bool {
        self.are_all_folded() || self.is_awaiting_showdown()
    }
    fn is_awaiting_showdown(&self) -> bool {
        self.is_awaiting_revealed() && self.board.street() == Street::Rive
    }
    fn is_awaiting_revealed(&self) -> bool {
        self.are_all_called() || self.are_all_shoved()
    }

    fn are_all_folded(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .count()
            == 1
    }
    fn are_all_shoved(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .all(|s| s.state() == State::Shoving)
    }
    fn are_all_called(&self) -> bool {
        self.is_everyone_matched()
            && (self.is_after_rotation()
                || (self.is_first_decision() && self.is_only_one_playing()))
    }

    fn is_first_decision(&self) -> bool {
        self.nturns
            == match self.board.street() {
                Street::Pref => 2,
                _ => 0,
            }
    }
    fn is_after_rotation(&self) -> bool {
        assert!(self.seats.len() == N);
        self.nturns
            >= match self.board.street() {
                Street::Pref => N + 2,
                _ => N,
            }
    }
    fn is_only_one_playing(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.state() == State::Playing)
            .count()
            == 1
    }
    fn is_everyone_matched(&self) -> bool {
        let call = self.effective_stake();
        self.seats
            .iter()
            .filter(|s| s.state() == State::Playing)
            .all(|s| s.stake() == call)
    }

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

    pub fn to_call(&self) -> Chips {
        self.effective_stake() - self.actor_ref().stake()
    }
    pub fn to_shove(&self) -> Chips {
        std::cmp::max(self.actor_ref().stack(), self.to_call())
    }
    pub fn to_raise(&self) -> Chips {
        let mut stakes = self
            .seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .map(|s| s.stake())
            .collect::<Vec<Chips>>();
        stakes.sort_unstable();
        let most = stakes.pop().unwrap_or(0);
        let next = stakes.pop().unwrap_or(0);
        let diff = most - next;
        std::cmp::max(most + diff, most + Self::bblind())
    }

    fn settle(&self) -> [Payout; N] {
        assert!(self.is_terminal());
        Showdown::from(self.ledger()).settle()
    }
    fn ledger(&self) -> [Payout; N] {
        assert!(self.is_terminal());
        self.seats
            .iter()
            .map(|seat| self.summary(seat))
            .collect::<Vec<Payout>>()
            .try_into()
            .expect("const N")
    }
    fn summary(&self, seat: &Seat) -> Payout {
        assert!(self.is_terminal());
        Payout {
            reward: 0,
            risked: seat.spent(),
            status: seat.state(),
            strength: self.strength(seat),
        }
    }
    fn strength(&self, seat: &Seat) -> Strength {
        assert!(self.is_terminal());
        let hole = seat.cards().to_owned();
        let hand = Hand::from(hole);
        let hand = Hand::add(Hand::from(self.board), hand);
        Strength::from(hand)
    }
}

impl std::fmt::Display for Spot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:>6}   {}", self.chips, self.board)?;
        for seat in &self.seats {
            write!(f, "{}", seat)?;
        }
        Ok(())
    }
}
