use super::*;

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
