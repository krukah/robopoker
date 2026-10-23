use crate::*;
use kicker::*;
use mccfr::*;
use neural::*;
use pokerkit::*;

/// What both players know: the hand's [`History`], the legal edges here, the chips, who acts,
/// and the button.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FullPublic {
    history: History,
    choices: Path,
    ledger: Ledger,
    turn: Turn,
    button: Position,
}

impl FullPublic {
    pub fn new(history: History, choices: Path, ledger: Ledger, turn: Turn, button: Position) -> Self {
        Self {
            history,
            choices,
            ledger,
            turn,
            button,
        }
    }

    pub fn history(&self) -> History {
        self.history
    }

    pub fn ledger(&self) -> Ledger {
        self.ledger
    }

    pub fn turn(&self) -> Turn {
        self.turn
    }

    pub fn button(&self) -> Position {
        self.button
    }
}

impl CfrPublic for FullPublic {
    type G = NlheGame;

    fn menu(&self) -> impl Iterator<Item = NlheEdge> + use<> {
        self.choices.into_iter().map(NlheEdge::from)
    }

    fn count(&self) -> usize {
        self.choices.length()
    }
}

/// The [`History`], then the chips as the actor sees them (pot, every stack from the actor's
/// seat on, pot odds, SPR squashed into `[0, 1)`), then street and seat one-hot and the legal
/// edges.
impl Block for FullPublic {
    const WIDTH: usize = History::WIDTH + (N + 3) + (4 + N + NlheEdge::CHOICES);

    fn values(self) -> impl Iterator<Item = f32> {
        self.history.values().chain(self.chips()).chain(self.spot())
    }
}

impl FullPublic {
    fn chips(self) -> impl Iterator<Item = f32> {
        let pot = f32::from(self.ledger.pot());
        let call = f32::from(self.ledger.call());
        let spr = f32::from(self.ledger.effective()) / pot;
        std::iter::once(pot / f32::from(STACK) / N as f32)
            .chain((0..N).map(move |offset| self.stack(offset) / f32::from(STACK)))
            .chain(std::iter::once(call / (pot + call)))
            .chain(std::iter::once(spr / (1. + spr)))
    }

    fn spot(self) -> impl Iterator<Item = f32> {
        let street = self.history.street() as usize;
        let seat = (self.turn.position() + N - self.button) % N;
        let legal = self.menu().map(|edge| edge.index()).collect::<Vec<_>>();
        (0..4)
            .map(move |i| f32::from(i == street))
            .chain((0..N).map(move |i| f32::from(i == seat)))
            .chain((0..NlheEdge::CHOICES).map(move |i| f32::from(legal.contains(&i))))
    }

    fn stack(self, offset: usize) -> f32 {
        f32::from(self.ledger.stacks()[(self.turn.position() + offset) % N])
    }
}
