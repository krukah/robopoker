use super::*;
use deuce::*;
use pokerkit::*;

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
        *self = Self {
            root,
            epoch,
            ..Self::default()
        };
    }

    pub fn deal_hole(&mut self, seat: Position, hole: Hole) {
        self.holes_mut()[seat] = Some(hole);
    }

    pub fn deal(&mut self, hand: Hand) {
        self.actions_mut().push(Action::Draw(hand));
    }

    pub fn act(&mut self, action: Action) {
        if action.is_choice() {
            self.actions_mut().push(action);
        }
    }

    pub fn show(&mut self, seat: Position, hole: Hole) {
        self.reveals_mut()[seat] = Some(hole);
    }

    pub fn settle(&mut self, settlements: Vec<Settlement>) {
        *self.settlements_mut() = settlements;
    }
}
impl LiveGame {
    pub fn game(&self) -> Game {
        self.actions().iter().copied().fold(
            self.holes()
                .iter()
                .enumerate()
                .fold(*self.root(), |g, (i, hole)| hole.map_or(g, |h| g.deal(i, h))),
            |mut g, a| g.consume(a),
        )
    }

    pub fn root(&self) -> &Game {
        &self.root
    }

    fn holes_mut(&mut self) -> &mut [Option<Hole>; N] {
        &mut self.holes
    }

    fn reveals(&self) -> &[Option<Hole>; N] {
        &self.shown
    }

    fn reveals_mut(&mut self) -> &mut [Option<Hole>; N] {
        &mut self.shown
    }

    fn actions_mut(&mut self) -> &mut Vec<Action> {
        &mut self.actions
    }

    fn settlements_mut(&mut self) -> &mut Vec<Settlement> {
        &mut self.settlements
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Lifecycle phase, derived from holes + settlements + game state.
    pub fn phase(&self) -> Phase {
        if !self.settlements().is_empty() {
            return Phase::Settled;
        }
        let game = self.game();
        if game.turn() == Turn::Terminal && game.is_showdown() {
            Phase::Showdown
        } else if self.holes().iter().any(Option::is_some) {
            Phase::Playing
        } else {
            Phase::Waiting
        }
    }

    pub fn hole(&self, seat: Position) -> Option<Hole> {
        self.holes().get(seat).copied().flatten()
    }

    pub fn holes(&self) -> &[Option<Hole>] {
        &self.holes
    }

    pub fn shown(&self, seat: Position) -> Option<Hole> {
        self.reveals().get(seat).copied().flatten()
    }

    pub fn visible(&self, seat: Position, hero: Position) -> Option<Hole> {
        if seat == hero { self.hole(seat) } else { self.shown(seat) }
    }

    pub fn opening(&self) -> [Chips; N] {
        self.root().buyins()
    }

    pub fn settlements(&self) -> &[Settlement] {
        &self.settlements
    }

    /// Community cards in deal order, deduplicated via bitmask.
    pub fn dealt(&self) -> Vec<Card> {
        let mut seen = Hand::empty();
        self.actions()
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
        let cards = Vec::<Card>::from(Hand::from(hole))
            .into_iter()
            .chain(self.dealt())
            .collect::<Vec<Card>>();
        Some(self.actions().iter().copied().filter(Action::is_choice).fold(
            Witness::initial_with(
                Turn::Choice(seat),
                Arrangement::from(cards),
                self.root().buyins(),
                self.root().dealer().position(),
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
        for action in self.actions() {
            if action.is_chance() {
                rebuilt.push(*action);
            } else if ci < server.len() {
                rebuilt.push(server[ci]);
                ci += 1;
            }
        }
        rebuilt.extend_from_slice(&server[ci..]);
        *self.actions_mut() = rebuilt;
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
