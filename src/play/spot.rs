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
use crate::players::human::Human;

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
    dealer: Position,
    player: Position,
}

impl Spot {
    pub fn new() -> Self {
        let seats: [Seat; N] = std::array::from_fn(|_| Seat::new(100));
        Self {
            chips: 0,
            seats,
            board: Board::empty(),
            dealer: 0usize,
            player: 0usize,
        }
    }
    pub fn play_loop(&mut self) {
        println!("play_loop");
        self.next_hand();
        loop {
            match self.chooser() {
                Continuation::Decision(_) => {
                    self.apply(Human::act(self));
                }
                Continuation::Awaiting(_) => {
                    self.next_street();
                }
                Continuation::Terminal => {
                    self.next_hand();
                }
            }
        }
    }
    /// apply an Action to the game state.
    /// rotate if it's a decision == not a Card Draw.
    pub fn apply(&mut self, ref action: Action) {
        // assert!(self.options().contains(action));
        println!("{}", action);
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
        if self.is_sampling() {
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
            let hole = Hand::from(seat.cards());
            removed = Hand::add(removed, hole);
        }
        Deck::from(removed.complement())
    }

    fn actor_idx(&self) -> Position {
        assert!(self.seats.len() == N);
        (self.dealer + self.player) % N
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
        2
    }
    const fn sblind() -> Chips {
        1
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
                if self.is_everyone_waiting() {
                    break 'left;
                }
                self.player += 1;
                match self.actor_ref().state() {
                    State::Playing => break 'left,
                    State::Folding => continue 'left,
                    State::Shoving => continue 'left,
                }
            },
        }
    }

    //
    pub fn next_hand(&mut self) {
        println!("next hand");
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
        self.player = 0;
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

    //
    pub fn next_street(&mut self) {
        println!("next street");
        self.next_street_public();
        self.next_street_stacks();
        self.player = 0;
    }
    fn next_street_public(&mut self) {
        assert!(self.board.street() != Street::Show);
        let mut deck = self.deck();
        match self.board.street() {
            Street::Rive | Street::Show => {}
            Street::Flop => self.apply(Action::Draw(deck.draw())),
            Street::Turn => self.apply(Action::Draw(deck.draw())),
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

    //
    pub fn chooser(&self) -> Continuation {
        if self.is_terminal() {
            return Continuation::Terminal;
        }
        if self.is_sampling() {
            return Continuation::Awaiting(self.board.street().next());
        }
        if self.is_decision() {
            return Continuation::Decision(self.player);
        }
        unreachable!("game rules violated")
    }
    fn is_terminal(&self) -> bool {
        self.board.street() == Street::Rive && self.is_everyone_waiting()
            || self.is_everyone_folding()
    }
    fn is_sampling(&self) -> bool {
        self.board.street() != Street::Rive && self.is_everyone_waiting()
    }
    fn is_decision(&self) -> bool {
        self.actor().state() == State::Playing
            && !self.is_terminal() // could be assertions?
            && !self.is_sampling() // could be assertions?
    }

    //
    fn is_everyone_waiting(&self) -> bool {
        self.is_everyone_shoving() || self.is_everyone_calling()
    }
    fn is_everyone_calling(&self) -> bool {
        self.is_everyone_matched() && self.is_everyone_decided()
    }
    fn is_everyone_shoving(&self) -> bool {
        self.player == 0
            && self
                .seats
                .iter()
                .filter(|s| s.state() == State::Playing)
                .count()
                == 1
            || self
                .seats
                .iter()
                .filter(|s| s.state() != State::Folding)
                .all(|s| s.state() == State::Shoving)
    }
    fn is_everyone_matched(&self) -> bool {
        let stake = self.effective_stake();
        self.seats
            .iter()
            .filter(|s| s.state() == State::Playing)
            .all(|s| s.stake() == stake)
    }
    fn is_everyone_folding(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .count()
            == 1
    }
    fn is_everyone_decided(&self) -> bool {
        self.player
            >= match self.board.street() {
                Street::Pref => N + 2,
                _ => N,
            }
    }

    //
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
        self.to_shove() > self.to_raise()
    }
    fn can_shove(&self) -> bool {
        self.to_shove() > 0
    }

    //
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
        std::cmp::max(most + diff, most + Self::bblind()) - self.actor().stake()
    }

    //
    fn strength(&self, seat: &Seat) -> Strength {
        assert!(self.is_terminal());
        let hole = seat.cards();
        let hand = Hand::from(hole);
        let hand = Hand::add(Hand::from(self.board), hand);
        Strength::from(hand)
    }
    fn entry(&self, seat: &Seat) -> Payout {
        assert!(self.is_terminal());
        Payout {
            reward: 0,
            risked: seat.spent(),
            status: seat.state(),
            strength: self.strength(seat),
        }
    }
    fn ledger(&self) -> [Payout; N] {
        assert!(self.is_terminal());
        self.seats
            .iter()
            .map(|seat| self.entry(seat))
            .collect::<Vec<Payout>>()
            .try_into()
            .expect("const N")
    }
    fn showdown(&self) -> Showdown {
        assert!(self.is_terminal());
        Showdown::from(self.ledger())
    }
}

impl std::fmt::Display for Spot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:>6}   {}", self.chips, self.board)
    }
}

pub enum Continuation {
    Decision(Position),
    Awaiting(Street),
    Terminal,
}
type Position = usize;
