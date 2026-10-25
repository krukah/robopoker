use super::*;

/// Iterator over actions across multiple hands, stopping on bust.
pub struct Session(Game);
impl Session {
    pub fn new(game: Game) -> Self {
        Self(game)
    }
}
impl Iterator for Session {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let actions = self.0.legal();
            if !actions.is_empty() {
                let action = actions[rand::random_range(0..actions.len())];
                self.0 = self.0.apply(action);
                return Some(action);
            }
            self.0 = self.0.continuation()?;
        }
    }
}
