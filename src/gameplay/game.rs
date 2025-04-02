#![allow(dead_code)]

use super::action::Action;
use super::seat::Seat;
use super::seat::State;
use super::settlement::Settlement;
use super::showdown::Showdown;
use super::turn::Turn;
use crate::cards::board::Board;
use crate::cards::deck::Deck;
use crate::cards::hand::Hand;
use crate::cards::hole::Hole;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::cards::strength::Strength;
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
    pot: Chips,
    board: Board, // could be [Card; N] to preserve deal order
    dealer: Position,
    ticker: Position,
}

impl crate::cfr::traits::game::Game for Game {
    type E = crate::gameplay::edge::Edge;
    type T = crate::gameplay::turn::Turn;
    fn root() -> Self {
        Self::root()
    }
    fn turn(&self) -> Self::T {
        self.turn()
    }
    fn apply(&self, edge: Self::E) -> Self {
        self.apply(self.actionize(&edge))
    }
    fn payoff(&self, turn: Self::T) -> crate::Utility {
        self.settlements()
            .get(turn.position())
            .map(|settlement| settlement.pnl() as crate::Utility)
            .expect("player index in bounds")
    }
}

impl Game {
    pub fn base() -> Self {
        Self {
            pot: Chips::from(0i16),
            board: Board::empty(),
            seats: [Seat::from(STACK); N],
            dealer: 0usize,
            ticker: 1usize,
        }
    }
    pub fn deal(mut self) -> Self {
        self.deal_cards();
        self
    }
    pub fn post(mut self) -> Self {
        self.act(Action::Blind(self.to_post()));
        self.act(Action::Blind(self.to_post()));
        self
    }
    pub fn wipe(mut self, hole: Hole) -> Self {
        for seat in self.seats.iter_mut() {
            seat.reset_cards(hole);
        }
        self
    }
    /// this will start the game at the first decision
    /// NOT the first action, which are blinds and hole cards dealt.
    /// stack size is always 100 and P1 is always dealer.
    /// these should not matter too much in the MCCFR algorithm,
    /// as long as we alternate the traverser/paths explored
    pub fn root() -> Self {
        Self::base().deal().post()
    }
    pub fn blinds() -> Vec<Action> {
        vec![Action::Blind(Self::sblind()), Action::Blind(Self::bblind())]
    }
    pub fn n(&self) -> usize {
        N
    }
    pub fn apply(&self, action: Action) -> Self {
        let mut child = self.clone();
        child.act(action);
        child
    }
    //
    pub fn pot(&self) -> Chips {
        self.pot
    }
    pub fn board(&self) -> Board {
        self.board
    }
    pub fn turn(&self) -> Turn {
        if self.must_stop() {
            Turn::Terminal
        } else if self.must_deal() {
            Turn::Chance
        } else {
            Turn::Choice(self.actor_idx())
        }
    }
    pub fn actor(&self) -> &Seat {
        self.actor_ref()
    }
    pub fn sweat(&self) -> Observation {
        Observation::from((
            Hand::from(self.actor().cards()), //
            Hand::from(self.board()),         //
        ))
    }
    pub fn street(&self) -> Street {
        self.board.street()
    }
    pub fn legal(&self) -> Vec<Action> {
        let mut options = Vec::new();
        if self.must_stop() {
            return options;
        }
        if self.must_deal() {
            options.push(Action::Draw(self.deck().deal(self.street())));
            return options;
        }
        if self.must_post() {
            options.push(Action::Blind(Self::sblind()));
            return options;
        }
        if self.may_raise() {
            options.push(Action::Raise(self.to_raise()));
        }
        if self.may_shove() {
            options.push(Action::Shove(self.to_shove()));
        }
        if self.may_call() {
            options.push(Action::Call(self.to_call()));
        }
        if self.may_fold() {
            options.push(Action::Fold);
        }
        if self.may_check() {
            options.push(Action::Check);
        }
        assert!(options.len() > 0);
        options
    }

    //
    pub fn is_allowed(&self, action: &Action) -> bool {
        if self.must_stop() {
            return false;
        }
        match action {
            Action::Raise(raise) => {
                self.may_raise()
                    && raise.clone() >= self.to_raise()
                    && raise.clone() <= self.to_shove() - 1
            }
            Action::Draw(cards) => {
                self.must_deal()
                    && cards.clone().all(|c| self.deck().contains(&c))
                    && cards.count() == self.board().street().n_revealed()
            }
            Action::Blind(_) => self.must_post(),
            _ => self.legal().contains(action),
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
        self.act(Action::Blind(self.to_post()));
        self.act(Action::Blind(self.to_post()));
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
        self.pot = 0;
        self.board.clear();
        assert!(self.street() == Street::Pref);
    }
    fn move_button(&mut self) {
        assert!(self.seats.len() == self.n());
        assert!(self.street() == Street::Pref);
        self.dealer += 1;
        self.dealer %= self.n();
        self.ticker = self.dealer;
        self.next_player();
    }
    fn deal_cards(&mut self) {
        assert!(self.street() == Street::Pref);
        let mut deck = Deck::new();
        for seat in self.seats.iter_mut() {
            seat.reset_state(State::Betting);
            seat.reset_cards(deck.hole());
            seat.reset_stake();
            seat.reset_spent();
        }
    }

    //
    fn act(&mut self, a: Action) {
        assert!(self.is_allowed(&a));
        match a {
            Action::Check => {
                self.next_player();
            }
            Action::Fold => {
                self.fold();
                self.next_player();
            }
            Action::Call(chips)
            | Action::Blind(chips)
            | Action::Raise(chips)
            | Action::Shove(chips) => {
                self.bet(chips);
                self.next_player();
            }
            Action::Draw(cards) => {
                self.show(cards);
                self.next_player();
                self.next_street();
            }
        }
    }
    fn bet(&mut self, bet: Chips) {
        assert!(self.actor_ref().stack() >= bet);
        self.pot += bet;
        self.actor_mut().bet(bet);
        if self.actor_ref().stack() == 0 {
            self.shove();
        }
    }
    fn shove(&mut self) {
        self.actor_mut().reset_state(State::Shoving);
    }
    fn fold(&mut self) {
        self.actor_mut().reset_state(State::Folding);
    }
    fn show(&mut self, hand: Hand) {
        self.ticker = self.dealer;
        self.board.add(hand);
    }
    fn next_street(&mut self) {
        for seat in self.seats.iter_mut() {
            seat.reset_stake();
        }
    }
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

    /// we're waiting for showdown or everyone folded
    fn must_stop(&self) -> bool {
        if self.street() == Street::Rive {
            self.is_everyone_alright()
        } else {
            self.is_everyone_folding()
        }
    }
    /// we're waiting for a card to be revealed
    fn must_deal(&self) -> bool {
        if self.street() == Street::Rive {
            false
        } else {
            self.is_everyone_alright()
        }
    }
    /// blinds have not yet been posted // TODO some edge case of all in blinds
    fn must_post(&self) -> bool {
        if self.street() == Street::Pref {
            self.pot() < Self::sblind() + Self::bblind()
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
        self.ticker > self.n() + if self.street() == Street::Pref { 2 } else { 0 }
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
    fn may_fold(&self) -> bool {
        self.to_call() > 0
    }
    fn may_call(&self) -> bool {
        self.may_fold() && self.to_call() < self.to_shove()
    }
    fn may_check(&self) -> bool {
        self.effective_stake() == self.actor_ref().stake()
    }
    fn may_raise(&self) -> bool {
        self.to_raise() < self.to_shove()
    }
    fn may_shove(&self) -> bool {
        self.to_shove() > 0
    }

    //
    pub fn to_call(&self) -> Chips {
        self.effective_stake() - self.actor_ref().stake()
    }
    pub fn to_post(&self) -> Chips {
        assert!(self.street() == Street::Pref);
        match (self.ticker as isize - self.dealer as isize) % self.n() as isize {
            1 => Self::sblind().min(self.actor_ref().stack()),
            _ => Self::bblind().min(self.actor_ref().stack()),
        }
    }
    pub fn to_shove(&self) -> Chips {
        self.actor_ref().stack()
    }
    pub fn to_raise(&self) -> Chips {
        let (most_large_stake, next_large_stake) = self
            .seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .map(|s| s.stake())
            .fold((0, 0), |(most, next), stake| {
                if stake > most {
                    (stake, most)
                } else if stake > next {
                    (most, stake)
                } else {
                    (most, next)
                }
            });
        let relative_raise = most_large_stake - self.actor().stake();
        let marginal_raise = most_large_stake - next_large_stake;
        let required_raise = std::cmp::max(marginal_raise, Self::bblind());
        relative_raise + required_raise
    }

    //
    pub fn settlements(&self) -> Vec<Settlement> {
        assert!(self.must_stop(), "non terminal game state:\n{}", self);
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
    pub fn draw(&self) -> Hand {
        self.deck().deal(self.street())
    }
    pub fn deck(&self) -> Deck {
        let mut removed = Hand::from(self.board);
        for seat in self.seats.iter() {
            let hole = Hand::from(seat.cards());
            removed = Hand::add(removed, hole);
        }
        Deck::from(removed.complement())
    }
    fn actor_idx(&self) -> Position {
        (self.dealer + self.ticker) % self.n()
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

    pub const fn bblind() -> Chips {
        crate::B_BLIND
    }
    pub const fn sblind() -> Chips {
        crate::S_BLIND
    }
}

impl From<Game> for String {
    fn from(game: Game) -> Self {
        format!(" @ {:>6} {} {}", game.pot, game.board, game.street())
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for seat in self.seats.iter() {
            write!(f, "{}{:<6}", seat.state(), seat.stack())?;
        }
        #[cfg(feature = "native")]
        {
            use colored::Colorize;
            write!(f, "{}", String::from(*self).bright_green())
        }
        #[cfg(not(feature = "native"))]
        {
            write!(f, "{}", String::from(*self))
        }
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
        assert!(game.pot() == Game::sblind() + Game::bblind());
    }

    #[test]
    fn everyone_folds_pref() {
        let game = Game::root();
        let game = game.apply(Action::Fold);
        assert!(game.is_everyone_folding() == true);
        assert!(game.is_everyone_alright() == true);
        assert!(game.is_everyone_calling() == false);
        assert!(game.must_deal() == true); // ambiguous
        assert!(game.must_stop() == true);
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
        assert!(game.must_deal() == true); // ambiguous
        assert!(game.must_stop() == true);
    }
    #[test]
    fn history_of_checks() {
        // Blinds
        let game = Game::root();
        assert!(game.board().street() == Street::Pref);
        assert!(game.pot() == 3);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == false);
        assert!(game.is_everyone_matched() == false);

        // SmallB Preflop
        let game = game.apply(Action::Call(1));
        assert!(game.board().street() == Street::Pref);
        assert!(game.pot() == 4); //
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == false);
        assert!(game.is_everyone_matched() == true); //

        // Dealer Preflop
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Pref);
        assert!(game.pot() == 4);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == true); //
        assert!(game.is_everyone_alright() == true); //
        assert!(game.is_everyone_calling() == true); //
        assert!(game.is_everyone_touched() == true); //
        assert!(game.is_everyone_matched() == true);

        // Flop
        let flop = game.deck().deal(game.board().street());
        let game = game.apply(Action::Draw(flop));
        assert!(game.board().street() == Street::Flop); //
        assert!(game.pot() == 4);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == false); //
        assert!(game.is_everyone_alright() == false); //
        assert!(game.is_everyone_calling() == false); //
        assert!(game.is_everyone_touched() == false); //
        assert!(game.is_everyone_matched() == true);

        // SmallB Flop
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Flop);
        assert!(game.pot() == 4);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == false);
        assert!(game.is_everyone_matched() == true);

        // Dealer Flop
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Flop);
        assert!(game.pot() == 4);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == true); //
        assert!(game.is_everyone_alright() == true); //
        assert!(game.is_everyone_calling() == true); //
        assert!(game.is_everyone_touched() == true); //
        assert!(game.is_everyone_matched() == true);

        // Turn
        let turn = game.deck().deal(game.board().street());
        let game = game.apply(Action::Draw(turn));
        assert!(game.board().street() == Street::Turn);
        assert!(game.pot() == 4);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == false); //
        assert!(game.is_everyone_alright() == false); //
        assert!(game.is_everyone_calling() == false); //
        assert!(game.is_everyone_touched() == false); //
        assert!(game.is_everyone_matched() == true);

        // SmallB Turn
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Turn);
        assert!(game.pot() == 4);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == false);
        assert!(game.is_everyone_matched() == true);

        // Dealer Turn
        let game = game.apply(Action::Raise(4));
        assert!(game.board().street() == Street::Turn);
        assert!(game.pot() == 8);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == true); //
        assert!(game.is_everyone_matched() == false); //

        // SmallB Turn
        let game = game.apply(Action::Call(4));
        assert!(game.board().street() == Street::Turn);
        assert!(game.pot() == 12); //
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == true); //
        assert!(game.is_everyone_alright() == true); //
        assert!(game.is_everyone_calling() == true); //
        assert!(game.is_everyone_touched() == true);
        assert!(game.is_everyone_matched() == true);

        // River
        let rive = game.deck().deal(game.board().street());
        let game = game.apply(Action::Draw(rive));
        assert!(game.board().street() == Street::Rive); //
        assert!(game.pot() == 12);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == false); //
        assert!(game.is_everyone_alright() == false); //
        assert!(game.is_everyone_calling() == false); //
        assert!(game.is_everyone_touched() == false); //
        assert!(game.is_everyone_matched() == true); //

        // SmallB River
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Rive);
        assert!(game.pot() == 12);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == false);
        assert!(game.must_deal() == false);
        assert!(game.is_everyone_alright() == false);
        assert!(game.is_everyone_calling() == false);
        assert!(game.is_everyone_touched() == false);
        assert!(game.is_everyone_matched() == true);

        // Dealer River
        let game = game.apply(Action::Check);
        assert!(game.board().street() == Street::Rive);
        assert!(game.pot() == 12);
        assert!(game.must_post() == false);
        assert!(game.must_stop() == true); //
        assert!(game.must_deal() == false);
        assert!(game.is_everyone_alright() == true); //
        assert!(game.is_everyone_calling() == true); //
        assert!(game.is_everyone_touched() == true); //
        assert!(game.is_everyone_matched() == true); //
    }
}

// odds and tree building stuff

impl Game {
    /// convert an Edge into an Action by using Game state to
    /// determine free parameters (stack size, pot size, etc)
    ///
    /// NOTE
    /// this conversion is not injective, as multiple edges may
    /// represent the same action. moreover, we "snap" raises to be
    /// within range of legal bet sizes, so sometimes Raise(5:1) yields
    /// an identical Game node as Raise(1:1) or Shove.
    pub fn actionize(&self, edge: &crate::gameplay::edge::Edge) -> Action {
        let game = self;
        match &edge {
            crate::gameplay::edge::Edge::Check => Action::Check,
            crate::gameplay::edge::Edge::Fold => Action::Fold,
            crate::gameplay::edge::Edge::Draw => Action::Draw(game.draw()),
            crate::gameplay::edge::Edge::Call => Action::Call(game.to_call()),
            crate::gameplay::edge::Edge::Shove => Action::Shove(game.to_shove()),
            crate::gameplay::edge::Edge::Raise(odds) => {
                let min = game.to_raise();
                let max = game.to_shove();
                let pot = game.pot() as crate::Utility;
                let odd = crate::Utility::from(*odds);
                let bet = (pot * odd) as Chips;
                match bet {
                    bet if bet >= max => Action::Shove(max),
                    bet if bet <= min => Action::Raise(min),
                    _ => Action::Raise(bet),
                }
            }
        }
    }

    pub fn edgify(&self, action: Action) -> crate::gameplay::edge::Edge {
        use crate::gameplay::edge::Edge;
        use crate::gameplay::odds::Odds;
        match action {
            Action::Fold => Edge::Fold,
            Action::Check => Edge::Check,
            Action::Draw(_) => Edge::Draw,
            Action::Shove(_) => Edge::Shove,
            Action::Blind(_) | Action::Call(_) => Edge::Call,
            Action::Raise(amount) => Edge::Raise(Odds::nearest((amount, self.pot()))),
        }
    }
}
