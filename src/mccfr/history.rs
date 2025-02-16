use crate::gameplay::action::Action;
use crate::gameplay::game::Game;

#[derive(Debug, Clone)]
pub struct History {
    root: Game,
    path: Vec<Action>,
}

impl History {
    pub fn game(&self) -> Game {
        self.path
            .iter()
            .cloned()
            .fold(self.root.clone(), |game, action| game.apply(action))
    }
    pub fn undo(&mut self) -> () {
        // i guess we constrain history to always include
        // blinds, and thus be in a maximally advanced state
        assert!(!self.path.iter().all(|a| a.is_blind()));
        self.path.pop();
    }
    pub fn push(&mut self, action: Action) -> () {
        assert!(self.game().is_legal(&action));
        self.path.push(action);
    }
}

impl From<History> for Game {
    fn from(history: History) -> Self {
        history.game()
    }
}
