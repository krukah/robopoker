use rbp_cards::Street;
use rbp_gameplay::*;

/// Wraps Game with common dealing and betting operations.
/// Provides a convenience API for applying actions and checking game state.
#[derive(Debug)]
pub struct Dealer<'g> {
    game: &'g mut Game,
}

impl<'g> Dealer<'g> {
    /// Creates a new Dealer wrapping the given game.
    pub fn new(game: &'g mut Game) -> Self {
        Self { game }
    }
    /// Deals the next street's cards, returning the street and resulting game state.
    /// Panics if the game is not ready to deal (not at a chance node).
    pub fn deal(&mut self) -> Street {
        *self.game = self.game.apply(self.game.reveal());
        self.game.street()
    }
    /// Applies an action to the game.
    /// Panics if the action is not legal.
    pub fn apply(&mut self, action: Action) {
        *self.game = self.game.apply(action);
    }
    /// Returns the passive action (check if allowed, else fold).
    pub fn passive(&self) -> Action {
        self.game.passive()
    }
    /// Returns all legal actions in the current state.
    pub fn legal(&self) -> Vec<Action> {
        self.game.legal()
    }
    /// Checks if a specific action is legal.
    pub fn is_allowed(&self, action: &Action) -> bool {
        self.game.is_allowed(action)
    }
    /// Returns true if the game has reached a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self.game.turn(), Turn::Terminal)
    }
    /// Returns true if the game is at a showdown (multiple active players at terminal).
    pub fn is_showdown(&self) -> bool {
        self.game.is_showdown()
    }
    /// Returns true if the game is at a chance node (needs to deal).
    pub fn is_chance(&self) -> bool {
        matches!(self.game.turn(), Turn::Chance)
    }
    /// Returns whose turn it is (Choice, Chance, or Terminal).
    pub fn turn(&self) -> Turn {
        self.game.turn()
    }
    /// Returns the current street.
    pub fn street(&self) -> Street {
        self.game.street()
    }
    /// Returns a reference to the underlying game.
    pub fn game(&self) -> &Game {
        self.game
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dealer_wraps_game() {
        let mut game = Game::root();
        let dealer = Dealer::new(&mut game);
        assert!(!dealer.is_terminal());
        assert!(!dealer.is_chance());
        assert!(matches!(dealer.turn(), Turn::Choice(_)));
    }
    #[test]
    fn dealer_applies_action() {
        let mut game = Game::root();
        let pot_before = game.pot();
        {
            let mut dealer = Dealer::new(&mut game);
            dealer.apply(Action::Call(1));
        }
        assert!(game.pot() > pot_before);
    }
    #[test]
    fn dealer_deals_flop() {
        let mut game = Game::root();
        {
            let mut dealer = Dealer::new(&mut game);
            dealer.apply(Action::Call(1));
            dealer.apply(Action::Check);
            assert!(dealer.is_chance());
            let street = dealer.deal();
            assert_eq!(street, Street::Flop);
        }
    }
    #[test]
    fn dealer_passive_action() {
        let mut game = Game::root();
        {
            let mut dealer = Dealer::new(&mut game);
            dealer.apply(Action::Call(1));
            // BB can check after limp
            assert_eq!(dealer.passive(), Action::Check);
        }
    }
    #[test]
    fn dealer_legal_actions() {
        let mut game = Game::root();
        let dealer = Dealer::new(&mut game);
        let legal = dealer.legal();
        assert!(legal.contains(&Action::Fold));
        assert!(legal.iter().any(|a| matches!(a, Action::Call(_))));
    }
    #[test]
    fn dealer_terminal_after_fold() {
        let mut game = Game::root();
        {
            let mut dealer = Dealer::new(&mut game);
            dealer.apply(Action::Fold);
            assert!(dealer.is_terminal());
        }
    }
}
