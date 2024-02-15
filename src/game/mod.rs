use crate::cards::{Board, Card, Hole};

// data structures
pub struct Player {
    stack: u32,
    wager: u32,
    cards: Hole,
}

pub struct Node {
    pot: u32,
    board: Board,
    actor: Option<Player>,
}

pub enum Action {
    Check,
    Fold,
    Skip,
    Call(u32),
    Open(u32),
    Shove(u32),
    Raise(u32),
    Deal(Card), // random draw
}

pub struct Tree {
    root: Node,
    history: Vec<Action>,
    paths_considered: Vec<Vec<Action>>,
    paths_eliminated: Vec<Vec<Action>>,
}

pub enum Street {
    PreFlop,
    Flop,
    Turn,
    River,
}

// algos
impl Node {
    fn apply(&mut self, action: Action) {
        match action {
            Action::Skip | Action::Check => (),
            Action::Deal(card) => self.board.deal(card),
            Action::Call(bet) | Action::Shove(bet) | Action::Raise(bet) | Action::Open(bet) => {
                self.pot += bet;
                self.actor.unwrap().wager += bet;
                self.actor.unwrap().stack -= bet;
            }
            Action::Fold => {
                self.pot += self.actor.unwrap().wager;
                self.actor.unwrap().wager = 0;
            }
        }
    }
}
