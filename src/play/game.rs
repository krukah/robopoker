use super::action::Action;
use super::payout::Payout;
use super::seat::Seat;
use super::seat::State;
use super::Chips;
use super::N;
use super::STACK;
use crate::cards::board::Board;
use crate::cards::card::Card;
use crate::cards::deck::Deck;
use crate::cards::hand::Hand;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::cards::strength::Strength;
use crate::play::continuation::Transition;
use crate::play::showdown::Showdown;
use crate::players::human::Human;

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
    player: Position,
}

impl Game {
    /// this will start the game at the first decision
    /// NOT the first action, which are blinds and hole cards dealt.
    /// stack size is always 100 and P1 is always dealer.
    /// these should not matter too much in the MCCFR algorithm,
    /// as long as we alternate the traverser/paths explored
    pub fn root() -> Self {
        let mut root = Self {
            chips: 0u16,
            dealer: 0usize,
            player: 0usize,
            board: Board::empty(),
            seats: [Seat::new(STACK); N],
        };
        root.rotate();
        root.deal_cards();
        root.post_blinds(Self::sblind());
        root.post_blinds(Self::bblind());
        root
    }

    /// for chance transitions, only bc of Preflop,
    /// we use an arbitrary (MIN) draw card
    /// it will be "coerced" into an Edge::Chance
    /// variant in the end anyway, in MCCFR
    ///
    /// actually we should not have chance transitions at preflop. Explorer::root will
    /// "spawn" us at a node where blinds are posted and hands are dealt!
    pub fn children(&self) -> Vec<(Game, Action)> {
        match self.chooser() {
            Transition::Terminal => self.ending_actions(),
            Transition::Chance(_) => self.chance_actions(),
            Transition::Choice(_) => self.choice_actions(),
        }
    }

    /// play against yourself in an infinite loop
    /// similar to children(), except a single decision action will come from
    /// Human::act() rather than all possible decision actions
    /// coming from self.options()
    pub fn play() -> ! {
        let mut node = Self::root();
        loop {
            match node.chooser() {
                Transition::Terminal => {
                    node.into_terminal(); // node.clone(); node...(&mut self) ; node = node
                }
                Transition::Chance(_) => {
                    node.show_revealed(); // node.clone(); node...(&mut self) ; node = node
                }
                Transition::Choice(_) => {
                    node.make_decision(); // node.clone(); node...(&mut self) ; node = node
                }
            }
        }
    }

    fn ending_actions(&self) -> Vec<(Game, Action)> {
        // just for symmetry, sorry, had to do it to 'em
        vec![]
    }
    fn chance_actions(&self) -> Vec<(Game, Action)> {
        let action = Action::Draw(Card::draw());
        let mut child = self.clone();
        child.show_revealed();
        vec![(child, action)]
    }
    fn choice_actions(&self) -> Vec<(Game, Action)> {
        self.options()
            .into_iter()
            .inspect(|action| assert!(!matches!(action, Action::Draw(_) | Action::Blind(_))))
            .map(|action| (self.spawn(action), action))
            .collect()
    }

    fn apply(&mut self, ref action: Action) {
        // assert!(self.options().contains(action));
        self.update_stdout(action);
        self.update_stacks(action);
        self.update_states(action);
        self.update_boards(action);
        self.update_player(action);
    }
    fn spawn(&self, action: Action) -> Self {
        let mut child = self.clone();
        child.apply(action);
        child
    }

    pub fn actor(&self) -> &Seat {
        self.actor_ref()
    }
    pub fn chips(&self) -> Chips {
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
            // for card in self.deck() {}
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
        if self.can_check() {
            options.push(Action::Check);
        }
        if self.can_fold() {
            options.push(Action::Fold);
        }
        options
    }
    pub fn chooser(&self) -> Transition {
        if self.is_terminal() {
            Transition::Terminal
        } else if self.is_sampling() {
            Transition::Chance(self.board.street().next())
        } else if self.is_decision() {
            Transition::Choice(self.actor_relative_idx())
        } else {
            unreachable!("game rules violated")
        }
    }

    fn deck(&self) -> Deck {
        let mut removed = Hand::from(self.board);
        for seat in self.seats.iter() {
            let hole = Hand::from(seat.cards());
            removed = Hand::add(removed, hole);
        }
        Deck::from(removed.complement())
    }
    fn actor_relative_idx(&self) -> Position {
        assert!(self.seats.len() == N);
        self.player.wrapping_sub(self.dealer) % N
    }
    fn actor_absolute_idx(&self) -> Position {
        assert!(self.seats.len() == N);
        (self.dealer + self.player) % N
    }
    fn actor_ref(&self) -> &Seat {
        let index = self.actor_absolute_idx();
        self.seats
            .get(index)
            .expect("index should be in bounds bc modulo")
    }
    fn actor_mut(&mut self) -> &mut Seat {
        let index = self.actor_absolute_idx();
        self.seats
            .get_mut(index)
            .expect("index should be in bounds bc modulo")
    }

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
    fn update_player(&mut self, action: &Action) {
        match action {
            Action::Draw(_) => {}
            _ => self.rotate(),
        }
    }
    fn update_stdout(&self, action: &Action) {
        match action {
            Action::Draw(_) => {
                println!("  {}", action);
            }
            _ => {
                println!("{} {}", self.actor_absolute_idx(), action);
            }
        }
    }
    fn rotate(&mut self) {
        'left: loop {
            self.player += 1;
            match self.actor_ref().state() {
                State::Playing => break 'left,
                State::Folding => continue 'left,
                State::Shoving => continue 'left,
            }
        }
    }

    //
    fn into_terminal(&mut self) {
        assert!(self.seats.iter().all(|s| s.stack() > 0), "game over");
        // conclusion of this hand
        self.give_chips();
        // beginning of next hand
        self.wipe_board();
        self.deal_cards();
        self.move_button();
        self.post_blinds(Self::sblind());
        self.post_blinds(Self::bblind());
    }
    fn give_chips(&mut self) {
        println!("::::::::::::::");
        println!("{}", self.board());
        for (i, (settlement, seat)) in self
            .settlement()
            .iter()
            .zip(self.seats.iter_mut())
            .enumerate()
        {
            println!("{} {} {:>7} {}", i, seat.cards(), seat.stack(), settlement);
            seat.win(settlement.reward);
        }
        println!();
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
            seat.set_state(State::Playing);
            seat.set_cards(deck.hole());
            seat.set_stake();
            seat.set_spent();
        }
    }
    fn move_button(&mut self) {
        assert!(self.seats.len() == N);
        assert!(self.board.street() == Street::Pref);
        self.dealer += 1;
        self.dealer %= N;
        self.player = 0;
        self.rotate();
    }
    fn post_blinds(&mut self, blind: Chips) {
        assert!(self.board.street() == Street::Pref);
        let stack = self.actor_ref().stack();
        if blind < stack {
            self.apply(Action::Blind(blind))
        } else {
            self.apply(Action::Shove(stack))
        }
    }

    //
    fn show_revealed(&mut self) {
        println!("{}", self.board.street().next());
        self.player = 0;
        self.rotate();
        self.next_street_public();
        self.next_street_stacks();
    }
    fn next_street_public(&mut self) {
        let mut deck = self.deck();
        match self.board.street() {
            Street::Rive => unreachable!("terminal"),
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
    fn make_decision(&mut self) {
        self.apply(Human::act(&self));
    }

    //
    fn is_terminal(&self) -> bool {
        self.board.street() == Street::Rive && self.is_everyone_waiting()
            || self.is_everyone_folding()
    }
    fn is_sampling(&self) -> bool {
        self.board.street() != Street::Rive && self.is_everyone_waiting()
    }
    fn is_decision(&self) -> bool {
        assert!(!self.is_terminal());
        assert!(!self.is_sampling());
        assert!(self.actor().state() == State::Playing);
        true
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
            > match self.board.street() {
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
        self.to_shove() > 0 && false
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
    pub fn settlement(&self) -> Vec<Payout> {
        assert!(self.is_terminal());
        Showdown::from(self.ledger()).settle()
    }
    fn ledger(&self) -> Vec<Payout> {
        self.seats
            .iter()
            .map(|seat| self.entry(seat))
            .collect::<Vec<Payout>>()
    }
    fn entry(&self, seat: &Seat) -> Payout {
        Payout {
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
