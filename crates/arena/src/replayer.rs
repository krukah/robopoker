use super::*;
use deuce::*;
use kicker::*;
use parlor::records::{Hand as HandRecord, Participant, Play};

/// Walks a stored hand through the game engine, yielding each decision point.
///
/// Encapsulates the board-card partitioning and chance-node advancement that
/// every consumer (replay, AIVAT, hand display) needs. Consumers iterate
/// `steps()` and react to each `Step` without duplicating the walk logic.
pub struct Replayer {
    game: Game,
    board: Vec<Card>,
    cursor: usize,
}
impl Replayer {
    /// Build a replayer from database records.
    ///
    /// When `plays` contains [`Draw`](Action::Draw) actions, the board
    /// ordering is derived from those draws (preserving actual deal order).
    /// Falls back to [`Hand`] iteration order for older hands without draws.
    pub fn new(hand: &HandRecord, participants: &[Participant], plays: &[Play]) -> anyhow::Result<Self> {
        let stacks = Replay::stacks(participants)?;
        let game = participants
            .iter()
            .fold(Game::from_start(hand.dealer(), stacks), |g, p| g.deal(p.seat(), p.hole()));
        let draws = plays
            .iter()
            .map(parlor::Play::action)
            .filter_map(Action::hand)
            .flatten()
            .collect::<Vec<Card>>();
        let board = if draws.is_empty() {
            Vec::<Card>::from(Hand::from(hand.board()))
        } else {
            draws
        };
        Ok(Self { game, board, cursor: 0 })
    }
    /// Current game state.
    pub fn game(&self) -> &Game {
        &self.game
    }
    /// Advance through chance nodes, then apply the given action.
    /// Returns the street the action was taken on.
    pub fn advance(&mut self, action: Action) -> anyhow::Result<Street> {
        self.deal()?;
        let street = self.game().street();
        self.apply(action)?;
        Ok(street)
    }
    /// Apply a single action without dealing through chance nodes.
    pub fn apply(&mut self, action: Action) -> anyhow::Result<()> {
        *self.game_mut() = self.game().try_apply(action)?;
        Ok(())
    }
    /// Advance through any remaining chance nodes to reach terminal.
    pub fn finish(&mut self) -> anyhow::Result<()> {
        self.deal()
    }
    /// Advance past all chance nodes.
    pub fn deal(&mut self) -> anyhow::Result<()> {
        while self.game().turn() == Turn::Chance {
            self.deal_one()?;
        }
        Ok(())
    }
    /// Peek at the cards about to be dealt at a chance node.
    pub fn peek_deal(&self) -> Option<Hand> {
        (self.game().turn() == Turn::Chance).then(|| self.upcoming())
    }
    /// Advance exactly one chance node (one street's deal).
    pub fn deal_one(&mut self) -> anyhow::Result<()> {
        if self.game().turn() == Turn::Chance {
            let n = self.reveals();
            anyhow::ensure!(
                self.cursor() + n <= self.board().len(),
                "not enough board cards at cursor {}",
                self.cursor()
            );
            self.apply(Action::Draw(self.upcoming()))?;
            *self.cursor_mut() += n;
        }
        Ok(())
    }
    /// Is the game terminal?
    pub fn terminal(&self) -> bool {
        self.game().turn() == Turn::Terminal
    }

    fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }

    fn board(&self) -> &[Card] {
        &self.board
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn cursor_mut(&mut self) -> &mut usize {
        &mut self.cursor
    }

    /// Cards the next chance node reveals.
    fn reveals(&self) -> usize {
        self.game().street().next().n_revealed()
    }

    fn upcoming(&self) -> Hand {
        self.board()[self.cursor()..self.cursor() + self.reveals()]
            .iter()
            .copied()
            .map(Hand::from)
            .fold(Hand::empty(), Hand::add)
    }
}
