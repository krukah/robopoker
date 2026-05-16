use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_gameroom::records::{Hand as HandRecord, Participant, Play};

/// Per-hand result from replaying a stored hand.
#[derive(Debug, Clone)]
pub struct Recap {
    hand: ID<HandRecord>,
    seat: Position,
    hole: Hole,
    stack: Chips,
    won: Chips,
    pot: Chips,
    street: Street,
    folded: bool,
    showdown: bool,
    vpip: bool,
    pfr: bool,
    actions: Vec<(Street, Action)>,
}

impl Recap {
    pub fn hand(&self) -> ID<HandRecord> {
        self.hand
    }

    pub fn seat(&self) -> Position {
        self.seat
    }

    pub fn hole(&self) -> Hole {
        self.hole
    }

    pub fn stack(&self) -> Chips {
        self.stack
    }

    pub fn won(&self) -> Chips {
        self.won
    }

    pub fn pot(&self) -> Chips {
        self.pot
    }

    pub fn street(&self) -> Street {
        self.street
    }

    pub fn folded(&self) -> bool {
        self.folded
    }

    pub fn showdown(&self) -> bool {
        self.showdown
    }

    pub fn vpip(&self) -> bool {
        self.vpip
    }

    pub fn pfr(&self) -> bool {
        self.pfr
    }

    pub fn actions(&self) -> &[(Street, Action)] {
        &self.actions
    }
}

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
    pub fn new(
        hand: &HandRecord,
        participants: &[Participant],
        plays: &[Play],
    ) -> anyhow::Result<Self> {
        let stacks = stacks(participants)?;
        let game = participants
            .iter()
            .fold(Game::from_start(hand.dealer(), stacks), |g, p| {
                g.deal(p.seat(), p.hole())
            });
        let draws: Vec<Card> = plays
            .iter()
            .map(|p| p.action())
            .filter_map(|a| a.hand())
            .flatten()
            .collect();
        let board = if draws.is_empty() {
            Vec::<Card>::from(Hand::from(hand.board()))
        } else {
            draws
        };
        Ok(Self {
            game,
            board,
            cursor: 0,
        })
    }
    /// Current game state.
    pub fn game(&self) -> &Game {
        &self.game
    }
    /// Advance through chance nodes, then apply the given action.
    /// Returns the street the action was taken on.
    pub fn advance(&mut self, action: Action) -> anyhow::Result<Street> {
        self.deal()?;
        let street = self.game.street();
        self.game = self.game.try_apply(action)?;
        Ok(street)
    }
    /// Apply a single action without dealing through chance nodes.
    pub fn apply(&mut self, action: Action) -> anyhow::Result<()> {
        self.game = self.game.try_apply(action)?;
        Ok(())
    }
    /// Advance through any remaining chance nodes to reach terminal.
    pub fn finish(&mut self) -> anyhow::Result<()> {
        self.deal()
    }
    /// Advance past all chance nodes.
    pub fn deal(&mut self) -> anyhow::Result<()> {
        while self.game.turn() == Turn::Chance {
            let n = self.game.street().next().n_revealed();
            anyhow::ensure!(
                self.cursor + n <= self.board.len(),
                "not enough board cards at cursor {}",
                self.cursor
            );
            let cards = self.board[self.cursor..self.cursor + n]
                .iter()
                .copied()
                .map(Hand::from)
                .fold(Hand::empty(), Hand::add);
            self.cursor += n;
            self.game = self.game.try_apply(Action::Draw(cards))?;
        }
        Ok(())
    }
    /// Peek at the cards about to be dealt at a chance node.
    pub fn peek_deal(&self) -> Option<Hand> {
        (self.game.turn() == Turn::Chance).then(|| {
            let n = self.game.street().next().n_revealed();
            self.board[self.cursor..self.cursor + n]
                .iter()
                .copied()
                .map(Hand::from)
                .fold(Hand::empty(), Hand::add)
        })
    }
    /// Advance exactly one chance node (one street's deal).
    pub fn deal_one(&mut self) -> anyhow::Result<()> {
        if self.game.turn() == Turn::Chance {
            let n = self.game.street().next().n_revealed();
            anyhow::ensure!(
                self.cursor + n <= self.board.len(),
                "not enough board cards at cursor {}",
                self.cursor
            );
            let cards = self.board[self.cursor..self.cursor + n]
                .iter()
                .copied()
                .map(Hand::from)
                .fold(Hand::empty(), Hand::add);
            self.cursor += n;
            self.game = self.game.try_apply(Action::Draw(cards))?;
        }
        Ok(())
    }
    /// Is the game terminal?
    pub fn terminal(&self) -> bool {
        self.game.turn() == Turn::Terminal
    }
}

/// Replay a stored hand to extract per-player results.
pub fn replay(
    hand: &HandRecord,
    participants: &[Participant],
    plays: &[Play],
    seat: Position,
) -> anyhow::Result<Recap> {
    let participant = participants
        .iter()
        .find(|p| p.seat() == seat)
        .ok_or_else(|| anyhow::anyhow!("seat {} not found", seat))?;
    let reveals = plays_arrangement(participant.hole(), hand.board(), plays);
    let witness = plays
        .iter()
        .filter(|p| !p.action().is_blind())
        .filter(|p| !p.action().is_chance())
        .try_fold(
            Witness::initial_with(
                Turn::Choice(seat),
                reveals,
                stacks(participants)?,
                hand.dealer(),
            ),
            |r, p| r.try_push(p.action()),
        )?;
    let actions = witness
        .plays()
        .into_iter()
        .filter(|&(pos, _, _)| pos == seat)
        .map(|(_, a, s)| (s, a))
        .collect::<Vec<(Street, Action)>>();
    let folded = actions.iter().any(|(_, a)| matches!(a, Action::Fold));
    Ok(Recap {
        hand: hand.id(),
        seat,
        hole: participant.hole(),
        stack: participant.stack(),
        won: participant.pnl(),
        pot: hand.pot(),
        street: witness.head().street(),
        folded,
        showdown: witness.head().is_showdown(),
        vpip: actions.iter().any(|(s, a)| {
            *s == Street::Pref && matches!(a, Action::Call(_) | Action::Raise(_) | Action::Shove(_))
        }),
        pfr: actions
            .iter()
            .any(|(s, a)| *s == Street::Pref && matches!(a, Action::Raise(_) | Action::Shove(_))),
        actions,
    })
}

/// Build the stacks array from participants.
pub fn stacks(participants: &[Participant]) -> anyhow::Result<[Chips; N]> {
    participants.iter().try_fold([0i16; N], |mut acc, p| {
        anyhow::ensure!(p.seat() < N, "seat {} out of bounds for N={}", p.seat(), N);
        acc[p.seat()] = p.stack();
        Ok(acc)
    })
}
/// Build an [`Arrangement`] from a hole and board stored as unordered sets.
///
/// Deterministic per-street card assignment via [`Hand`] iteration order
/// (low-to-high). Matches [`Replayer`]'s board ordering so [`Witness`]
/// Draw actions stay consistent with the Replayer's deals.
pub fn board_arrangement(hole: Hole, board: Board) -> Arrangement {
    Arrangement::from(
        Hand::from(hole)
            .chain(Hand::from(board))
            .collect::<Vec<Card>>(),
    )
}
/// Build an [`Arrangement`] from a hole and the [`Draw`](Action::Draw)
/// actions in a play sequence, preserving the actual deal order.
///
/// Falls back to [`board_arrangement`] when no Draw actions are present
/// (backward compatibility with hands recorded before draws were persisted).
pub fn plays_arrangement(hole: Hole, board: Board, plays: &[Play]) -> Arrangement {
    let draws: Vec<Card> = plays
        .iter()
        .map(|p| p.action())
        .filter_map(|a| a.hand())
        .flatten()
        .collect();
    if draws.is_empty() {
        board_arrangement(hole, board)
    } else {
        Arrangement::from(Hand::from(hole).chain(draws).collect::<Vec<Card>>())
    }
}
