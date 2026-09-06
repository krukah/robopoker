use super::*;
use deuce::*;
use pokerkit::Translated;
use pokerkit::*;
use std::ops::Not;

/// The memoryless state machine for a No-Limit Hold'em hand.
///
/// Deliberately memoryless: only the current state, not the history that reached
/// it, so the same CFR node can be arrived at via different action sequences.
/// Transitions are functional — [`apply`](Self::apply) returns a new `GameN`, so
/// tree traversal needs no undo logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameN<const P: usize> {
    pot: Chips,
    board: Board,
    seats: [Seat; P],
    dealer: Position,
    ticker: Position,
}

/// Heads-up game (the default configuration).
pub type Game = GameN<{ N }>;
/// Two-player game.
pub type HeadsUp = GameN<2>;
/// Six-player game.
pub type FunTable = GameN<6>;
/// Ten-player game.
pub type NitTable = GameN<10>;

impl<const P: usize> Default for GameN<P> {
    fn default() -> Self {
        Self::preblind(0, [STACK; P])
    }
}

/// Game tree entry points.
impl<const P: usize> GameN<P> {
    /// Deals random hole cards to each player but does NOT post blinds.
    pub fn preblind(dealer: Position, stacks: [Chips; P]) -> Self {
        let mut deck = Deck::new();
        Self {
            pot: 0,
            board: Board::empty(),
            seats: std::array::from_fn(|i| Seat::from((deck.hole(), stacks[i]))),
            dealer,
            ticker: usize::from(P != 2),
        }
    }
    /// Canonical MCCFR root: blinds posted, 100bb stacks, P0 on the button.
    pub fn root() -> Self {
        let mut game = Self::default();
        game.act(game.posts());
        game.act(game.posts());
        game
    }
    /// Creates a game with custom dealer and stacks, posts blinds.
    pub fn from_start(dealer: Position, stacks: [Chips; P]) -> Self {
        let mut game = Self::preblind(dealer, stacks);
        game.act(game.posts());
        game.act(game.posts());
        game
    }
    /// Sets a specific seat's hole cards.
    pub fn deal(mut self, position: Position, hole: Hole) -> Self {
        self.seats[position].reset_cards(hole);
        self
    }
    /// Replaces all players' hole cards with the given hand.
    pub fn wipe(mut self, hole: Hole) -> Self {
        for seat in &mut self.seats {
            seat.reset_cards(hole);
        }
        self
    }
    /// Replaces all players' hole cards EXCEPT the given seat.
    ///
    /// Seat identity is position-indexed (`Turn::Choice(i)` matches seat `i`),
    /// independent of dealer button rotation.
    pub fn fix(mut self, hero: Turn, hole: Hole) -> Self {
        self.seats
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| Turn::Choice(*i) != hero)
            .for_each(|(_, seat)| seat.reset_cards(hole));
        self
    }
    /// Fast-forward to `target` by repeatedly applying `passive()`.
    ///
    /// Used to construct subgame roots at arbitrary streets. Panics if the
    /// target has passed, or if the hand goes terminal/all-in en route.
    pub fn ffwd(mut self, target: Street) -> Self {
        while self.street() < target {
            match self.turn() {
                Turn::Terminal => panic!("reached terminal before target street"),
                Turn::Chance => {
                    let action = self.reveal();
                    self.act(action);
                }
                Turn::Choice(_) => {
                    let action = self.passive();
                    self.act(action);
                }
            }
        }
        debug_assert_eq!(self.street(), target, "overshot target street");
        self
    }
}

/// Public state accessors.
impl<const P: usize> GameN<P> {
    /// Number of players (constant for heads-up).
    pub fn n(&self) -> usize {
        self.seats.len()
    }
    /// Total chips in the pot.
    pub fn pot(&self) -> Chips {
        self.pot
    }
    /// All player seats.
    pub fn seats(&self) -> [Seat; P] {
        self.seats
    }
    /// Community cards on the board.
    pub fn board(&self) -> Board {
        self.board
    }
    /// Determines whether it's a player's turn, chance node, or terminal.
    pub fn turn(&self) -> Turn {
        if self.must_stop() {
            Turn::Terminal
        } else if self.must_deal() {
            Turn::Chance
        } else {
            Turn::Choice(self.actor_idx())
        }
    }
    /// The seat of the player to act.
    pub fn actor(&self) -> &Seat {
        self.actor_ref()
    }
    /// The current observation from the acting player's perspective.
    pub fn sweat(&self) -> Observation {
        Observation::from((
            Hand::from(self.actor().cards()), //
            Hand::from(self.board()),         //
        ))
    }
    /// The observation from a specific player's perspective.
    pub fn sweat_at(&self, position: usize) -> Observation {
        Observation::from((Hand::from(self.seats[position].cards()), Hand::from(self.board())))
    }
    /// The dealer position as a turn.
    pub fn dealer(&self) -> Turn {
        Turn::Choice(self.dealer)
    }
    /// Current street based on board cards.
    pub fn street(&self) -> Street {
        self.board.street()
    }
    /// The acting player's Turn if it's a choice node (None for chance/terminal).
    pub fn player(&self) -> Option<Turn> {
        self.turn().is_choice().then_some(self.turn())
    }
    /// Stack sizes for all seats.
    pub fn stacks(&self) -> [Chips; P] {
        std::array::from_fn(|i| self.seats[i].stack())
    }
    /// Original stack sizes before current street bets.
    pub fn buyins(&self) -> [Chips; P] {
        std::array::from_fn(|i| self.seats[i].stack() + self.seats[i].stake())
    }
    /// Current street stakes for all seats.
    pub fn stakes(&self) -> [Chips; P] {
        std::array::from_fn(|i| self.seats[i].stake())
    }
    /// Player states for all seats (active/folding).
    pub fn states(&self) -> [State; P] {
        std::array::from_fn(|i| self.seats[i].state())
    }
    /// Community cards as a list (for display).
    pub fn board_cards(&self) -> Vec<Card> {
        Vec::<Card>::from(Hand::from(self.board()))
    }
}

/// Action validation and application.
impl<const P: usize> GameN<P> {
    /// Applies an action mutably and returns a clone of the new state.
    pub fn consume(&mut self, action: Action) -> Self {
        self.act(action);
        *self
    }
    /// Returns a new game state with the action applied. Panics if illegal.
    pub fn apply(&self, action: Action) -> Self {
        self.try_apply(action).expect("valid action")
    }
    /// Fallible version of [`apply`](Self::apply).
    pub fn try_apply(&self, action: Action) -> anyhow::Result<Self> {
        if !self.is_allowed(&action) {
            return Err(anyhow::anyhow!("illegal action {:?} in state {:?}", action, self.turn()));
        }
        let mut child = *self;
        child.act(action);
        Ok(child)
    }
    /// All legal actions: empty at terminal, exactly one at chance nodes.
    pub fn legal(&self) -> Vec<Action> {
        // action is determined if it's Turn::Chance
        if self.must_stop() {
            return vec![];
        }
        if self.must_deal() {
            return vec![self.reveal()];
        }
        if self.must_post() {
            return vec![self.posts()];
        }
        // now it's certainly a Turn::Choice
        let mut options = Vec::new();
        if self.may_raise() {
            options.push(self.raise());
        }
        if self.may_shove() {
            options.push(self.shove());
        }
        if self.may_call() {
            options.push(self.calls());
        }
        if self.may_fold() {
            options.push(self.folds());
        }
        if self.may_check() {
            options.push(self.check());
        }
        debug_assert!(!options.is_empty());
        options
    }
    /// Applies an action without validation, returning a new state.
    ///
    /// Bypasses `is_allowed()` for server-authoritative actions where
    /// the local game state may have placeholder cards (e.g., client
    /// receives a `Draw` but its random seat cards collide with board).
    pub fn force_apply(&self, action: Action) -> Self {
        let mut next = *self;
        next.force_act(action);
        next
    }
    /// Checks if a specific action is legal.
    pub fn is_allowed(&self, action: &Action) -> bool {
        // do "bounds checking" on the two actions with degrees of freedom;
        // Action::Raise is constrained by min/max raise
        // Action::Draw is constrained by the deck and the number of cards
        match action {
            Action::Raise(raise) => {
                self.may_raise()
                    && self.must_stop().not()
                    && self.must_deal().not()
                    && *raise >= self.to_raise()
                    && *raise < self.to_shove()
            }
            Action::Draw(cards) => {
                self.must_deal()
                    && self.must_stop().not()
                    && cards.clone().all(|c| self.deck().contains(&c))
                    && cards.count() == self.board().street().next().n_revealed()
            }
            other => self.legal().contains(other),
        }
    }
}

/// Hand-to-hand transitions.
impl<const P: usize> GameN<P> {
    /// Advances to the next hand, or `None` if a player can't cover the BB.
    pub fn continuation(mut self) -> Option<Self> {
        debug_assert_eq!(self.turn(), Turn::Terminal);
        self.settlements()
            .iter()
            .zip(self.seats())
            .all(|(s, seat)| seat.stack() + s.pnl().reward() >= Self::bblind())
            .then(|| {
                self.give_chips();
                self.wipe_board();
                self.wipe_seats();
                self.move_button();
                self.act(self.posts());
                self.act(self.posts());
                self
            })
    }

    fn give_chips(&mut self) {
        for (_, (settlement, seat)) in self
            .settlements()
            .iter()
            .zip(self.seats.iter_mut())
            .enumerate()
            .inspect(|(i, (x, s))| tracing::trace!("{} {} {:>7} {}", i, s.cards(), s.stack(), x.won()))
        {
            seat.win(settlement.pnl().reward());
        }
        self.pot = 0;
    }

    fn wipe_board(&mut self) {
        debug_assert_eq!(self.pot(), 0);
        self.board.clear();
    }

    fn wipe_seats(&mut self) {
        debug_assert_eq!(self.pot(), 0);
        debug_assert_eq!(self.street(), Street::Pref);
        let mut deck = Deck::new();
        for seat in &mut self.seats {
            seat.reset_state(State::Betting);
            seat.reset_cards(deck.hole());
            seat.reset_stake();
            seat.reset_spent();
        }
    }

    fn move_button(&mut self) {
        debug_assert_eq!(self.pot(), 0);
        debug_assert_eq!(self.seats.len(), self.n());
        debug_assert_eq!(self.street(), Street::Pref);
        self.dealer += 1;
        self.dealer %= self.n();
        self.ticker = usize::from(P != 2);
    }
}

/// Private mutation methods.
impl<const P: usize> GameN<P> {
    /// Core state transition logic.
    fn act(&mut self, a: Action) {
        debug_assert!(self.is_allowed(&a));
        self.force_act(a);
    }
    /// Core state transition without validation.
    fn force_act(&mut self, a: Action) {
        match a {
            Action::Check => {
                self.next_player();
            }
            Action::Fold => {
                self.fold();
                self.next_player();
            }
            Action::Call(chips) | Action::Blind(chips) | Action::Raise(chips) | Action::Shove(chips) => {
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
        debug_assert!(self.actor_ref().stack() >= bet);
        self.pot += bet;
        self.actor_mut().bet(bet);
        if self.actor_ref().stack() == 0 {
            self.allin();
        }
    }

    fn allin(&mut self) {
        self.actor_mut().reset_state(State::Shoving);
    }

    fn fold(&mut self) {
        self.actor_mut().reset_state(State::Folding);
    }

    fn show(&mut self, hand: Hand) {
        self.ticker = 0;
        self.board.add(hand);
    }
}

/// Street and player advancement.
impl<const P: usize> GameN<P> {
    /// Resets per-street stakes when a new street begins.
    fn next_street(&mut self) {
        for seat in &mut self.seats {
            seat.reset_stake();
        }
    }
    /// Advances to the next active player, skipping folded/all-in players.
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
}

/// Termination and continuation predicates.
impl<const P: usize> GameN<P> {
    /// True if the hand is complete (showdown or everyone folded).
    pub fn must_stop(&self) -> bool {
        if self.street() == Street::Rive {
            self.is_everyone_alright()
        } else {
            self.is_everyone_folding()
        }
    }
    /// True if we need to deal the next street's cards.
    pub fn must_deal(&self) -> bool {
        self.street() != Street::Rive && self.is_everyone_alright()
    }
    /// True if blinds have not yet been posted.
    pub fn must_post(&self) -> bool {
        self.street() == Street::Pref && self.pot() < Self::sblind() + Self::bblind()
    }
    /// All players have acted and the pot is right.
    fn is_everyone_alright(&self) -> bool {
        self.is_everyone_calling() || self.is_everyone_folding() || self.is_everyone_shoving()
    }
    /// All betting players are in for the same amount.
    fn is_everyone_calling(&self) -> bool {
        self.is_everyone_touched() && self.is_everyone_matched()
    }
    /// All players have acted at least once this street.
    ///
    /// The ticker counts *seat-steps* from the button, not actions:
    /// [`Self::next_player`] increments it once per seat while skipping
    /// folded/all-in players, so a seat's ticker offset is positional and
    /// fold-invariant. A street closes once the ticker has passed one full
    /// orbit beyond the street's last-option seat: postflop that is the
    /// button (offset 0 → threshold `n`); preflop it is the big blind, whose
    /// option must survive a limped pot (offset `1` heads-up where BB is
    /// dealer+1, else `2` → threshold `n + bb`). Combined with
    /// [`Self::is_everyone_matched`], this closes action exactly when it
    /// returns to the last aggressor: an intervening raise breaks `matched`
    /// and forces another orbit, while `touched` guarantees the BB option
    /// and every live seat's first look are never skipped.
    fn is_everyone_touched(&self) -> bool {
        let bb = if P == 2 { 1 } else { 2 };
        self.ticker > self.n() + if self.street() == Street::Pref { bb } else { 0 }
    }
    /// All betting players are in for the effective stake.
    fn is_everyone_matched(&self) -> bool {
        let stake = self.max_stake();
        self.seats
            .iter()
            .filter(|s| s.state() == State::Betting)
            .all(|s| s.stake() == stake)
    }
    /// All non-folded players are all-in.
    fn is_everyone_shoving(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .all(|s| s.state() == State::Shoving)
    }
    /// Exactly one player remains (all others folded).
    fn is_everyone_folding(&self) -> bool {
        self.seats.iter().filter(|s| s.state() != State::Folding).count() == 1
    }
    /// True if folding is a legal option (facing a bet).
    pub fn may_fold(&self) -> bool {
        matches!(self.turn(), Turn::Choice(_)) && self.to_call() > 0
    }
    /// True if calling is legal (facing a bet we can cover).
    pub fn may_call(&self) -> bool {
        matches!(self.turn(), Turn::Choice(_)) && self.may_fold() && self.to_call() < self.to_shove()
    }
    /// True if checking is legal (no bet to call).
    pub fn may_check(&self) -> bool {
        matches!(self.turn(), Turn::Choice(_)) && self.max_stake() == self.actor_ref().stake()
    }
    /// True if raising is legal (have chips beyond the min-raise).
    pub fn may_raise(&self) -> bool {
        matches!(self.turn(), Turn::Choice(_)) && self.to_raise() < self.to_shove()
    }
    /// True if shoving (all-in) is legal.
    pub fn may_shove(&self) -> bool {
        matches!(self.turn(), Turn::Choice(_)) && self.to_shove() > 0
    }
}

/// Bet sizing constraints and action constructors.
impl<const P: usize> GameN<P> {
    /// Chips needed to call the current bet.
    pub fn to_call(&self) -> Chips {
        self.max_stake() - self.actor_ref().stake()
    }
    /// Blind amount to post (SB or BB depending on pot).
    pub fn to_post(&self) -> Chips {
        debug_assert_eq!(self.street(), Street::Pref);
        if self.pot() < Self::sblind() {
            Self::sblind().min(self.actor_ref().stack())
        } else {
            Self::bblind().min(self.actor_ref().stack())
        }
    }
    /// All remaining chips (for all-in).
    pub fn to_shove(&self) -> Chips {
        self.actor_ref().stack()
    }
    /// Minimum legal raise size.
    ///
    /// Computed as: chips to call + max(last raise increment, big blind).
    pub fn to_raise(&self) -> Chips {
        let (most_large_stake, next_large_stake) = self
            .seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .map(Seat::stake)
            .fold((0, 0), Self::podium);
        let relative_raise = most_large_stake - self.actor().stake();
        let marginal_raise = most_large_stake - next_large_stake;
        let required_raise = std::cmp::max(marginal_raise, Self::bblind());
        relative_raise + required_raise
    }
    /// Constructs a minimum-raise action.
    pub fn raise(&self) -> Action {
        Action::Raise(self.to_raise())
    }
    /// Constructs an all-in action.
    pub fn shove(&self) -> Action {
        Action::Shove(self.to_shove())
    }
    /// Constructs a call action.
    pub fn calls(&self) -> Action {
        Action::Call(self.to_call())
    }
    /// Constructs a blind-posting action.
    pub fn posts(&self) -> Action {
        Action::Blind(self.to_post())
    }
    /// Constructs a fold action.
    pub fn folds(&self) -> Action {
        Action::Fold
    }
    /// Constructs a check action.
    pub fn check(&self) -> Action {
        Action::Check
    }
    /// Returns check if allowed, otherwise fold.
    pub fn passive(&self) -> Action {
        if self.may_check() { Action::Check } else { Action::Fold }
    }
    /// Deals the next street's cards from the deck.
    pub fn reveal(&self) -> Action {
        Action::Draw(self.deck().deal(self.street()))
    }
}

/// Showdown and payout logic.
impl<const P: usize> GameN<P> {
    /// Computes final chip distributions at a terminal node.
    pub fn settlements(&self) -> Vec<Settlement> {
        debug_assert!(self.must_stop(), "non terminal game state:\n{self}");
        Showdown::from(self.ledger()).settle()
    }
    /// Returns true if this is a showdown (multiple players remain).
    pub fn is_showdown(&self) -> bool {
        self.seats.iter().filter(|s| s.state().is_active()).count() > 1
    }

    fn ledger(&self) -> Vec<Settlement> {
        self.seats
            .iter()
            .enumerate()
            .map(|(position, _)| self.settlement(position))
            .collect()
    }

    fn settlement(&self, position: usize) -> Settlement {
        let seat = &self.seats[position];
        let strength = Strength::from(Hand::add(Hand::from(seat.cards()), Hand::from(self.board())));
        Settlement::from((seat.spent(), seat.state(), strength))
    }
}

/// Card operations.
impl<const P: usize> GameN<P> {
    /// Deals random cards for the next street.
    pub fn draw(&self) -> Hand {
        self.deck().deal(self.street())
    }
    /// Returns the remaining deck (all cards not in play).
    pub fn deck(&self) -> Deck {
        let mut removed = Hand::from(self.board);
        for seat in &self.seats {
            removed = Hand::or(removed, Hand::from(seat.cards()));
        }
        Deck::from(removed.complement())
    }
}

/// Position tracking.
impl<const P: usize> GameN<P> {
    /// Index of the player to act.
    fn actor_idx(&self) -> Position {
        (self.dealer + self.ticker) % self.n()
    }

    fn actor_ref(&self) -> &Seat {
        self.seats
            .get(self.actor_idx())
            .expect("index should be in bounds bc modulo")
    }

    fn actor_mut(&mut self) -> &mut Seat {
        let index = self.actor_idx();
        self.seats.get_mut(index).expect("index should be in bounds bc modulo")
    }
}

/// Stack and SPR calculations.
impl<const P: usize> GameN<P> {
    /// Total chips in play (pot + all stacks).
    pub fn total(&self) -> Chips {
        self.pot() + self.seats().iter().map(Seat::stack).sum::<Chips>()
    }
    /// Effective stack: the second-largest stack among live (non-folded)
    /// players — the most chips that can actually be put at risk, since the
    /// largest stack has no one left to cover the excess. Heads-up this is
    /// exactly the smaller of the two, so behavior there is unchanged. With
    /// fewer than two live players (terminal-ish states) nothing more can be
    /// contested, so it returns 0.
    pub fn effective(&self) -> Chips {
        self.seats
            .iter()
            .filter(|s| s.state() != State::Folding)
            .map(Seat::stack)
            .fold((0, 0), Self::podium)
            .1
            .max(0)
    }

    /// Folds a value into a running (largest, second-largest) pair.
    fn podium((most, next): (Chips, Chips), value: Chips) -> (Chips, Chips) {
        if value > most {
            (value, most)
        } else if value > next {
            (most, value)
        } else {
            (most, next)
        }
    }
    /// Stack-to-pot ratio (effective stack / pot).
    pub fn spr(&self) -> f32 {
        match self.pot() {
            0 => 0.0,
            p => self.effective() as f32 / p as f32,
        }
    }
    /// Maximum stake among all players this street.
    fn max_stake(&self) -> Chips {
        self.seats.iter().map(Seat::stake).max().expect("non-empty seats")
    }
    /// True if this is a preflop opening spot (no player actions yet).
    /// Used to interpret Odds(n,1) as nBB rather than nx pot.
    #[allow(dead_code)]
    fn is_opening(&self) -> bool {
        self.street() == Street::Pref && self.pot() == Self::sblind() + Self::bblind()
    }
}

/// Blind configuration.
impl<const P: usize> GameN<P> {
    /// Returns the blind posting actions [SB, BB].
    pub const fn blinds() -> [Action; 2] {
        [Action::Blind(Self::sblind()), Action::Blind(Self::bblind())]
    }
    /// Big blind size.
    pub const fn bblind() -> Chips {
        pokerkit::B_BLIND
    }
    /// Small blind size.
    pub const fn sblind() -> Chips {
        pokerkit::S_BLIND
    }
}

/// Abstraction interface: mapping between concrete Actions and abstract Edges.
impl<const P: usize> GameN<P> {
    /// Returns all available edges for current game state.
    /// Expands legal actions into the discretized edge space.
    pub fn choices(&self, depth: usize) -> Path {
        self.legal()
            .into_iter()
            .flat_map(|action| self.unfold(depth, action))
            .collect()
    }
    /// Expands an action into edges using the street/depth bet grid.
    /// Non-raise actions map 1:1; raises expand to all grid sizes
    /// available at this `(street, depth)` cell.
    fn unfold(&self, depth: usize, action: Action) -> Vec<Edge> {
        match action {
            Action::Raise(_) => Edge::raises(self.street(), depth),
            _ => vec![Edge::from(action)],
        }
    }
    /// Converts an abstract [`Edge`] into a concrete [`Action`].
    /// The resulting action may be illegal; use [`Self::snap`] to coerce.
    pub fn actionize(&self, edge: Edge) -> Action {
        match edge {
            Edge::Fold => Action::Fold,
            Edge::Draw => self.reveal(),
            Edge::Call => Action::Call(self.to_call()),
            Edge::Check => Action::Check,
            Edge::Shove => Action::Shove(self.to_shove()),
            Edge::Open(n) => Action::Raise(n * pokerkit::B_BLIND),
            Edge::Raise(_) => Action::Raise(edge.into_chips(self.pot())),
        }
    }
    /// Converts a concrete [`Action`] into an abstract [`Edge`].
    /// Raise amounts snap to the closest grid size; other actions map directly.
    pub fn edgify(&self, action: Action, depth: usize) -> Edge {
        match action {
            Action::Fold => Edge::Fold,
            Action::Check => Edge::Check,
            Action::Draw(_) => Edge::Draw,
            Action::Call(_) => Edge::Call,
            Action::Blind(_) => Edge::Call,
            Action::Shove(_) => Edge::Shove,
            Action::Raise(chips) => self.snap_to_edge(chips, depth),
        }
    }
    /// Snap-only translation for the canonical history walk.
    ///
    /// Panics on the [`Translated::Free`] arm — no current [`Translation`]
    /// variant emits it; a future Brown-style variant needs a custom walker.
    pub fn snapped<R>(&self, depth: usize, action: Action, policy: &Translation, rng: &mut R) -> Edge
    where
        R: rand::Rng + ?Sized,
    {
        match self.translate(action, depth, policy, rng) {
            Translated::Snap(edge) => edge,
            Translated::Free(_) => unreachable!(
                "no current Translation variant emits Translated::Free; \
                 add a custom history walker for any future \
                 off-tree-emitting translation",
            ),
        }
    }

    /// Universal action-translation hook: returns either an on-tree [`Edge`]
    /// or an off-tree [`Action`] (the original raise amount, only emitted by
    /// injection-style policies like `Exact` or `EpsilonPrune`).
    ///
    /// [`Self::edgify`] is the [`Translation::Snap`] shorthand.
    pub fn translate<R>(
        &self,
        action: Action,
        depth: usize,
        policy: &Translation,
        rng: &mut R,
    ) -> Translated<Edge, Action>
    where
        R: rand::Rng + ?Sized,
    {
        match action {
            Action::Fold => Translated::Snap(Edge::Fold),
            Action::Check => Translated::Snap(Edge::Check),
            Action::Draw(_) => Translated::Snap(Edge::Draw),
            Action::Call(_) => Translated::Snap(Edge::Call),
            Action::Blind(_) => Translated::Snap(Edge::Call),
            Action::Shove(_) => Translated::Snap(Edge::Shove),
            Action::Raise(chips) => {
                // depth > MAX_RAISE_REPEATS carries an empty abstract raise
                // grid — the only legal aggressive action there is
                // Edge::Shove. Mirror snap_to_edge's `.unwrap_or(Edge::Shove)`
                // semantic instead of asking Size::translate to invent a
                // Size that doesn't exist.
                if Edge::raises(self.street(), depth).is_empty() {
                    return Translated::Snap(Edge::Shove);
                }
                match Size::translate(Raise::new(chips, self.pot(), self.street(), depth), policy, rng) {
                    Translated::Snap(Size::BBs(n)) => Translated::Snap(Edge::Open(n)),
                    Translated::Snap(Size::SPR(n, d)) => Translated::Snap(Edge::Raise(Odds::new(n, d))),
                    // An off-grid raise at or above the all-in threshold is a
                    // shove — on-grid via Edge::Shove. Free is reserved for
                    // genuinely off-grid, non-shove raises the nesting player
                    // re-solves against.
                    Translated::Free(c) if c >= self.to_shove() => Translated::Snap(Edge::Shove),
                    Translated::Free(c) => Translated::Free(Action::Raise(c)),
                }
            }
        }
    }
    /// Snaps a chip amount to the nearest edge in the grid.
    fn snap_to_edge(&self, chips: Chips, depth: usize) -> Edge {
        Edge::raises(self.street(), depth)
            .into_iter()
            .min_by_key(|e| (e.into_chips(self.pot()) as i32 - chips as i32).abs())
            .unwrap_or(Edge::Shove)
    }
    /// Maps an action to the nearest legal action in the current state.
    ///
    /// Used for CFR traversal where canonical edges may not correspond to
    /// legal actions due to stack/pot differences from prior streets.
    /// Semi-recursive: aggressive actions cascade through the fallback chain
    /// `Raise → Shove → Call → passive`.
    pub fn snap(&self, action: Action) -> Action {
        match action {
            Action::Raise(x) if x >= self.to_shove() => self.snap(self.shove()), //
            Action::Raise(_) if !self.may_raise() => self.snap(self.shove()),    //
            Action::Raise(x) if x < self.to_raise() => self.raise(),             //
            Action::Raise(x) => Action::Raise(x),                                //
            Action::Shove(_) if self.may_shove() => self.shove(),                //
            Action::Shove(_) if self.may_call() => self.calls(),                 // ? unnecessary
            Action::Shove(_) => self.passive(),                                  // ? unreachable
            Action::Call(_) if self.may_call() => self.calls(),                  // ? unnecessary
            Action::Call(_) if self.may_shove() => self.shove(),                 // ? unnecessary
            Action::Call(_) => self.passive(),                                   // ? unnecessary
            Action::Check if self.may_check() => Action::Check,                  // ? self.passive()
            Action::Check if self.may_call() => self.calls(),                    // ? self.passive()
            Action::Check => self.folds(),                                       // ? self.passive()
            Action::Fold if self.may_fold() => Action::Fold,                     // ? self.passive()
            Action::Fold => Action::Check,                                       // ? self.passive()
            Action::Draw(_) | Action::Blind(_) => action,
        }
    }
}

impl<const P: usize> std::fmt::Display for GameN<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for seat in &self.seats {
            writeln!(f, "{:>3} {:>3} {:<6}", seat.state(), seat.cards(), seat.stack())?;
        }
        writeln!(f, "Pot   {}", self.pot())?;
        writeln!(f, "Board {}", self.board())?;
        Ok(())
    }
}
/// Infinite iterator over actions, resetting to a fresh game when busted.
///
/// Never terminates — use `.take(n)` to bound iteration.
pub struct Perpetual(Game);
impl Perpetual {
    pub fn new(game: Game) -> Self {
        Self(game)
    }
}
impl Iterator for Perpetual {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let actions = self.0.legal();
            if !actions.is_empty() {
                let action = actions[rand::random_range(0..actions.len())];
                self.0 = self.0.apply(action);
                return Some(action);
            }
            self.0 = self.0.continuation().unwrap_or_else(Game::root);
        }
    }
}

/// Iterator over terminal `Game` states, one per hand, stopping on bust.
pub struct Hands(Game);
impl Hands {
    pub fn new(game: Game) -> Self {
        Self(game)
    }
}
impl Iterator for Hands {
    type Item = Game;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.0.must_stop() {
            let actions = self.0.legal();
            let action = actions[rand::random_range(0..actions.len())];
            self.0 = self.0.apply(action);
        }
        let terminal = self.0;
        self.0 = self.0.continuation()?;
        Some(terminal)
    }
}

/// Iterator over actions across multiple hands, stopping on bust.
pub struct Session(Game);
impl Session {
    pub fn new(game: Game) -> Self {
        Self(game)
    }
}
impl Iterator for Session {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let actions = self.0.legal();
            if !actions.is_empty() {
                let action = actions[rand::random_range(0..actions.len())];
                self.0 = self.0.apply(action);
                return Some(action);
            }
            self.0 = self.0.continuation()?;
        }
    }
}

impl Game {
    /// Infinite iterator over actions, resetting on bust.
    pub fn perpetual(self) -> Perpetual {
        Perpetual::new(self)
    }
    /// Iterator over completed hands, stopping when busted.
    pub fn hands(self) -> Hands {
        Hands::new(self)
    }
    /// Iterator over actions, stopping when busted.
    pub fn session(self) -> Session {
        Session::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// dealer posts SB, non-dealer posts BB, dealer acts first after blinds
    #[test]
    fn test_root() {
        let game = Game::root();
        assert_eq!(game.board().street(), Street::Pref);
        assert_eq!(game.actor().state(), State::Betting);
        assert_eq!(game.pot(), Game::sblind() + Game::bblind());
        assert_eq!(game.turn(), Turn::Choice(game.dealer)); // dealer acts first
    }

    #[test]
    fn everyone_folds_pref() {
        let game = Game::root();
        let game = game.apply(Action::Fold);
        assert!(game.is_everyone_folding());
        assert!(game.is_everyone_alright());
        assert!(!game.is_everyone_calling());
        assert!(game.must_deal()); // ambiguous
        assert!(game.must_stop());
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
        assert!(game.is_everyone_folding());
        assert!(game.is_everyone_alright());
        assert!(!game.is_everyone_calling());
        assert!(game.must_deal()); // ambiguous
        assert!(game.must_stop());
    }

    #[test]
    fn history_of_checks() {
        // Blinds
        let game = Game::root();
        assert_eq!(game.board().street(), Street::Pref);
        assert_eq!(game.pot(), 3);
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(!game.must_deal());
        assert!(!game.is_everyone_alright());
        assert!(!game.is_everyone_calling());
        assert!(!game.is_everyone_touched());
        assert!(!game.is_everyone_matched());
        // SmallB Preflop
        let game = game.apply(Action::Call(1));
        assert_eq!(game.board().street(), Street::Pref);
        assert_eq!(game.pot(), 4); //
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(!game.must_deal());
        assert!(!game.is_everyone_alright());
        assert!(!game.is_everyone_calling());
        assert!(!game.is_everyone_touched());
        assert!(game.is_everyone_matched()); //
        // Dealer Preflop
        let game = game.apply(Action::Check);
        assert_eq!(game.board().street(), Street::Pref);
        assert_eq!(game.pot(), 4);
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(game.must_deal()); //
        assert!(game.is_everyone_alright()); //
        assert!(game.is_everyone_calling()); //
        assert!(game.is_everyone_touched()); //
        assert!(game.is_everyone_matched());
        // Flop
        let flop = game.deck().deal(game.board().street());
        let game = game.apply(Action::Draw(flop));
        assert_eq!(game.board().street(), Street::Flop); //
        assert_eq!(game.pot(), 4);
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(!game.must_deal()); //
        assert!(!game.is_everyone_alright()); //
        assert!(!game.is_everyone_calling()); //
        assert!(!game.is_everyone_touched()); //
        assert!(game.is_everyone_matched());
        // SmallB Flop
        let game = game.apply(Action::Check);
        assert_eq!(game.board().street(), Street::Flop);
        assert_eq!(game.pot(), 4);
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(!game.must_deal());
        assert!(!game.is_everyone_alright());
        assert!(!game.is_everyone_calling());
        assert!(!game.is_everyone_touched());
        assert!(game.is_everyone_matched());
        // Dealer Flop
        let game = game.apply(Action::Check);
        assert_eq!(game.board().street(), Street::Flop);
        assert_eq!(game.pot(), 4);
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(game.must_deal()); //
        assert!(game.is_everyone_alright()); //
        assert!(game.is_everyone_calling()); //
        assert!(game.is_everyone_touched()); //
        assert!(game.is_everyone_matched());
        // Turn
        let turn = game.deck().deal(game.board().street());
        let game = game.apply(Action::Draw(turn));
        assert_eq!(game.board().street(), Street::Turn);
        assert_eq!(game.pot(), 4);
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(!game.must_deal()); //
        assert!(!game.is_everyone_alright()); //
        assert!(!game.is_everyone_calling()); //
        assert!(!game.is_everyone_touched()); //
        assert!(game.is_everyone_matched());
        // SmallB Turn
        let game = game.apply(Action::Check);
        assert_eq!(game.board().street(), Street::Turn);
        assert_eq!(game.pot(), 4);
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(!game.must_deal());
        assert!(!game.is_everyone_alright());
        assert!(!game.is_everyone_calling());
        assert!(!game.is_everyone_touched());
        assert!(game.is_everyone_matched());
        // Dealer Turn
        let game = game.apply(Action::Raise(4));
        assert_eq!(game.board().street(), Street::Turn);
        assert_eq!(game.pot(), 8);
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(!game.must_deal());
        assert!(!game.is_everyone_alright());
        assert!(!game.is_everyone_calling());
        assert!(game.is_everyone_touched()); //
        assert!(!game.is_everyone_matched()); //
        // SmallB Turn
        let game = game.apply(Action::Call(4));
        assert_eq!(game.board().street(), Street::Turn);
        assert_eq!(game.pot(), 12); //
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(game.must_deal()); //
        assert!(game.is_everyone_alright()); //
        assert!(game.is_everyone_calling()); //
        assert!(game.is_everyone_touched());
        assert!(game.is_everyone_matched());
        // River
        let rive = game.deck().deal(game.board().street());
        let game = game.apply(Action::Draw(rive));
        assert_eq!(game.board().street(), Street::Rive); //
        assert_eq!(game.pot(), 12);
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(!game.must_deal()); //
        assert!(!game.is_everyone_alright()); //
        assert!(!game.is_everyone_calling()); //
        assert!(!game.is_everyone_touched()); //
        assert!(game.is_everyone_matched()); //
        // SmallB River
        let game = game.apply(Action::Check);
        assert_eq!(game.board().street(), Street::Rive);
        assert_eq!(game.pot(), 12);
        assert!(!game.must_post());
        assert!(!game.must_stop());
        assert!(!game.must_deal());
        assert!(!game.is_everyone_alright());
        assert!(!game.is_everyone_calling());
        assert!(!game.is_everyone_touched());
        assert!(game.is_everyone_matched());
        // Dealer River
        let game = game.apply(Action::Check);
        assert_eq!(game.board().street(), Street::Rive);
        assert_eq!(game.pot(), 12);
        assert!(!game.must_post());
        assert!(game.must_stop()); //
        assert!(!game.must_deal());
        assert!(game.is_everyone_alright()); //
        assert!(game.is_everyone_calling()); //
        assert!(game.is_everyone_touched()); //
        assert!(game.is_everyone_matched()); //
    }

    /// next() resets game state correctly after terminal
    #[test]
    fn next_after_fold() {
        let game = Game::root().apply(Action::Fold);
        assert!(game.must_stop());
        let next = game.continuation().expect("can continue");
        assert_eq!(next.street(), Street::Pref);
        assert_eq!(next.pot(), Game::sblind() + Game::bblind());
        assert_eq!(next.board(), Board::empty());
        assert_eq!(next.dealer, 1); // rotated from 0
        assert_eq!(next.turn(), Turn::Choice(1)); // new dealer acts first
        assert!(!next.is_everyone_touched());
    }

    /// dealer rotates correctly across multiple hands
    #[test]
    fn dealer_rotation() {
        let game = Game::root();
        assert_eq!(game.dealer, 0);
        let game = game.apply(Action::Fold).continuation().unwrap();
        assert_eq!(game.dealer, 1);
        let game = game.apply(Action::Fold).continuation().unwrap();
        assert_eq!(game.dealer, 0); // wraps around
        let game = game.apply(Action::Fold).continuation().unwrap();
        assert_eq!(game.dealer, 1);
    }

    /// ticker resets correctly for each new hand, regardless of dealer
    #[test]
    fn ticker_reset_on_next() {
        let g0 = Game::root();
        let g1 = g0.apply(Action::Fold).continuation().unwrap();
        let g2 = g1.apply(Action::Fold).continuation().unwrap();
        // both should have same ticker after blinds, despite different dealers
        assert_eq!(g0.ticker, g1.ticker);
        assert_eq!(g1.ticker, g2.ticker);
        assert_eq!(g0.ticker, 2); // 2 blinds posted
    }

    /// is_everyone_touched works correctly for dealer=1
    #[test]
    fn touched_with_rotated_dealer() {
        let game = Game::root().apply(Action::Fold).continuation().unwrap();
        assert_eq!(game.dealer, 1);
        assert!(!game.is_everyone_touched()); // just blinds
        let game = game.apply(Action::Call(1));
        assert!(!game.is_everyone_touched()); // P1 called, P0 hasn't acted
        let game = game.apply(Action::Check);
        assert!(game.is_everyone_touched()); // both acted
        assert!(game.must_deal());
    }

    /// multi-street hand with rotated dealer
    #[test]
    fn full_hand_rotated_dealer() {
        let game = Game::root().apply(Action::Fold).continuation().unwrap();
        assert_eq!(game.dealer, 1);
        // preflop: P1 (dealer) calls, P0 checks
        let game = game.apply(Action::Call(1)).apply(Action::Check);
        assert!(game.must_deal());
        // flop
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Draw(flop));
        assert_eq!(game.street(), Street::Flop);
        assert_eq!(game.turn(), Turn::Choice(0)); // non-dealer first postflop
        assert!(!game.is_everyone_touched());
        // P0 checks, P1 checks
        let game = game.apply(Action::Check).apply(Action::Check);
        assert!(game.is_everyone_touched());
        assert!(game.must_deal());
    }

    /// five consecutive hands, verifying state after each
    #[test]
    fn five_hands_sequence() {
        let mut game = Game::root();
        for i in 0..5 {
            assert_eq!(game.dealer, i % 2);
            assert_eq!(game.pot(), Game::sblind() + Game::bblind());
            assert_eq!(game.street(), Street::Pref);
            assert!(!game.is_everyone_touched());
            assert_eq!(game.turn(), Turn::Choice(game.dealer));
            game = game.apply(Action::Fold).continuation().unwrap();
        }
    }

    /// call-check sequence works identically for both dealer positions
    #[test]
    fn symmetric_preflop_action() {
        // dealer=0: P0 calls, P1 checks
        let g0 = Game::root();
        assert_eq!(g0.dealer, 0);
        let g0 = g0.apply(Action::Call(1));
        assert!(!g0.is_everyone_touched());
        let g0 = g0.apply(Action::Check);
        assert!(g0.is_everyone_touched());
        assert!(g0.must_deal());
        // dealer=1: P1 calls, P0 checks
        let g1 = Game::root().apply(Action::Fold).continuation().unwrap();
        assert_eq!(g1.dealer, 1);
        let g1 = g1.apply(Action::Call(1));
        assert!(!g1.is_everyone_touched());
        let g1 = g1.apply(Action::Check);
        assert!(g1.is_everyone_touched());
        assert!(g1.must_deal());
    }

    /// actor position is correct for both dealers on flop
    #[test]
    fn flop_actor_both_dealers() {
        // dealer=0: non-dealer (P1) acts first on flop
        let g0 = Game::root().apply(Action::Call(1)).apply(Action::Check);
        let flop = g0.deck().deal(Street::Pref);
        let g0 = g0.apply(Action::Draw(flop));
        assert_eq!(g0.turn(), Turn::Choice(1)); // P1 (non-dealer) first
        // dealer=1: non-dealer (P0) acts first on flop
        let g1 = Game::root()
            .apply(Action::Fold)
            .continuation()
            .unwrap()
            .apply(Action::Call(1))
            .apply(Action::Check);
        let flop = g1.deck().deal(Street::Pref);
        let g1 = g1.apply(Action::Draw(flop));
        assert_eq!(g1.turn(), Turn::Choice(0)); // P0 (non-dealer) first
    }

    /// shove and call leads to showdown
    #[test]
    fn allin_showdown() {
        let game = Game::root();
        let shove = game.to_shove(); // dealer's stack = 99
        let game = game.apply(Action::Shove(shove));
        // BB's to_call (98) == to_shove (98), so must use Shove not Call
        let shove = game.to_shove();
        let game = game.apply(Action::Shove(shove));
        assert!(game.is_everyone_shoving());
        assert!(game.must_stop() || game.must_deal());
    }

    /// shove and fold is terminal
    #[test]
    fn allin_fold() {
        let game = Game::root();
        let shove = game.to_shove();
        let game = game.apply(Action::Shove(shove)).apply(Action::Fold);
        assert!(game.must_stop());
        assert!(game.is_everyone_folding());
    }

    /// raise-reraise sequence keeps action open
    #[test]
    fn raise_reraise() {
        let g0 = Game::root();
        let r1 = g0.to_raise();
        let g1 = g0.apply(Action::Raise(r1));
        let r2 = g1.to_raise();
        let g2 = g1.apply(Action::Raise(r2));
        assert!(!g2.must_deal()); // betting not closed
        assert!(!g2.is_everyone_alright()); // stakes unmatched
        assert_eq!(g2.turn(), Turn::Choice(0)); // back to dealer
        assert!(g2.may_raise() || g2.may_call()); // can continue
    }

    /// stacks update correctly after fold (before new blinds)
    #[test]
    fn stacks_after_fold() {
        let game = Game::root().apply(Action::Fold);
        assert!(game.must_stop());
        // check settlements before next hand
        let settlements = game.settlements();
        // reward() is total received, won() is net (reward - risked)
        assert_eq!(settlements[0].pnl().reward(), 0); // dealer folded
        assert_eq!(settlements[1].pnl().reward(), 3); // BB wins pot
        assert_eq!(settlements[0].won(), -1); // lost SB
        assert_eq!(settlements[1].won(), 1); // net gain
    }

    /// stacks update correctly after flop fold
    #[test]
    fn stacks_after_flop_bet_fold() {
        let game = Game::root().apply(Action::Call(1)).apply(Action::Check);
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Draw(flop));
        // P1 (non-dealer) acts first, raises
        let raise = game.to_raise();
        let game = game.apply(Action::Raise(raise));
        // P0 folds
        let game = game.apply(Action::Fold);
        assert!(game.must_stop());
        let settlements = game.settlements();
        // pot is 4 + raise, P1 wins it all
        assert_eq!(settlements[0].pnl().reward(), 0); // dealer folded
        assert!(settlements[1].pnl().reward() > 0); // BB wins pot
        assert_eq!(settlements[0].won(), -2); // lost 2
    }

    /// multi-hand with betting, not just folds
    #[test]
    fn multi_hand_with_betting() {
        let g0 = Game::root();
        // hand 1: call-check, bet-fold on flop
        let g0 = g0.apply(Action::Call(1)).apply(Action::Check);
        let flop = g0.deck().deal(Street::Pref);
        let g0 = g0.apply(Action::Draw(flop));
        let raise = g0.to_raise();
        let g0 = g0.apply(Action::Raise(raise)).apply(Action::Fold);
        let g1 = g0.continuation().unwrap();
        assert_eq!(g1.dealer, 1);
        // hand 2: raise-call, bet-fold on flop
        let r1 = g1.to_raise();
        let g1 = g1.apply(Action::Raise(r1));
        let c1 = g1.to_call();
        let g1 = g1.apply(Action::Call(c1));
        let flop = g1.deck().deal(Street::Pref);
        let g1 = g1.apply(Action::Draw(flop));
        let raise = g1.to_raise();
        let g1 = g1.apply(Action::Raise(raise)).apply(Action::Fold);
        let g2 = g1.continuation().unwrap();
        assert_eq!(g2.dealer, 0);
        assert_eq!(g2.pot(), 3);
    }

    /// legal() returns correct options preflop after blinds
    #[test]
    fn legal_preflop_options() {
        let game = Game::root();
        let legal = game.legal();
        assert!(legal.contains(&Action::Fold));
        assert!(legal.contains(&Action::Call(1)));
        assert!(legal.iter().any(|a| matches!(a, Action::Raise(_))));
        assert!(legal.iter().any(|a| matches!(a, Action::Shove(_))));
        assert!(!legal.contains(&Action::Check)); // can't check facing BB
    }

    /// legal() after limp allows check
    #[test]
    fn legal_bb_can_check() {
        let game = Game::root().apply(Action::Call(1));
        let legal = game.legal();
        assert!(legal.contains(&Action::Check));
        assert!(!legal.contains(&Action::Fold)); // no need to fold
    }

    /// legal() on flop
    #[test]
    fn legal_flop_options() {
        let game = Game::root().apply(Action::Call(1)).apply(Action::Check);
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Draw(flop));
        let legal = game.legal();
        assert!(legal.contains(&Action::Check));
        assert!(legal.iter().any(|a| matches!(a, Action::Raise(_))));
        assert!(!legal.contains(&Action::Fold)); // no bet to fold to
    }

    /// terminal via river showdown
    #[test]
    fn terminal_river_showdown() {
        let mut game = Game::root().apply(Action::Call(1)).apply(Action::Check);
        for street in [Street::Pref, Street::Flop, Street::Turn] {
            let cards = game.deck().deal(street);
            game = game
                .apply(Action::Draw(cards))
                .apply(Action::Check)
                .apply(Action::Check);
        }
        assert_eq!(game.street(), Street::Rive);
        assert!(game.must_stop());
        assert!(!game.must_deal());
    }

    /// ten consecutive hands alternate dealers correctly
    #[test]
    fn ten_hands_alternation() {
        let mut game = Game::root();
        for i in 0..10 {
            assert_eq!(game.dealer, i % 2);
            assert_eq!(game.turn(), Turn::Choice(game.dealer));
            game = game.apply(Action::Fold).continuation().unwrap();
        }
    }

    /// min raise calculation
    #[test]
    fn min_raise_size() {
        let game = Game::root();
        // dealer stake=1, BB stake=2. to_raise = (2-1) + max(2-1, BB) = 1 + 2 = 3
        assert_eq!(game.to_raise(), 3);
        let game = game.apply(Action::Raise(3));
        // dealer stake=4, BB stake=2. to_raise = (4-2) + max(4-2, BB) = 2 + 2 = 4
        assert_eq!(game.to_raise(), 4);
    }

    /// pot size tracks correctly through streets
    #[test]
    fn pot_tracking() {
        let game = Game::root();
        assert_eq!(game.pot(), 3);
        let game = game.apply(Action::Call(1));
        assert_eq!(game.pot(), 4);
        let game = game.apply(Action::Raise(4));
        assert_eq!(game.pot(), 8);
        let game = game.apply(Action::Call(4));
        assert_eq!(game.pot(), 12);
    }

    /// cannot continue if player busts
    #[test]
    fn bust_prevents_next() {
        // create game where one player will bust
        let game = Game::root();
        let shove = game.to_shove();
        let game = game.apply(Action::Shove(shove));
        // BB must shove (not call) since to_call == to_shove
        let shove = game.to_shove();
        let game = game.apply(Action::Shove(shove));
        // run to showdown
        let mut game = game;
        while !game.must_stop() {
            if game.must_deal() {
                let cards = game.deck().deal(game.street());
                game = game.apply(Action::Draw(cards));
            }
        }
        // total pot is 2*STACK (STACK from each)
        // either winner gets it all or split pot
        let rewards: Vec<_> = game.settlements().iter().map(|s| s.pnl().reward()).collect();
        let total: Chips = rewards.iter().sum();
        assert_eq!(total, 2 * STACK);
        assert!((rewards.contains(&0) && rewards.contains(&(2 * STACK))) || (rewards.iter().all(|&r| r == STACK)));
    }

    /// actor_idx wraps correctly with ticker
    #[test]
    fn actor_idx_wrapping() {
        let game = Game::root();
        assert_eq!(game.actor_idx(), 0); // dealer, ticker=2, (0+2)%2=0
        let game = game.apply(Action::Call(1));
        assert_eq!(game.actor_idx(), 1); // ticker=3, (0+3)%2=1
        let game = game.apply(Action::Check);
        // must_deal is true, but if we peek at actor_idx...
        assert_eq!((game.dealer + game.ticker) % game.n(), 0); // wraps
    }

    /// snap preserves legal actions unchanged
    /// TODO: expand beyond only testing at the root node. apply some pot actions
    #[test]
    fn snap_legal_unchanged() {
        let game = Game::root();
        game.legal()
            .iter()
            .inspect(|&&action| assert_eq!(game.snap(action), action))
            .count();
    }

    /// snap coerces oversized raise to shove
    #[test]
    fn snap_raise_to_shove_too_large() {
        let game = Game::root();
        let shove = game.to_shove();
        assert_eq!(game.snap(Action::Raise(Chips::MAX)), game.shove());
        assert_eq!(game.snap(Action::Raise(shove)), game.shove());
    }

    /// snap coerces undersized raise to min-raise
    #[test]
    fn snap_raise_to_minim_too_small() {
        let game = Game::root();
        let minraise = game.to_raise();
        assert_eq!(game.snap(Action::Raise(1)), Action::Raise(minraise));
        assert_eq!(game.snap(Action::Raise(0)), Action::Raise(minraise));
    }

    /// snap coerces fold to check when not facing bet
    #[test]
    fn snap_fold_to_check_not_facing_bet() {
        let game = Game::root().apply(Action::Call(1));
        assert!(!game.may_fold());
        assert!(game.may_check());
        assert_eq!(game.snap(Action::Fold), Action::Check);
    }

    /// snap coerces check to call when facing bet
    #[test]
    fn snap_check_to_call_facing_bet() {
        let game = Game::root();
        assert!(!game.may_check());
        assert!(game.may_call());
        assert_eq!(game.snap(Action::Check), game.calls());
    }

    // ─── Multiplayer (P > 2) ───────────────────────────────────────────

    type Game3 = GameN<3>;
    type Game6 = FunTable;

    /// 3-player root: SB and BB posted, dealer acts first preflop
    #[test]
    fn three_player_root() {
        let game = Game3::root();
        assert_eq!(game.pot(), Game3::sblind() + Game3::bblind());
        assert_eq!(game.street(), Street::Pref);
        assert_eq!(game.n(), 3);
        assert_eq!(game.turn(), Turn::Choice(game.dealer));
    }

    /// 6-player root: SB and BB posted, UTG (dealer+3) acts first preflop
    #[test]
    fn six_player_root() {
        let game = Game6::root();
        assert_eq!(game.pot(), Game6::sblind() + Game6::bblind());
        assert_eq!(game.street(), Street::Pref);
        assert_eq!(game.n(), 6);
        assert_eq!(game.turn(), Turn::Choice((game.dealer + 3) % 6));
    }

    /// 3-player: two folds reaches terminal
    #[test]
    fn three_player_fold_to_terminal() {
        let game = Game3::root();
        let game = game.apply(Action::Fold);
        assert!(!game.must_stop());
        let game = game.apply(Action::Fold);
        assert!(game.must_stop());
        assert!(game.is_everyone_folding());
    }

    /// 6-player: five folds reaches terminal
    #[test]
    fn six_player_fold_to_terminal() {
        let mut game = Game6::root();
        for i in 0..5 {
            assert!(!game.must_stop(), "terminal too early at fold {i}");
            game = game.apply(Action::Fold);
        }
        assert!(game.must_stop());
        assert!(game.is_everyone_folding());
    }

    /// 3-player: full orbit of calls reaches flop
    #[test]
    fn three_player_call_around() {
        let game = Game3::root();
        // dealer calls BB (needs 2 to match, has 0 stake)
        let game = game.apply(Action::Call(game.to_call()));
        assert!(!game.is_everyone_touched());
        // SB completes (needs 1 more to match BB=2)
        let game = game.apply(Action::Call(game.to_call()));
        assert!(!game.is_everyone_touched());
        // BB checks (option)
        let game = game.apply(Action::Check);
        assert!(game.is_everyone_touched());
        assert!(game.is_everyone_matched());
        assert!(game.must_deal());
        assert_eq!(game.pot(), 6); // 3 * 2
    }

    /// 6-player: full orbit of calls reaches flop
    #[test]
    fn six_player_call_around() {
        let mut game = Game6::root();
        // 4 players call (UTG through BTN)
        for _ in 0..4 {
            game = game.apply(Action::Call(game.to_call()));
        }
        // SB completes
        game = game.apply(Action::Call(game.to_call()));
        assert!(!game.is_everyone_touched());
        // BB checks
        game = game.apply(Action::Check);
        assert!(game.is_everyone_touched());
        assert!(game.must_deal());
        assert_eq!(game.pot(), 12); // 6 * 2
    }

    /// 3-player: postflop action starts left of dealer
    #[test]
    fn three_player_postflop_order() {
        let mut game = Game3::root();
        game = game.apply(Action::Call(game.to_call()));
        game = game.apply(Action::Call(game.to_call()));
        game = game.apply(Action::Check);
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Draw(flop));
        assert_eq!(game.street(), Street::Flop);
        // first to act postflop is SB (dealer+1)
        assert_eq!(game.turn(), Turn::Choice((game.dealer + 1) % 3));
    }

    /// 3-player: postflop skips folded player
    #[test]
    fn three_player_postflop_skip_folded() {
        let mut game = Game3::root();
        // dealer folds, SB completes, BB checks
        game = game.apply(Action::Fold);
        game = game.apply(Action::Call(game.to_call()));
        game = game.apply(Action::Check);
        assert!(game.must_deal());
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Draw(flop));
        assert_eq!(game.street(), Street::Flop);
        // SB acts first (dealer+1), dealer is folded but SB is still active
        let actor = game.turn().position();
        assert_ne!(actor, game.dealer); // not the folded dealer
        assert_eq!(game.seats()[actor].state(), State::Betting);
    }

    /// 3-player: button rotates through all three seats
    #[test]
    fn three_player_dealer_rotation() {
        let game = Game3::root();
        assert_eq!(game.dealer, 0);
        let game = game.apply(Action::Fold).apply(Action::Fold);
        let game = game.continuation().unwrap();
        assert_eq!(game.dealer, 1);
        let game = game.apply(Action::Fold).apply(Action::Fold);
        let game = game.continuation().unwrap();
        assert_eq!(game.dealer, 2);
        let game = game.apply(Action::Fold).apply(Action::Fold);
        let game = game.continuation().unwrap();
        assert_eq!(game.dealer, 0); // wraps around
    }

    /// 6-player: button rotates through all six seats
    #[test]
    fn six_player_dealer_rotation() {
        let mut game = Game6::root();
        for i in 0..6 {
            assert_eq!(game.dealer, i % 6);
            for _ in 0..5 {
                game = game.apply(Action::Fold);
            }
            game = game.continuation().unwrap();
        }
        assert_eq!(game.dealer, 0);
    }

    /// 3-player: raise-call-fold dynamics
    #[test]
    fn three_player_raise_fold() {
        let game = Game3::root();
        let raise = game.to_raise();
        let game = game.apply(Action::Raise(raise));
        // SB folds
        let game = game.apply(Action::Fold);
        assert!(!game.must_stop());
        // BB calls — now all three have acted: dealer raised, SB folded, BB called
        let game = game.apply(Action::Call(game.to_call()));
        assert!(game.is_everyone_touched());
        assert!(game.must_deal());
    }

    /// 3-player: full hand through river showdown
    #[test]
    fn three_player_full_hand() {
        let mut game = Game3::root();
        // preflop: all call
        game = game.apply(Action::Call(game.to_call()));
        game = game.apply(Action::Call(game.to_call()));
        game = game.apply(Action::Check);
        // flop through river: check around
        for street in [Street::Pref, Street::Flop, Street::Turn] {
            assert!(game.must_deal());
            let cards = game.deck().deal(street);
            game = game.apply(Action::Draw(cards));
            game = game.apply(Action::Check).apply(Action::Check).apply(Action::Check);
        }
        assert_eq!(game.street(), Street::Rive);
        assert!(game.must_stop());
    }

    /// 3-player: all-in showdown
    #[test]
    fn three_player_allin() {
        let game = Game3::root();
        let shove = game.to_shove();
        let game = game.apply(Action::Shove(shove));
        let shove = game.to_shove();
        let game = game.apply(Action::Shove(shove));
        let shove = game.to_shove();
        let game = game.apply(Action::Shove(shove));
        assert!(game.is_everyone_shoving());
        assert!(game.must_stop() || game.must_deal());
    }

    /// 3-player: settlements sum to total chips
    #[test]
    fn three_player_settlements_conserve_chips() {
        let game = Game3::root();
        let game = game.apply(Action::Fold).apply(Action::Fold);
        assert!(game.must_stop());
        let total: Chips = game.settlements().iter().map(|s| s.pnl().reward()).sum();
        assert_eq!(total, game.pot());
    }

    /// 3-player: stacks + stakes + pot = total chips invariant
    #[test]
    fn three_player_chip_conservation() {
        let mut game = Game3::root();
        let initial = game.total();
        game = game.apply(Action::Call(game.to_call()));
        assert_eq!(game.total(), initial);
        game = game.apply(Action::Call(game.to_call()));
        assert_eq!(game.total(), initial);
        let raise = game.to_raise();
        game = game.apply(Action::Raise(raise));
        assert_eq!(game.total(), initial);
    }

    /// 3-player: legal actions are nonempty at choice nodes
    #[test]
    fn three_player_legal_nonempty() {
        let game = Game3::root();
        assert!(!game.legal().is_empty());
        let game = game.apply(Action::Fold);
        assert!(!game.legal().is_empty());
    }

    /// 6-player: multiple hands with mixed actions
    #[test]
    fn six_player_multi_hand() {
        let mut game = Game6::root();
        for hand in 0..3 {
            assert_eq!(game.dealer, hand % 6);
            assert_eq!(game.street(), Street::Pref);
            // everyone folds to BB
            for _ in 0..5 {
                game = game.apply(Action::Fold);
            }
            assert!(game.must_stop());
            game = game.continuation().unwrap();
        }
    }

    /// 3-player: first to act preflop is always dealer
    #[test]
    fn three_player_preflop_actor_all_dealers() {
        let mut game = Game3::root();
        for expected_dealer in 0..3 {
            assert_eq!(game.dealer, expected_dealer);
            assert_eq!(game.turn(), Turn::Choice(expected_dealer));
            game = game.apply(Action::Fold).apply(Action::Fold);
            game = game.continuation().unwrap();
        }
    }

    /// `Game::translate` under `SNAP` is behaviorally equivalent to `Game::edgify`
    /// for raise actions — both implement classical-nearest mapping.
    #[test]
    fn translate_snap_matches_edgify_on_raise() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let game = Game::root();
        let ref mut rng = SmallRng::seed_from_u64(0);
        for chips in [4, 6, 8, 10, 16] {
            let edgify = game.edgify(Action::Raise(chips), 0);
            let translate = game.translate(Action::Raise(chips), 0, &Translation::Snap, rng);
            assert_eq!(translate, Translated::Snap(edgify));
        }
    }

    /// Non-raise actions always resolve OnTree, regardless of policy.
    #[test]
    fn translate_passes_through_non_raise() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let game = Game::root();
        let ref mut rng = SmallRng::seed_from_u64(0);
        assert_eq!(game.translate(Action::Fold, 0, &Translation::Snap, rng), Translated::Snap(Edge::Fold),);
        assert_eq!(game.translate(Action::Check, 0, &Translation::Phargmax, rng), Translated::Snap(Edge::Check),);
        assert_eq!(game.translate(Action::Call(1), 0, &Translation::Harmonic, rng), Translated::Snap(Edge::Call),);
    }

    /// `Phargmax` on canonical raises produces the same edge as `Snap`
    /// (single bracket = single anchor, no harmonic computation).
    #[test]
    fn translate_phargmax_canonical_matches_snap() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let game = Game::root();
        let ref mut rng = SmallRng::seed_from_u64(0);
        for chips in [4, 6, 8, 10] {
            let snap = game.translate(Action::Raise(chips), 0, &Translation::Snap, rng);
            let phargmax = game.translate(Action::Raise(chips), 0, &Translation::Phargmax, rng);
            assert_eq!(snap, phargmax, "canonical Raise({chips}): Phargmax must match Snap");
        }
    }

    /// `Phargmax` on an off-tree raise returns one of the two bracketing
    /// canonical edges (deterministic argmax of harmonic distribution).
    /// Preflop depth 0: OPENS = [2, 3, 4, 5] BBs = [4, 6, 8, 10] chips.
    /// Raise(7) = 3.5 BB, brackets are BBs(3) (= Open(3)) and BBs(4) (= Open(4)).
    #[test]
    fn translate_phargmax_off_tree_returns_bracketing_edge() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let game = Game::root();
        let ref mut rng = SmallRng::seed_from_u64(0);
        let result = game.translate(Action::Raise(7), 0, &Translation::Phargmax, rng);
        let lo = Translated::Snap(Edge::Open(3));
        let hi = Translated::Snap(Edge::Open(4));
        assert!(result == lo || result == hi, "Phargmax(Raise(7)) = {result:?} must be one of {{{lo:?}, {hi:?}}}");
    }

    /// `Harmonic` on an off-tree raise: 100 trials, every result must be
    /// one of the two bracketing canonical edges. Low-entropy assertion
    /// (set membership, not exact value).
    #[test]
    fn translate_harmonic_off_tree_always_in_bracket_set() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let game = Game::root();
        let ref mut rng = SmallRng::seed_from_u64(0xCAFEF00D);
        let lo = Translated::Snap(Edge::Open(3));
        let hi = Translated::Snap(Edge::Open(4));
        for trial in 0..100 {
            let result = game.translate(Action::Raise(7), 0, &Translation::Harmonic, rng);
            assert!(
                result == lo || result == hi,
                "trial {trial}: Harmonic(Raise(7)) = {result:?} must be one of {{{lo:?}, {hi:?}}}",
            );
        }
    }

    /// `Exact` snaps a raise that lands on (or within the relative
    /// `EXACT_SNAP_TOLERANCE` of) a grid anchor and leaves a genuinely
    /// off-grid raise `Free`. Preflop depth 0: OPENS = [2,3,4,5] BBs =
    /// [4,6,8,10] chips. Raise(6) = 3 BB = Open(3) (exact); Raise(7) = 3.5 BB
    /// is 16.7% off the nearest anchor, well beyond tolerance → off-grid.
    #[test]
    fn translate_exact_snaps_on_anchor_else_free() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let game = Game::root();
        let ref mut rng = SmallRng::seed_from_u64(0);
        assert_eq!(game.translate(Action::Raise(6), 0, &Translation::Exact, rng), Translated::Snap(Edge::Open(3)));
        assert_eq!(game.translate(Action::Raise(7), 0, &Translation::Exact, rng), Translated::Free(Action::Raise(7)));
    }

    /// The shove guard: an off-grid raise at or above the all-in threshold
    /// snaps to `Edge::Shove` rather than escaping as `Free`, keeping `Free`
    /// reserved for genuinely off-grid, non-shove raises.
    #[test]
    fn translate_exact_off_grid_at_shove_snaps_to_shove() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let game = Game::root();
        let ref mut rng = SmallRng::seed_from_u64(0);
        let shove = game.to_shove();
        assert_eq!(game.translate(Action::Raise(shove), 0, &Translation::Exact, rng), Translated::Snap(Edge::Shove));
        assert_eq!(
            game.translate(Action::Raise(shove + 50), 0, &Translation::Exact, rng),
            Translated::Snap(Edge::Shove)
        );
    }

    /// Non-raise actions resolve to the same canonical Edge under every
    /// translation. Exhaustive over the three variants.
    #[test]
    fn translate_non_raise_actions_invariant_across_translations() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let game = Game::root();
        let ref mut rng = SmallRng::seed_from_u64(0);
        let translations = [Translation::Snap, Translation::Harmonic, Translation::Phargmax];
        let cases = [
            (Action::Fold, Edge::Fold),
            (Action::Check, Edge::Check),
            (Action::Call(1), Edge::Call),
        ];
        for lang in &translations {
            for (action, expected) in &cases {
                assert_eq!(
                    game.translate(*action, 0, lang, rng),
                    Translated::Snap(*expected),
                    "translation {lang:?} on {action:?} must produce OnTree({expected:?})",
                );
            }
        }
    }

    /// `Snap` on the boundary cases — chips below the smallest open and
    /// above the largest — clamps to the extreme.
    #[test]
    fn translate_snap_clamps_outside_grid() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let game = Game::root();
        let ref mut rng = SmallRng::seed_from_u64(0);
        // Below smallest: Raise(2) = 1 BB, smallest is BBs(2) = Open(2).
        assert_eq!(game.translate(Action::Raise(2), 0, &Translation::Snap, rng), Translated::Snap(Edge::Open(2)),);
        // Above largest: Raise(20) = 10 BB, largest is BBs(5) = Open(5).
        assert_eq!(game.translate(Action::Raise(20), 0, &Translation::Snap, rng), Translated::Snap(Edge::Open(5)),);
    }

    /// 6-player: BB raises their option after five limps; all five call;
    /// the street closes at the aggressor without BB acting again.
    #[test]
    fn six_player_bb_option_raise() {
        let mut game = Game6::root();
        for _ in 0..5 {
            game = game.apply(Action::Call(game.to_call()));
        }
        assert!(!game.is_everyone_touched(), "BB option must still be open");
        let bb = game.turn().position();
        game = game.apply(Action::Raise(game.to_raise()));
        assert_eq!(game.street(), Street::Pref);
        for _ in 0..5 {
            assert_ne!(game.turn().position(), bb, "BB must not act again before closure");
            game = game.apply(Action::Call(game.to_call()));
        }
        assert!(game.is_everyone_touched());
        assert!(game.is_everyone_matched());
        assert!(game.must_deal(), "all calls of the BB raise must close preflop");
    }

    /// 6-player: a 3-bet war closes exactly when the last aggressor is called.
    #[test]
    fn six_player_closure_at_aggressor() {
        let mut game = Game6::root();
        game = game.apply(Action::Raise(game.to_raise())); // UTG opens
        game = game.apply(Action::Fold); // HJ
        game = game.apply(Action::Fold); // CO
        game = game.apply(Action::Raise(game.to_raise())); // BTN 3-bets
        game = game.apply(Action::Fold); // SB
        game = game.apply(Action::Fold); // BB
        assert!(!game.must_deal(), "UTG still owes a decision on the 3-bet");
        game = game.apply(Action::Call(game.to_call())); // UTG calls
        assert!(game.must_deal(), "calling the last aggressor must close the street");
    }

    /// 6-player: only CO and BTN see the flop (raise folds out the blinds);
    /// check-check closes the street with each acting exactly once despite
    /// four folded seats between them.
    #[test]
    fn six_player_postflop_two_live_late_seats() {
        let mut game = Game6::root();
        game = game.apply(Action::Fold); // UTG
        game = game.apply(Action::Fold); // HJ
        game = game.apply(Action::Raise(game.to_raise())); // CO opens
        game = game.apply(Action::Call(game.to_call())); // BTN calls
        game = game.apply(Action::Fold); // SB
        game = game.apply(Action::Fold); // BB
        let flop = game.deck().deal(Street::Pref);
        let mut game = game.apply(Action::Draw(flop));
        assert_eq!(game.street(), Street::Flop);
        let first = game.turn().position();
        game = game.apply(Action::Check);
        let second = game.turn().position();
        assert_ne!(first, second, "two live players must alternate");
        game = game.apply(Action::Check);
        assert!(game.must_deal(), "check-check between the two live seats must close the flop");
    }

    /// 6-player: flop bet / raise / call between live players closes the
    /// street, and the multiway min-raise obeys the increment rule.
    #[test]
    fn six_player_min_raise_increment() {
        let mut game = Game6::root();
        for _ in 0..5 {
            game = game.apply(Action::Call(game.to_call()));
        }
        game = game.apply(Action::Check); // BB option
        let flop = game.deck().deal(Street::Pref);
        let mut game = game.apply(Action::Draw(flop));
        game = game.apply(Action::Raise(10)); // SB bets 10
        assert_eq!(game.to_raise(), 20, "min-raise must call 10 and raise by the 10 increment");
        game = game.apply(Action::Raise(game.to_raise())); // BB raises to 20
        assert_eq!(game.to_raise(), 30, "increment is 10, so next min-raise is to 30");
    }

    /// 6-player: shove and a single covering caller runs out to showdown;
    /// the lone live player checks down and chips are conserved throughout.
    /// (BB gets a covering stack — with equal stacks the exact-all-in call
    /// would have to be a Shove; see `six_player_exact_call_is_shove`.)
    #[test]
    fn six_player_allin_single_caller_runout() {
        let mut game = Game6::preblind(0, [200, 200, 400, 200, 200, 200]);
        game.act(game.posts());
        game.act(game.posts());
        let total = game.total();
        game = game.apply(Action::Shove(game.to_shove())); // UTG jams
        for _ in 0..4 {
            game = game.apply(Action::Fold); // HJ, CO, BTN, SB
        }
        game = game.apply(Action::Call(game.to_call())); // BB calls, covering
        let mut steps = 0;
        while game.turn() != Turn::Terminal {
            assert_eq!(game.total(), total, "chips must be conserved during runout");
            match game.turn() {
                Turn::Chance => game = game.apply(game.reveal()),
                Turn::Choice(_) => game = game.apply(Action::Check),
                Turn::Terminal => unreachable!(),
            }
            steps += 1;
            assert!(steps < 32, "runout must terminate");
        }
        assert_eq!(game.street(), Street::Rive, "all-in call must run out all streets");
        let rewards = game.settlements().iter().map(|s| s.pnl().reward()).sum::<Chips>();
        assert_eq!(rewards, game.pot(), "settlements must redistribute exactly the pot");
    }

    /// 3-player: three different stacks all in preflop creates side pots;
    /// settlement rewards redistribute exactly the pot.
    #[test]
    fn three_player_side_pot_conservation() {
        let mut game = Game3::preblind(0, [50, 100, 200]);
        game.act(game.posts());
        game.act(game.posts());
        let total = game.total();
        game = game.apply(Action::Shove(game.to_shove()));
        game = game.apply(Action::Shove(game.to_shove()));
        game = game.apply(Action::Shove(game.to_shove()));
        let mut steps = 0;
        while game.turn() != Turn::Terminal {
            assert_eq!(game.total(), total);
            game = game.apply(game.reveal());
            steps += 1;
            assert!(steps < 8, "all-shoved runout must terminate");
        }
        let rewards = game.settlements().iter().map(|s| s.pnl().reward()).sum::<Chips>();
        assert_eq!(rewards, game.pot(), "side pots must redistribute exactly the pot");
    }

    /// 6-player: everyone folds to the big blind, who wins the blinds
    /// without acting (walking BB).
    #[test]
    fn six_player_walking_bb() {
        let mut game = Game6::root();
        for _ in 0..5 {
            game = game.apply(Action::Fold);
        }
        assert_eq!(game.turn(), Turn::Terminal);
        let rewards = game.settlements().iter().map(|s| s.pnl().reward()).sum::<Chips>();
        assert_eq!(rewards, game.pot(), "walking BB must collect exactly the pot");
    }

    /// Multiway effective stack is the second-largest among live players,
    /// so folded stacks can no longer drag the SPR down.
    #[test]
    fn six_player_effective_second_largest() {
        let mut game = Game6::preblind(0, [10, 300, 200, 100, 50, 25]);
        game.act(game.posts());
        game.act(game.posts());
        // post-blind stacks: [10, 299, 198, 100, 50, 25] -> 2nd-largest = 198
        assert_eq!(game.effective(), 198, "second-largest of live stacks");
        let game = game.apply(Action::Fold); // UTG (seat 3, stack 100) folds
        assert_eq!(game.effective(), 198, "folding a mid stack keeps 2nd-largest among live");
        let game = game.apply(Action::Fold); // HJ (seat 4, stack 50) folds
        let game = game.apply(Action::Fold); // CO (seat 5, stack 25) folds
        let game = game.apply(Action::Fold); // BTN (seat 0, stack 10) folds
        // live: SB 299, BB 198 -> heads-up semantics = smaller of the two
        assert_eq!(game.effective(), 198, "reduces to min of the two live stacks");
    }

    /// A call that is exactly all-in is not a legal Call — the engine
    /// requires it to be expressed as a Shove. Pins the call-as-shove
    /// semantic so action encoding stays consistent multiway.
    #[test]
    fn six_player_exact_call_is_shove() {
        let mut game = Game6::root();
        game = game.apply(Action::Shove(game.to_shove())); // UTG jams 200
        for _ in 0..4 {
            game = game.apply(Action::Fold); // HJ, CO, BTN, SB
        }
        // BB has 198 behind and owes exactly 198: Call is illegal, Shove is the call.
        assert_eq!(game.to_call(), game.to_shove());
        assert!(!game.may_call(), "exact all-in call must not be a Call");
        assert!(game.may_shove());
        let game = game.apply(Action::Shove(game.to_shove()));
        assert!(game.is_everyone_shoving(), "both all-in triggers pure runout");
    }

    /// Randomized playouts: 3- and 6-player games driven by seeded random
    /// legal actions must conserve chips at every step, keep legal actions
    /// available at every decision, and terminate within a bounded number
    /// of actions with settlements that redistribute exactly the pot.
    /// Stacks are randomized down to the big blind (stressing all-in-from-
    /// blind and side-pot paths) and the button visits every seat.
    #[test]
    fn multiway_random_playout_invariants() {
        fn playout<const P: usize>(seed: u64) {
            use rand::Rng;
            use rand::SeedableRng;
            let ref mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let dealer = rng.random_range(0..P);
            let stacks = std::array::from_fn::<Chips, P, _>(|_| rng.random_range(GameN::<P>::bblind()..3 * STACK));
            let mut game = GameN::<P>::from_start(dealer, stacks);
            let total = game.total();
            let mut steps = 0;
            while game.turn() != Turn::Terminal {
                assert_eq!(game.total(), total, "P={P} seed={seed}: chips must be conserved");
                assert_eq!(
                    game.board_cards().len(),
                    game.street().n_observed() - 2,
                    "P={P} seed={seed}: board must match street"
                );
                let legal = game.legal();
                assert!(!legal.is_empty(), "P={P} seed={seed}: non-terminal state must have legal actions");
                let action = legal[rng.random_range(0..legal.len())];
                let action = match action {
                    Action::Raise(_) if game.to_raise() < game.to_shove() => {
                        Action::Raise(rng.random_range(game.to_raise()..game.to_shove()))
                    }
                    a => a,
                };
                assert!(game.is_allowed(&action), "P={P} seed={seed}: sampled action must be legal");
                game = game.apply(action);
                steps += 1;
                assert!(steps < 512, "P={P} seed={seed}: playout must terminate");
            }
            let settlements = game.settlements();
            assert_eq!(settlements.len(), P, "P={P} seed={seed}: one settlement per seat");
            let rewards = settlements.iter().map(|s| s.pnl().reward()).sum::<Chips>();
            assert_eq!(rewards, game.pot(), "P={P} seed={seed}: settlements must redistribute the pot");
            assert_eq!(game.total(), total, "P={P} seed={seed}: chips must be conserved at terminal");
        }
        for seed in 0..512 {
            playout::<3>(seed);
            playout::<6>(seed);
        }
    }

    /// Pins the current min-raise rule after a short all-in: the engine
    /// bases the next minimum raise on the *actual* largest increment (the
    /// short shove), which is more lenient than the TDA "last full raise"
    /// rule. Deliberately unchanged — the (street, depth)-keyed sizing grid
    /// snaps amounts anyway, and altering `to_raise` would churn the choices
    /// mask that keys the live heads-up blueprint. Revisit only alongside a
    /// deliberate re-key.
    #[test]
    fn three_player_short_allin_min_raise_pins_lenient_rule() {
        // dealer=0 -> seat 1 = SB (200), seat 2 = BB (13, i.e. 11 behind).
        let mut game = Game3::preblind(0, [200, 200, 13]);
        game.act(game.posts());
        game.act(game.posts());
        game = game.apply(Action::Raise(10)); // BTN opens to 10
        game = game.apply(Action::Call(game.to_call())); // SB calls 9 (stake 10)
        game = game.apply(Action::Shove(game.to_shove())); // BB short-jams 11 (stake 13)
        assert!(game.is_everyone_shoving().not() && game.turn().is_choice());
        assert_eq!(game.turn().position(), 0, "action returns to BTN");
        // BTN min-raise puts in call 3 + max(short increment 3, bb 2) = 6,
        // NOT the TDA call 3 + last full raise 8 = 11.
        assert_eq!(game.to_raise(), 6, "lenient short-all-in increment is the pinned behavior");
    }

    /// Pins reopen rights: after a short all-in, the original aggressor may
    /// raise again when action returns (TDA would deny re-raise rights).
    /// Same rationale as the min-raise pin: abstraction-tolerant, and fixing
    /// it would churn live blueprint keys.
    #[test]
    fn three_player_short_allin_reopens_action_pins_lenient_rule() {
        let mut game = Game3::preblind(0, [200, 200, 13]);
        game.act(game.posts());
        game.act(game.posts());
        game = game.apply(Action::Raise(10)); // BTN opens to 10
        game = game.apply(Action::Call(game.to_call())); // SB calls (stake 10)
        game = game.apply(Action::Shove(game.to_shove())); // BB short-jams (stake 13)
        assert_eq!(game.turn().position(), 0, "action returns to the original aggressor");
        assert!(game.may_raise(), "pinned: short all-in reopens raising for the original aggressor");
    }

    /// A single 6-max street can legally produce more abstract edges than a
    /// heads-up `Path<1>` holds (12). The width-pinned [`Subgame`] type must
    /// hold it — and if even that capacity is ever exceeded, the collect
    /// must PANIC rather than silently truncate onto a colliding info-set
    /// key (the failure mode that motivated widening `Path`).
    #[test]
    fn six_player_street_exceeds_headsup_path_capacity() {
        // A legal 6-max preflop: 4 limps + SB call + BB raise + 5 calls +
        // UTG re-raise + 2 calls = 14 abstract edges on one street.
        let street = [
            Edge::Call,
            Edge::Call,
            Edge::Call,
            Edge::Call,
            Edge::Call,
            Edge::Raise(Odds::new(1, 1)),
            Edge::Call,
            Edge::Call,
            Edge::Call,
            Edge::Call,
            Edge::Call,
            Edge::Raise(Odds::new(1, 1)),
            Edge::Call,
            Edge::Call,
        ];
        assert!(street.len() > Path::<1>::CAPACITY, "sequence exceeds heads-up capacity by construction");
        assert!(street.len() <= Path::<4>::CAPACITY, "a 6-max-width Subgame must hold a legal street");
        let path = street.into_iter().collect::<Path<4>>();
        assert_eq!(path.length(), street.len(), "no truncation at multiway width");
        assert_eq!(path.into_iter().collect::<Vec<_>>(), street.to_vec(), "lossless round-trip");
        assert!(
            std::panic::catch_unwind(|| street.into_iter().collect::<Path<1>>()).is_err(),
            "overflowing a narrow Path must panic, never truncate"
        );
    }

    /// Randomized multi-hand sessions: continuation() must rotate the button,
    /// re-post blinds, and preserve total chips across hand boundaries.
    #[test]
    fn multiway_random_session_continuation() {
        use rand::Rng;
        use rand::SeedableRng;
        let ref mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let mut game = Game6::root();
        let total = game.total();
        for _ in 0..64 {
            let mut steps = 0;
            while game.turn() != Turn::Terminal {
                let legal = game.legal();
                let action = legal[rng.random_range(0..legal.len())];
                game = game.apply(action);
                steps += 1;
                assert!(steps < 512, "hand must terminate");
            }
            match game.continuation() {
                Some(next) => {
                    assert_eq!(next.total(), total, "chips must be conserved across hands");
                    assert_eq!(next.street(), Street::Pref);
                    assert_eq!(next.pot(), Game6::sblind() + Game6::bblind());
                    game = next;
                }
                None => break, // someone busted; session over
            }
        }
    }
}
