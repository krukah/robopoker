use super::action::Action;
use super::seat::Seat;
use super::seat::State;
use super::settlement::Settlement;
use crate::cards::board::Board;
use crate::cards::deck::Deck;
use crate::cards::hand::Hand;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::cards::strength::Strength;
use crate::play::ply::Ply;
use crate::play::showdown::Showdown;
use crate::players::human::Human;
use crate::Chips;
use crate::N;
use crate::STACK;

type Position = usize;
/// Rotation represents the memoryless state of the game in between actions.
///
/// It records both public and private data structs, and is responsible for managing the
/// rotation of players, the pot, and the board. Its immutable methods reveal
/// pure functions representing the rules of how the game may proceed.
/// This full game state will also be our CFR node representation.
#[derive(Debug, Clone, Copy)]
pub struct Game {
    seats: [Seat; N],
    chips: Chips,
    board: Board,
    dealer: Position,
    ticker: Position,
}

impl Game {
    /// this will start the game at the first decision
    /// NOT the first action, which are blinds and hole cards dealt.
    /// stack size is always 100 and P1 is always dealer.
    /// these should not matter too much in the MCCFR algorithm,
    /// as long as we alternate the traverser/paths explored
    pub fn root() -> Self {
        let mut root = Self {
            chips: 0 as Chips,
            dealer: 0usize,
            ticker: 0usize,
            board: Board::empty(),
            seats: [Seat::new(STACK); N],
        };
        root.next_player();
        root.deal_cards();
        root.post_blinds(Self::sblind());
        root.post_blinds(Self::bblind());
        root
    }
    pub fn n(&self) -> usize {
        self.seats.len()
    }
    pub fn pot(&self) -> Chips {
        self.chips()
    }
    pub fn apply(&self, action: Action) -> Self {
        let mut child = self.clone();
        child.act(action);
        child
    }
    pub fn play() -> ! {
        let mut node = Self::root();
        loop {
            match node.player() {
                Ply::Chance => todo!(), // node.show_revealed(),
                Ply::Choice(_) => {
                    node.act(Human::decide(&node));
                }
                Ply::Terminal => {
                    node.conclude();
                    node.commence();
                }
            }
        }
    }

    //
    pub fn actor(&self) -> &Seat {
        self.actor_ref()
    }
    pub fn chips(&self) -> Chips {
        self.chips
    }
    pub fn board(&self) -> Board {
        self.board
    }
    pub fn legal(&self) -> Vec<Action> {
        let mut options = Vec::new();
        if self.is_terminal() {
            return options;
        }
        if self.is_sampling() {
            options.push(Action::Draw(self.deck().deal(self.board.street())));
            return options;
        }
        if self.is_blinding() {
            options.push(Action::Blind(Self::sblind()));
            return options;
        }
        if self.can_check() {
            options.push(Action::Check);
        }
        if self.can_fold() {
            options.push(Action::Fold);
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
        options
    }
    pub fn player(&self) -> Ply {
        if self.is_terminal() {
            Ply::Terminal
        } else if self.is_sampling() {
            Ply::Chance
        } else {
            Ply::Choice(self.actor_idx())
        }
    }

    //
    fn conclude(&mut self) {
        self.give_chips();
    }
    fn commence(&mut self) {
        assert!(self.seats.iter().all(|s| s.stack() > 0), "game over");
        self.wipe_board();
        self.deal_cards();
        self.move_button();
        self.post_blinds(Self::sblind());
        self.post_blinds(Self::bblind());
    }
    fn give_chips(&mut self) {
        log::trace!("::::::::::::::");
        log::trace!("{}", self.board());
        for (_, (settlement, seat)) in self
            .settlements()
            .iter()
            .zip(self.seats.iter_mut())
            .enumerate()
            .inspect(|(i, (x, s))| log::trace!("{} {} {:>7} {}", i, s.cards(), s.stack(), x.pnl()))
        {
            seat.win(settlement.reward);
        }
    }
    fn wipe_board(&mut self) {
        self.chips = 0;
        self.board.clear();
        assert!(self.board.street() == Street::Pref);
    }
    fn deal_cards(&mut self) {
        assert!(self.board.street() == Street::Pref);
        let mut deck = Deck::new();
        for seat in self.seats.iter_mut() {
            seat.reset_state(State::Betting);
            seat.reset_cards(deck.hole());
            seat.reset_stake();
            seat.reset_spent();
        }
    }
    fn move_button(&mut self) {
        assert!(self.seats.len() == N);
        assert!(self.board.street() == Street::Pref);
        self.dealer += 1;
        self.dealer %= N;
        self.ticker = self.dealer;
        self.next_player();
    }
    fn post_blinds(&mut self, blind: Chips) {
        assert!(self.board.street() == Street::Pref);
        let stack = self.actor_ref().stack();
        if blind < stack {
            self.act(Action::Blind(blind))
        } else {
            self.act(Action::Shove(stack))
        }
    }

    //
    fn act(&mut self, ref a: Action) {
        log::trace!("acting {} {}", self.actor_idx(), a);
        assert!(self.is_terminal() == false);
        assert!(self
            .legal()
            .iter()
            .map(|o| std::mem::discriminant(o))
            .any(|o| std::mem::discriminant(a) == o));
        match a {
            &Action::Draw(cards) => {
                self.reveal(cards);
                self.next_street();
                self.next_player();
            }
            &Action::Check => {
                self.next_player();
            }
            &Action::Fold => {
                self.actor_mut().reset_state(State::Folding);
                self.next_player();
            }
            &Action::Blind(chips) | &Action::Raise(chips) | &Action::Call(chips) => {
                self.remove(chips);
                self.next_player();
            }
            &Action::Shove(chips) => {
                self.remove(chips);
                self.actor_mut().reset_state(State::Shoving);
                self.next_player();
            }
        }
    }
    fn remove(&mut self, bet: Chips) {
        self.chips += bet;
        self.actor_mut().bet(bet);
    }
    fn reveal(&mut self, hand: Hand) {
        // tightly coupled with next_street?
        self.ticker = self.dealer;
        self.board.add(hand);
        for seat in self.seats.iter_mut() {
            seat.reset_stake();
        }
    }
    fn next_street(&mut self) {}
    fn next_player(&mut self) {
        if !self.is_everyone_alright() {
            loop {
                self.ticker += 1;
                match self.actor_ref().state() {
                    State::Betting => break,
                    State::Folding => continue,
                    State::Shoving => continue,
                }
            }
        }
    }

    /// we're waiting for showdown
    fn is_terminal(&self) -> bool {
        if self.board.street() == Street::Rive {
            self.is_everyone_alright()
        } else {
            self.is_everyone_folding()
        }
    }
    /// we're waiting for a card to be revealed
    fn is_sampling(&self) -> bool {
        if self.board.street() == Street::Rive {
            false
        } else {
            self.is_everyone_alright()
        }
    }
    /// blinds have not yet been posted // TODO some edge case of all in blinds
    fn is_blinding(&self) -> bool {
        if self.board.street() == Street::Pref {
            self.chips() < Self::sblind() + Self::bblind()
        } else {
            false
        }
    }
    /// all players have acted, the pot is right.
    fn is_everyone_alright(&self) -> bool {
        self.is_everyone_calling() || self.is_everyone_folding() || self.is_everyone_shoving()
    }
    /// all players betting are in for the same amount
    fn is_everyone_calling(&self) -> bool {
        self.is_everyone_touched() && self.is_everyone_matched()
    }
    /// all players have acted at least once
    fn is_everyone_touched(&self) -> bool {
        self.ticker
            > if self.board.street() == Street::Pref {
                N + 2
            } else {
                N
            }
    }
    /// all players betting are in for the effective stake
    fn is_everyone_matched(&self) -> bool {
        let stake = self.effective_stake();
        self.seats
            .iter()
            .filter(|s| s.state() == State::Betting)
            .all(|s| s.stake() == stake)
    }
    /// all players betting or shoving are shoving
    fn is_everyone_shoving(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .all(|s| s.state() == State::Shoving)
    }
    /// there is exactly one player betting or shoving
    fn is_everyone_folding(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .count()
            == 1
    }

    //
    fn can_fold(&self) -> bool {
        self.to_call() > 0
    }
    fn can_call(&self) -> bool {
        self.can_fold() && self.to_call() < self.to_shove()
    }
    fn can_check(&self) -> bool {
        self.effective_stake() == self.actor_ref().stake()
    }
    fn can_raise(&self) -> bool {
        self.to_raise() < self.to_shove()
    }
    fn can_shove(&self) -> bool {
        self.to_shove() > 0
    }

    //
    pub fn to_call(&self) -> Chips {
        self.effective_stake() - self.actor_ref().stake()
    }
    pub fn to_shove(&self) -> Chips {
        self.actor_ref().stack()
    }
    pub fn to_raise(&self) -> Chips {
        let mut stakes = self
            .seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .map(|s| s.stake())
            .collect::<Vec<Chips>>();
        stakes.sort_unstable();
        let most_large_stake = stakes.pop().unwrap_or(0);
        let next_large_stake = stakes.pop().unwrap_or(0);
        let relative_raise = most_large_stake - self.actor().stake();
        let marginal_raise = most_large_stake - next_large_stake;
        let required_raise = std::cmp::max(marginal_raise, Self::bblind());
        relative_raise + required_raise
    }

    //
    pub fn settlements(&self) -> Vec<Settlement> {
        assert!(self.is_terminal());
        Showdown::from(self.ledger()).settle()
    }
    fn ledger(&self) -> Vec<Settlement> {
        self.seats
            .iter()
            .map(|seat| self.entry(seat))
            .collect::<Vec<Settlement>>()
    }
    fn entry(&self, seat: &Seat) -> Settlement {
        Settlement {
            reward: 0,
            risked: seat.spent(),
            status: seat.state(),
            strength: self.strength(seat),
        }
    }
    fn strength(&self, seat: &Seat) -> Strength {
        Strength::from(Hand::add(
            Hand::from(seat.cards()),
            Hand::from(self.board()),
        ))
    }

    //
    fn deck(&self) -> Deck {
        let mut removed = Hand::from(self.board);
        for seat in self.seats.iter() {
            let hole = Hand::from(seat.cards());
            removed = Hand::add(removed, hole);
        }
        Deck::from(removed.complement())
    }
    fn actor_idx(&self) -> Position {
        (self.dealer + self.ticker) % N
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

    //
    #[allow(dead_code)]
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
        crate::B_BLIND
    }
    const fn sblind() -> Chips {
        crate::S_BLIND
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for seat in self.seats.iter() {
            write!(f, "{:>6}", seat.stack())?;
        }
        write!(f, " :: {:>6} {}", self.chips, self.board)?;
        Ok(())
    }
}

impl From<&Game> for Observation {
    fn from(game: &Game) -> Self {
        Observation::from((
            Hand::from(game.actor().cards()), //
            Hand::from(game.board()),         //
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root() {
        let game = Game::root();
        assert!(game.ticker != game.dealer);
        assert!(game.board().street() == Street::Pref);
        assert!(game.actor().state() == State::Betting);
        assert!(game.chips() == Game::sblind() + Game::bblind());
    }

    #[test]
    fn everyone_folds_pref() {
        let game = Game::root();
        let game = game.apply(Action::Fold);
        assert!(game.is_everyone_folding() == true);
        assert!(game.is_everyone_alright() == true);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_sampling() == true); // ambiguous
        assert!(game.is_terminal() == true);
    }
    #[test]
    fn everyone_folds_flop() {
        let game = Game::root();
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Call(1));
        let game = game.apply(Action::Check);
        let game = game.apply(Action::Draw(flop));
        let game = game.apply(Action::Raise(10));
        let game = game.apply(Action::Fold);
        assert!(game.is_everyone_folding() == true);
        assert!(game.is_everyone_alright() == true); // fail
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_sampling() == true); // ambiguous
        assert!(game.is_terminal() == true);
    }
    #[test]
    fn history_of_checks() {
        // Blinds
        let game = Game::root();
        assert!(game.board().street() == Street::Pref);
        assert!(game.chips() == 3);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == false);
        assert!(game.is_everyone_matched() == false);

        // SmallB Preflop
        let game = game.apply(Action::Call(1));
        assert!(game.board().street() == Street::Pref);
        assert!(game.chips() == 4); //
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == false);
        assert!(game.is_everyone_matched() == true); //

        // Dealer Preflop
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Pref);
        assert!(game.chips() == 4);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == true); //
        assert!(game.is_everyone_alright() == true); //
        assert!(game.is_everyone_calling() == true); //
        assert!(game.is_everyone_touched() == true); //
        assert!(game.is_everyone_matched() == true);

        // Flop
        let flop = game.deck().deal(game.board().street());
        let game = game.apply(Action::Draw(flop));
        assert!(game.board().street() == Street::Flop); //
        assert!(game.chips() == 4);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == false); //
        assert!(game.is_everyone_alright() == false); //
        assert!(game.is_everyone_calling() == false); //
        assert!(game.is_everyone_touched() == false); //
        assert!(game.is_everyone_matched() == true);

        // SmallB Flop
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Flop);
        assert!(game.chips() == 4);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == false);
        assert!(game.is_everyone_matched() == true);

        // Dealer Flop
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Flop);
        assert!(game.chips() == 4);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == true); //
        assert!(game.is_everyone_alright() == true); //
        assert!(game.is_everyone_calling() == true); //
        assert!(game.is_everyone_touched() == true); //
        assert!(game.is_everyone_matched() == true);

        // Turn
        let turn = game.deck().deal(game.board().street());
        let game = game.apply(Action::Draw(turn));
        assert!(game.board().street() == Street::Turn);
        assert!(game.chips() == 4);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == false); //
        assert!(game.is_everyone_alright() == false); //
        assert!(game.is_everyone_calling() == false); //
        assert!(game.is_everyone_touched() == false); //
        assert!(game.is_everyone_matched() == true);

        // SmallB Turn
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Turn);
        assert!(game.chips() == 4);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == false);
        assert!(game.is_everyone_matched() == true);

        // Dealer Turn
        let game = game.apply(Action::Raise(4));
        assert!(game.board().street() == Street::Turn);
        assert!(game.chips() == 8);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == true); //
        assert!(game.is_everyone_matched() == false); //

        // SmallB Turn
        let game = game.apply(Action::Call(4));
        assert!(game.board().street() == Street::Turn);
        assert!(game.chips() == 12); //
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == true); //
        assert!(game.is_everyone_alright() == true); //
        assert!(game.is_everyone_calling() == true); //
        assert!(game.is_everyone_touched() == true);
        assert!(game.is_everyone_matched() == true);

        // River
        let rive = game.deck().deal(game.board().street());
        let game = game.apply(Action::Draw(rive));
        assert!(game.board().street() == Street::Rive); //
        assert!(game.chips() == 12);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == false); //
        assert!(game.is_everyone_alright() == false); //
        assert!(game.is_everyone_calling() == false); //
        assert!(game.is_everyone_touched() == false); //
        assert!(game.is_everyone_matched() == true); //

        // SmallB River
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Rive);
        assert!(game.chips() == 12);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == false);
        assert!(game.is_sampling() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == false);
        assert!(game.is_everyone_matched() == true);

        // Dealer River
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Rive);
        assert!(game.chips() == 12);
        assert!(game.is_blinding() == false);
        assert!(game.is_terminal() == true); //
        assert!(game.is_sampling() == false);
        assert!(game.is_everyone_alright() == true); //
        assert!(game.is_everyone_calling() == true); //
        assert!(game.is_everyone_touched() == true); //
        assert!(game.is_everyone_matched() == true); //
    }
}
