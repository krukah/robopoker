use crate::Chips;
use crate::cards::*;
use crate::gameplay::*;

/// Events broadcast by Room to all participants.
/// Clean separation between game actions, meta actions, and revelations.
#[derive(Clone, Debug)]
pub enum Event {
    Play(Action),
    NextHand(usize, Meta),
    ShowHand(usize, Hole),
    YourTurn(Recall),
}

/// Meta-actions for table and player management.
/// These are not part of the core poker game logic.
/// Position is lifted to the GameEvent::Meta variant.
#[derive(Clone, Debug)]
pub enum Meta {
    StandUp,
    SitDown,
    CashOut(Chips),
}
