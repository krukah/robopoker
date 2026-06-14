use super::*;
use kicker::*;
use pokerkit::*;

/// Lifecycle phase of a live hand. Derived from `LiveGame` state — never
/// stored, never set explicitly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Waiting,
    Playing,
    Showdown,
    Settled,
}

/// Shared live game state for both frontend and backend.
///
/// `game()` is derived from `root + holes + actions` on each call.
/// `phase()` is derived from `(holes, settlements, game.is_showdown())`.
#[derive(Clone, Debug)]
pub struct LiveGame {
    root: Game,
    holes: [Option<Hole>; N],
    shown: [Option<Hole>; N],
    actions: Vec<Action>,
    settlements: Vec<Settlement>,
    epoch: u64,
}

impl Default for LiveGame {
    fn default() -> Self {
        Self {
            root: Game::root(),
            epoch: 0,
            holes: [None; N],
            shown: [None; N],
            actions: Vec::new(),
            settlements: Vec::new(),
        }
    }
}

/// State-mutation surface. Each method captures one transition; preconditions
/// are documented per-method. Phase derives from state — no method writes it.
impl LiveGame {
    pub fn start(&mut self, epoch: u64, root: Game) {
        self.epoch = epoch;
        self.root = root;
        self.holes = [None; N];
        self.shown = [None; N];
        self.actions = Vec::new();
        self.settlements = Vec::new();
    }

    pub fn deal_hole(&mut self, seat: Position, hole: Hole) {
        self.holes[seat] = Some(hole);
    }

    pub fn deal(&mut self, hand: Hand) {
        self.actions.push(Action::Draw(hand));
    }

    pub fn act(&mut self, action: Action) {
        if action.is_choice() {
            self.actions.push(action);
        }
    }

    pub fn show(&mut self, seat: Position, hole: Hole) {
        self.shown[seat] = Some(hole);
    }

    pub fn settle(&mut self, settlements: Vec<Settlement>) {
        self.settlements = settlements;
    }
}

impl LiveGame {
    pub fn game(&self) -> Game {
        self.actions.iter().copied().fold(
            self.holes
                .iter()
                .enumerate()
                .fold(self.root, |g, (i, hole)| hole.map_or(g, |h| g.deal(i, h))),
            |mut g, a| g.consume(a),
        )
    }

    pub fn root(&self) -> &Game {
        &self.root
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Lifecycle phase, derived from holes + settlements + game state.
    pub fn phase(&self) -> Phase {
        if !self.settlements.is_empty() {
            return Phase::Settled;
        }
        let game = self.game();
        if game.turn() == Turn::Terminal && game.is_showdown() {
            Phase::Showdown
        } else if self.holes.iter().any(Option::is_some) {
            Phase::Playing
        } else {
            Phase::Waiting
        }
    }

    pub fn hole(&self, seat: Position) -> Option<Hole> {
        self.holes.get(seat).copied().flatten()
    }

    pub fn holes(&self) -> &[Option<Hole>] {
        &self.holes
    }

    pub fn shown(&self, seat: Position) -> Option<Hole> {
        self.shown.get(seat).copied().flatten()
    }

    pub fn visible(&self, seat: Position, hero: Position) -> Option<Hole> {
        if seat == hero { self.hole(seat) } else { self.shown(seat) }
    }

    pub fn opening(&self) -> [Chips; N] {
        self.root.buyins()
    }

    pub fn settlements(&self) -> &[Settlement] {
        &self.settlements
    }

    /// Community cards in deal order, deduplicated via bitmask.
    pub fn dealt(&self) -> Vec<Card> {
        let mut seen = Hand::empty();
        self.actions
            .iter()
            .filter_map(|a| match a {
                Action::Draw(h) => Some(*h),
                _ => None,
            })
            .flat_map(Vec::<Card>::from)
            .filter(|&c| {
                let prev = seen;
                seen = Hand::or(seen, Hand::from_iter([c]));
                seen != prev
            })
            .collect()
    }

    pub fn is_playing(&self) -> bool {
        self.phase() == Phase::Playing
    }

    pub fn is_showdown(&self) -> bool {
        self.phase() == Phase::Showdown
    }

    pub fn is_settled(&self) -> bool {
        self.phase() == Phase::Settled
    }
}

/// Recall reconstruction.
impl LiveGame {
    /// Reconstruct perfect-recall history from visible state at a given seat.
    pub fn recall(&self, seat: Position) -> Option<Witness> {
        let hole = self.hole(seat)?;
        let cards: Vec<Card> = Vec::<Card>::from(Hand::from(hole))
            .into_iter()
            .chain(self.dealt())
            .collect();
        Some(self.actions.iter().filter(|a| a.is_choice()).copied().fold(
            Witness::initial_with(
                Turn::Choice(seat),
                Arrangement::from(cards),
                self.root.buyins(),
                self.root.dealer().position(),
            ),
            |r, a| r.push(a),
        ))
    }

    /// All actions applied since hand start (choice + draw, no blinds).
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Sync choice actions from server-authoritative recall, preserving local Draw actions.
    pub fn sync(&mut self, recall: &Witness) {
        let server = recall.actions();
        let mut rebuilt = Vec::new();
        let mut ci = 0;
        for action in &self.actions {
            if action.is_chance() {
                rebuilt.push(*action);
            } else if ci < server.len() {
                rebuilt.push(server[ci]);
                ci += 1;
            }
        }
        rebuilt.extend_from_slice(&server[ci..]);
        self.actions = rebuilt;
    }
}

impl Recall for LiveGame {
    fn root(&self) -> Game {
        self.root
    }

    fn actions(&self) -> &[Action] {
        &self.actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh(epoch: u64) -> LiveGame {
        let mut live = LiveGame::default();
        live.start(epoch, Game::from_start(0, [STACK; N]));
        live
    }

    #[test]
    fn start_resets_state() {
        let live = fresh(1);
        assert_eq!(live.epoch(), 1);
        assert_eq!(live.phase(), Phase::Waiting);
        assert!(live.holes().iter().all(Option::is_none));
        assert!(live.dealt().is_empty());
        assert!(live.actions().is_empty());
        assert!(live.settlements().is_empty());
    }

    #[test]
    fn deal_hole_transitions_to_playing() {
        let mut live = fresh(1);
        let mut deck = Deck::new();
        let h0 = deck.hole();
        live.deal_hole(0, h0);
        assert_eq!(live.hole(0), Some(h0));
        assert_eq!(live.hole(1), None);
        assert_eq!(live.phase(), Phase::Playing);
    }

    #[test]
    fn deal_appends_draw_action() {
        let mut live = fresh(1);
        let mut deck = Deck::new();
        live.deal_hole(0, deck.hole());
        live.deal_hole(1, deck.hole());
        live.act(Action::Call(1));
        live.act(Action::Check);
        live.deal(deck.deal(Street::Pref));
        assert_eq!(live.dealt().len(), 3);
        assert_eq!(live.game().street(), Street::Flop);
    }

    #[test]
    fn show_writes_to_shown_not_holes() {
        let mut live = fresh(1);
        let mut deck = Deck::new();
        let h1 = deck.hole();
        live.show(1, h1);
        assert_eq!(live.hole(1), None);
        assert_eq!(live.shown(1), Some(h1));
    }

    #[test]
    fn settle_transitions_to_settled() {
        let mut live = fresh(1);
        let mut deck = Deck::new();
        let strength = Strength::from(Hand::from(deck.hole()));
        live.settle(vec![Settlement::from((0, State::Folding, strength))]);
        assert_eq!(live.phase(), Phase::Settled);
    }

    #[test]
    fn consecutive_hands_reset_state() {
        let mut live = fresh(1);
        let mut deck = Deck::new();
        let h1 = deck.hole();
        live.show(1, h1);
        assert_eq!(live.shown(1), Some(h1));
        live.start(2, Game::from_start(0, [STACK; N]));
        assert_eq!(live.epoch(), 2);
        assert_eq!(live.phase(), Phase::Waiting);
        assert!(live.holes().iter().all(Option::is_none));
        assert_eq!(live.shown(1), None);
        assert!(live.actions().is_empty());
    }

    #[test]
    fn visible_pov_logic() {
        let mut live = fresh(1);
        let mut deck = Deck::new();
        let h0 = deck.hole();
        let h1 = deck.hole();
        live.deal_hole(0, h0);
        live.deal_hole(1, h1);
        assert_eq!(live.visible(0, 0), Some(h0));
        assert_eq!(live.visible(1, 0), None);
        live.show(1, h1);
        assert_eq!(live.visible(1, 0), Some(h1));
        assert_eq!(live.visible(1, 1), Some(h1));
        assert_eq!(live.hole(1), Some(h1));
    }

    #[test]
    fn recall_builds_partial() {
        let mut live = fresh(1);
        let mut deck = Deck::new();
        live.deal_hole(0, deck.hole());
        live.deal_hole(1, deck.hole());
        live.act(Action::Call(1));
        let witness = live.recall(0).expect("has hole cards");
        assert_eq!(witness.turn(), Turn::Choice(0));
        assert_eq!(witness.actions().len(), 1);
    }

    #[test]
    fn recall_none_without_hole_cards() {
        let live = fresh(1);
        assert!(live.recall(0).is_none());
    }

    #[test]
    fn act_tracks_choice_only() {
        let mut live = fresh(1);
        let mut deck = Deck::new();
        live.deal_hole(0, deck.hole());
        live.deal_hole(1, deck.hole());
        live.act(Action::Call(1));
        live.act(Action::Check);
        assert_eq!(live.actions().len(), 2);
        live.deal(deck.deal(Street::Pref));
        assert_eq!(live.actions().len(), 3);
        assert!(live.actions()[2].is_chance());
    }
}
