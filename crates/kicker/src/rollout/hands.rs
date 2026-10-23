use super::*;

/// Iterator over terminal `Game` states, one per hand, stopping on bust.
pub struct Hands(Game);
impl Hands {
    pub fn new(game: Game) -> Self {
        Self(game)
    }
}
impl Iterator for Hands {
    type Item = Game;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.0.must_stop() {
            let actions = self.0.legal();
            let action = actions[rand::random_range(0..actions.len())];
            self.0 = self.0.apply(action);
        }
        let terminal = self.0;
        self.0 = self.0.continuation()?;
        Some(terminal)
    }
}
